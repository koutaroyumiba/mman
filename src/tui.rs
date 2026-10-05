use std::io;

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout as RatatuiLayout, Rect},
    style::{Color, Modifier, Style},
    text::{Line as TuiLine, Span as TuiSpan},
    widgets::{Block, Borders, Clear, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::{Action, SearchState, ViewBounds, ViewerMode, ViewerState},
    layout::{LineKind, RenderedSpan, layout_document},
    markdown::parse_markdown,
};

const BASE: Color = Color::Rgb(25, 23, 36);
const SURFACE: Color = Color::Rgb(31, 29, 46);
const OVERLAY: Color = Color::Rgb(38, 35, 58);
const MUTED: Color = Color::Rgb(110, 106, 134);
const TEXT: Color = Color::Rgb(224, 222, 244);
const GOLD: Color = Color::Rgb(246, 193, 119);
const ROSE: Color = Color::Rgb(235, 188, 186);
const PINE: Color = Color::Rgb(49, 116, 143);
const FOAM: Color = Color::Rgb(156, 207, 216);
const IRIS: Color = Color::Rgb(196, 167, 231);

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;

        if let Err(error) = execute!(io::stdout(), EnterAlternateScreen, Hide) {
            let _ = disable_raw_mode();
            return Err(error);
        }

        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

pub fn run_viewer(topic: &str, source: &str) -> io::Result<()> {
    let document = parse_markdown(source);
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut state = ViewerState::default();
    let mut bounds = ViewBounds {
        line_count: 0,
        viewport_height: 0,
        widest_line: 0,
        viewport_width: 0,
    };
    let mut plain_lines = Vec::new();

    while !state.should_quit {
        terminal.draw(|frame| {
            (bounds, plain_lines) = render_viewer(frame, topic, &document, &mut state);
        })?;

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if let Some(action) = action_for_key(key, &state) {
                    state.apply(action, bounds, &plain_lines);
                }
            }
            Event::Resize(_, _) => state.clamp(bounds),
            _ => {}
        }
    }

    Ok(())
}

fn render_viewer(
    frame: &mut Frame<'_>,
    topic: &str,
    document: &crate::document::Document,
    state: &mut ViewerState,
) -> (ViewBounds, Vec<String>) {
    let regions = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());
    let body = regions[0];
    let status = regions[1];
    let viewport_width = body.width.saturating_sub(2) as usize;
    let viewport_height = body.height.saturating_sub(2) as usize;
    let layout = layout_document(document, viewport_width);
    let widest_line = layout
        .lines
        .iter()
        .map(|line| UnicodeWidthStr::width(line.plain_text().as_str()))
        .max()
        .unwrap_or(0);
    let bounds = ViewBounds {
        line_count: layout.lines.len(),
        viewport_height,
        widest_line,
        viewport_width,
    };
    let plain_lines: Vec<String> = layout.lines.iter().map(|line| line.plain_text()).collect();
    state.refresh_search(&plain_lines);
    state.clamp(bounds);

    let rendered_lines: Vec<TuiLine<'static>> = layout
        .lines
        .iter()
        .enumerate()
        .map(|(index, line)| to_tui_line(line, index, &state.search))
        .collect();
    let title = format!(" mman · {topic} ");
    let page = Paragraph::new(rendered_lines)
        .style(Style::default().fg(TEXT).bg(BASE))
        .scroll((
            state.vertical_scroll.min(u16::MAX as usize) as u16,
            state.horizontal_scroll.min(u16::MAX as usize) as u16,
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(MUTED))
                .style(Style::default().bg(BASE)),
        );
    frame.render_widget(page, body);

    let position = if layout.lines.is_empty() {
        String::from("0/0")
    } else {
        format!("{}/{}", state.vertical_scroll + 1, layout.lines.len())
    };
    let status_text = match state.mode {
        ViewerMode::SearchInput => format!(" /{}", state.search.input),
        ViewerMode::Reading if !state.search.query.is_empty() => {
            let current = state.search.current.map_or(0, |index| index + 1);
            format!(
                " {position}  /{}  {current}/{}  n/N matches  Esc clear  q quit ",
                state.search.query,
                state.search.matches.len()
            )
        }
        ViewerMode::Reading => format!(
            " {position}  x:{}  j/k scroll  / search  ? help  q quit ",
            state.horizontal_scroll
        ),
    };
    frame.render_widget(
        Paragraph::new(status_text).style(Style::default().fg(MUTED).bg(SURFACE)),
        status,
    );

    if state.show_help {
        render_help(frame);
    }

    (bounds, plain_lines)
}

