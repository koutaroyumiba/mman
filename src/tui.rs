use std::io;

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout as RatatuiLayout},
    style::{Color, Modifier, Style},
    text::{Line as TuiLine, Span as TuiSpan},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    document::Span,
    layout::{LineKind, layout_document},
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
    let mut scroll = 0usize;

    loop {
        let mut maximum_scroll = 0usize;
        let mut page_step = 1usize;

        terminal.draw(|frame| {
            let regions = RatatuiLayout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(frame.area());
            let body = regions[0];
            let status = regions[1];
            let content_width = body.width.saturating_sub(2) as usize;
            let viewport_height = body.height.saturating_sub(2) as usize;
            let layout = layout_document(&document, content_width);

            maximum_scroll = layout.lines.len().saturating_sub(viewport_height);
            page_step = viewport_height.max(1);
            scroll = scroll.min(maximum_scroll);

            let visible_lines: Vec<TuiLine<'static>> = layout
                .lines
                .iter()
                .skip(scroll)
                .take(viewport_height)
                .map(to_tui_line)
                .collect();

            let title = format!(" mman · {topic} ");
            let page = Paragraph::new(visible_lines)
                .style(Style::default().fg(TEXT).bg(BASE))
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
                format!("{}/{}", scroll + 1, layout.lines.len())
            };
            let status_text = format!(" {position}  j/k scroll  q quit ");
            frame.render_widget(
                Paragraph::new(status_text).style(Style::default().fg(MUTED).bg(SURFACE)),
                status,
            );
        })?;

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key {
                KeyEvent {
                    code: KeyCode::Char('q') | KeyCode::Esc,
                    ..
                } => break,
                KeyEvent {
                    code: KeyCode::Char('j') | KeyCode::Down,
                    ..
                } => scroll = (scroll + 1).min(maximum_scroll),
                KeyEvent {
                    code: KeyCode::Char('k') | KeyCode::Up,
                    ..
                } => scroll = scroll.saturating_sub(1),
                KeyEvent {
                    code: KeyCode::PageDown,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => scroll = (scroll + page_step).min(maximum_scroll),
                KeyEvent {
                    code: KeyCode::PageUp,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Char('u'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => scroll = scroll.saturating_sub(page_step),
                KeyEvent {
                    code: KeyCode::Home | KeyCode::Char('g'),
                    ..
                } => scroll = 0,
                KeyEvent {
                    code: KeyCode::End | KeyCode::Char('G'),
                    ..
                } => scroll = maximum_scroll,
                _ => {}
            }
        }
    }

    Ok(())
}

fn to_tui_line(line: &crate::layout::Line) -> TuiLine<'static> {
    let spans = line
        .spans
        .iter()
        .map(|span| TuiSpan::styled(span.text.clone(), span_style(span, line.kind)))
        .collect::<Vec<_>>();
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
        LineKind::ThematicBreak => Style::default().fg(MUTED).bg(BASE),
    }
}

fn span_style(span: &Span, kind: LineKind) -> Style {
    let mut style = line_style(kind);

    if span.style.emphasis {
        style = style.fg(ROSE).add_modifier(Modifier::ITALIC);
    }
    if span.style.strong {
        style = style.fg(GOLD).add_modifier(Modifier::BOLD);
    }
    if span.style.inline_code {
        style = style.fg(FOAM).bg(OVERLAY);
    }
    if span.link.is_some() {
        style = style.fg(PINE).add_modifier(Modifier::UNDERLINED);
    }

    style
}