fn action_for_key(key: KeyEvent, state: &ViewerState) -> Option<Action> {
    if state.show_help {
        return match key.code {
            KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Esc => Some(Action::ToggleHelp),
            _ => None,
        };
    }

    if state.mode == ViewerMode::SearchInput {
        return match key {
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => Some(Action::SubmitSearch),
            KeyEvent {
                code: KeyCode::Esc, ..
            } => Some(Action::CancelSearch),
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => Some(Action::SearchBackspace),
            KeyEvent {
                code: KeyCode::Char(character),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            } => Some(Action::SearchCharacter(character)),
            _ => None,
        };
    }

    match key {
        KeyEvent {
            code: KeyCode::Char('q'),
            ..
        } => Some(Action::Quit),
        KeyEvent {
            code: KeyCode::Esc, ..
        } if !state.search.query.is_empty() => Some(Action::ClearSearch),
        KeyEvent {
            code: KeyCode::Esc, ..
        } => Some(Action::Quit),
        KeyEvent {
            code: KeyCode::Char('?'),
            ..
        } => Some(Action::ToggleHelp),
        KeyEvent {
            code: KeyCode::Char('/'),
            ..
        } => Some(Action::EnterSearch),
        KeyEvent {
            code: KeyCode::Char('n'),
            ..
        } => Some(Action::NextMatch),
        KeyEvent {
            code: KeyCode::Char('N'),
            ..
        } => Some(Action::PreviousMatch),
        KeyEvent {
            code: KeyCode::Char('j') | KeyCode::Down,
            ..
        } => Some(Action::ScrollDown),
        KeyEvent {
            code: KeyCode::Char('k') | KeyCode::Up,
            ..
        } => Some(Action::ScrollUp),
        KeyEvent {
            code: KeyCode::Char('h') | KeyCode::Left,
            ..
        } => Some(Action::ScrollLeft),
        KeyEvent {
            code: KeyCode::Char('l') | KeyCode::Right,
            ..
        } => Some(Action::ScrollRight),
        KeyEvent {
            code: KeyCode::PageDown,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Some(Action::PageDown),
        KeyEvent {
            code: KeyCode::PageUp,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Some(Action::PageUp),
        KeyEvent {
            code: KeyCode::Home | KeyCode::Char('g'),
            ..
        } => Some(Action::GoToTop),
        KeyEvent {
            code: KeyCode::End | KeyCode::Char('G'),
            ..
        } => Some(Action::GoToBottom),
        _ => None,
    }
}

fn render_help(frame: &mut Frame<'_>) {
    let area = centered_rect(frame.area(), 64, 72);
    frame.render_widget(Clear, area);

    let help = vec![
        TuiLine::from("Navigation").style(Style::default().fg(IRIS).bold()),
        TuiLine::from(""),
        TuiLine::from("  j / Down          Scroll down"),
        TuiLine::from("  k / Up            Scroll up"),
        TuiLine::from("  Ctrl-d / PageDown Move down one page"),
        TuiLine::from("  Ctrl-u / PageUp   Move up one page"),
        TuiLine::from("  h / Left          Pan left"),
        TuiLine::from("  l / Right         Pan right"),
        TuiLine::from("  g / Home          Go to beginning"),
        TuiLine::from("  G / End           Go to end"),
        TuiLine::from("  /                 Search in page"),
        TuiLine::from("  n / N             Next / previous match"),
        TuiLine::from("  Esc               Clear active search"),
        TuiLine::from("  ?                 Close help"),
        TuiLine::from("  q                 Close help"),
    ];
    let popup = Paragraph::new(help)
        .style(Style::default().fg(TEXT).bg(OVERLAY))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(Style::default().fg(IRIS))
                .style(Style::default().bg(OVERLAY)),
        );
    frame.render_widget(popup, area);
}

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn to_tui_line(
    line: &crate::layout::Line,
    line_index: usize,
    search: &SearchState,
) -> TuiLine<'static> {
    let mut spans = Vec::new();
    let mut byte_offset = 0;

    for span in &line.spans {
        let base_style = span_style(span, line.kind);
        for character in span.text.chars() {
            let match_index = search.matches.iter().position(|item| {
                item.line == line_index && item.start <= byte_offset && byte_offset < item.end
            });
            let style = match match_index {
                Some(index) if search.current == Some(index) => {
                    base_style.fg(BASE).bg(ROSE).add_modifier(Modifier::BOLD)
                }
                Some(_) => base_style.fg(BASE).bg(GOLD),
                None => base_style,
            };
            spans.push(TuiSpan::styled(character.to_string(), style));
            byte_offset += character.len_utf8();
        }
    }

    TuiLine::from(spans).style(line_style(line.kind))
}

fn line_style(kind: LineKind) -> Style {
    match kind {
        LineKind::Prose => Style::default().fg(TEXT).bg(BASE),
        LineKind::Heading(_) => Style::default()
            .fg(IRIS)
            .bg(BASE)
            .add_modifier(Modifier::BOLD),
        LineKind::Code => Style::default().fg(FOAM).bg(SURFACE),
        LineKind::CodeBorder => Style::default().fg(MUTED).bg(SURFACE),
        LineKind::ThematicBreak => Style::default().fg(MUTED).bg(BASE),
    }
}

fn span_style(span: &RenderedSpan, kind: LineKind) -> Style {
    let mut style = line_style(kind);

    if span.style.emphasis {
        style = style.fg(ROSE).add_modifier(Modifier::ITALIC);
    }
    if span.style.strong {
        style = style.fg(GOLD).add_modifier(Modifier::BOLD);
    }
    if span.style.inline_code && kind != LineKind::Code {
        style = style.fg(FOAM).bg(OVERLAY);
    }
    if span.link.is_some() {
        style = style.fg(PINE).add_modifier(Modifier::UNDERLINED);
    }
    if let Some(syntax) = span.syntax {
        style = style.fg(Color::Rgb(
            syntax.foreground.0,
            syntax.foreground.1,
            syntax.foreground.2,
        ));
        if syntax.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        if syntax.italic {
            style = style.add_modifier(Modifier::ITALIC);
        }
    }

    style
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{Terminal, backend::TestBackend};

    use super::{Action, ROSE, ViewerState, action_for_key, render_viewer};
    use crate::{app::ViewBounds, markdown::parse_markdown};

    #[test]
    fn test_backend_renders_topic_and_document() {
        let backend = TestBackend::new(60, 12);
        let mut terminal = Terminal::new(backend).expect("test terminal should be created");
        let document = parse_markdown("# NAME\n\nbody text\n");
        let mut state = ViewerState::default();

        terminal
            .draw(|frame| {
                render_viewer(frame, "example", &document, &mut state);
            })
            .expect("viewer should render");

        let contents: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(contents.contains("mman · example"));
        assert!(contents.contains("body text"));
    }

    #[test]
    fn test_backend_highlights_search_matches_and_shows_match_count() {
        let backend = TestBackend::new(60, 8);
        let mut terminal = Terminal::new(backend).expect("test terminal should be created");
        let document = parse_markdown("Needle and needle\n");
        let mut state = ViewerState::default();
        let bounds = ViewBounds {
            line_count: 1,
            viewport_height: 5,
            widest_line: 19,
            viewport_width: 58,
        };
        let lines = vec![String::from("  Needle and needle")];
        state.apply(Action::EnterSearch, bounds, &lines);
        for character in "needle".chars() {
            state.apply(Action::SearchCharacter(character), bounds, &lines);
        }
        state.apply(Action::SubmitSearch, bounds, &lines);

        terminal
            .draw(|frame| {
                render_viewer(frame, "example", &document, &mut state);
            })
            .expect("search results should render");

        let buffer = terminal.backend().buffer();
        let contents: String = buffer.content.iter().map(|cell| cell.symbol()).collect();
        assert!(contents.contains("/needle  1/2"));
        assert!(buffer.content.iter().any(|cell| cell.bg == ROSE));
    }

    #[test]
    fn escape_clears_an_active_search_before_quitting() {
        let mut state = ViewerState::default();
        state.search.query = String::from("needle");

        assert_eq!(
            action_for_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &state),
            Some(Action::ClearSearch)
        );

        state.search.query.clear();
        assert_eq!(
            action_for_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &state),
            Some(Action::Quit)
        );
    }

    #[test]
    fn search_mode_maps_text_editing_and_submission_keys() {
        let state = ViewerState {
            mode: crate::app::ViewerMode::SearchInput,
            ..ViewerState::default()
        };

        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Char('é'), KeyModifiers::NONE),
                &state
            ),
            Some(Action::SearchCharacter('é'))
        );
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
                &state
            ),
            Some(Action::SearchBackspace)
        );
        assert_eq!(
            action_for_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), &state),
            Some(Action::SubmitSearch)
        );
        assert_eq!(
            action_for_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &state),
            Some(Action::CancelSearch)
        );
    }

    #[test]
    fn test_backend_renders_help_overlay_on_tiny_terminal() {
        let backend = TestBackend::new(24, 8);
        let mut terminal = Terminal::new(backend).expect("test terminal should be created");
        let document = parse_markdown("text\n");
        let mut state = ViewerState {
            show_help: true,
            ..ViewerState::default()
        };

        terminal
            .draw(|frame| {
                render_viewer(frame, "example", &document, &mut state);
            })
            .expect("help should render on a tiny terminal");

        let contents: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(contents.contains("Help"));
    }
}
