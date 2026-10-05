use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    document::{Block, Document, Link, Span, TextStyle},
    syntax::{SyntaxStyle, highlight_code},
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Layout {
    pub lines: Vec<Line>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub spans: Vec<RenderedSpan>,
    pub kind: LineKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedSpan {
    pub text: String,
    pub style: TextStyle,
    pub link: Option<Link>,
    pub syntax: Option<SyntaxStyle>,
}

impl Line {
    pub fn plain_text(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Prose,
    Heading(u8),
    Code,
    CodeBorder,
    ThematicBreak,
}

#[derive(Clone)]
struct StyledCharacter {
    character: char,
    style: TextStyle,
    link: Option<Link>,
}

enum Token {
    Word(Vec<StyledCharacter>),
    Space(StyledCharacter),
    Break,
}

pub fn layout_document(document: &Document, width: usize) -> Layout {
    let mut lines = Vec::new();

    for (index, block) in document.blocks.iter().enumerate() {
        if index > 0 && !lines.is_empty() {
            lines.push(empty_line());
        }
        let indent = if matches!(block, Block::Heading { .. }) {
            String::new()
        } else {
            String::from("  ")
        };
        layout_block(block, width, indent.clone(), indent, &mut lines);
    }

    Layout { lines }
}

fn layout_block(
    block: &Block,
    width: usize,
    first_prefix: String,
    continuation_prefix: String,
    lines: &mut Vec<Line>,
) {
    match block {
        Block::Paragraph { spans } => layout_spans(
            spans,
            width,
            LineKind::Prose,
            &first_prefix,
            &continuation_prefix,
            lines,
        ),
        Block::Heading { level, spans } => layout_spans(
            spans,
            width,
            LineKind::Heading(*level),
            &first_prefix,
            &continuation_prefix,
            lines,
        ),
        Block::CodeBlock { language, text } => {
            layout_code_block(
                text,
                language.as_deref(),
                width,
                &first_prefix,
                &continuation_prefix,
                lines,
            );
        }
        Block::BlockQuote { blocks } => {
            let quote_first = format!("{first_prefix}│ ");
            let quote_continuation = format!("{continuation_prefix}│ ");
            for child in blocks {
                layout_block(
                    child,
                    width,
                    quote_first.clone(),
                    quote_continuation.clone(),
                    lines,
                );
            }
        }
        Block::List { start, items } => {
            for (item_index, item) in items.iter().enumerate() {
                let marker = match start {
                    Some(start) => format!("{}. ", start + item_index as u64),
                    None => String::from("• "),
                };
                let base = if item_index == 0 {
                    &first_prefix
                } else {
                    &continuation_prefix
                };
                let item_first = format!("{base}{marker}");
                let item_continuation = format!(
                    "{}{}",
                    continuation_prefix,
                    " ".repeat(UnicodeWidthStr::width(marker.as_str()))
                );

                for (block_index, child) in item.blocks.iter().enumerate() {
                    let child_first = if block_index == 0 {
                        item_first.clone()
                    } else {
                        item_continuation.clone()
                    };
                    layout_block(child, width, child_first, item_continuation.clone(), lines);
                }
            }
        }
        Block::ThematicBreak => {
            let available = available_width(width, &first_prefix);
            let mut line = Line {
                spans: Vec::new(),
                kind: LineKind::ThematicBreak,
            };
            push_prefix(&mut line, &first_prefix);
            line.spans.push(RenderedSpan {
                text: "─".repeat(available),
                style: TextStyle::default(),
                link: None,
                syntax: None,
            });
            lines.push(line);
        }
    }
}

fn layout_code_block(
    text: &str,
    language: Option<&str>,
    width: usize,
    first_prefix: &str,
    continuation_prefix: &str,
    lines: &mut Vec<Line>,
) {
    let available = available_width(width, continuation_prefix);
    let content_width = available.saturating_sub(4).max(1);
    push_code_border(lines, first_prefix, content_width, true);

    if let Some(highlighted_lines) = highlight_code(text, language) {
        if highlighted_lines.is_empty() {
            push_code_line(lines, first_prefix, Vec::new(), content_width);
        } else {
            for (index, highlighted) in highlighted_lines.into_iter().enumerate() {
                let spans = highlighted
                    .into_iter()
                    .map(|span| RenderedSpan {
                        text: span.text,
                        style: TextStyle {
                            inline_code: true,
                            ..TextStyle::default()
                        },
                        link: None,
                        syntax: Some(span.style),
                    })
                    .collect();
                push_code_line(
                    lines,
                    if index == 0 {
                        first_prefix
                    } else {
                        continuation_prefix
                    },
                    spans,
                    content_width,
                );
            }
        }
    } else {
        let code_lines: Vec<&str> = if text.is_empty() {
            vec![""]
        } else {
            text.split_terminator('\n').collect()
        };

        for (index, code_line) in code_lines.into_iter().enumerate() {
            push_code_line(
                lines,
                if index == 0 {
                    first_prefix
                } else {
                    continuation_prefix
                },
                vec![RenderedSpan {
                    text: code_line.to_owned(),
                    style: TextStyle {
                        inline_code: true,
                        ..TextStyle::default()
                    },
                    link: None,
                    syntax: None,
                }],
                content_width,
            );
        }
    }

    push_code_border(lines, continuation_prefix, content_width, false);
}

fn push_code_line(
    lines: &mut Vec<Line>,
    prefix: &str,
    mut spans: Vec<RenderedSpan>,
    content_width: usize,
) {
    let text_width: usize = spans
        .iter()
        .map(|span| UnicodeWidthStr::width(span.text.as_str()))
        .sum();
    let padding = content_width.saturating_sub(text_width);

    spans.insert(0, plain_rendered_span(format!("{prefix}│ ")));
    spans.push(plain_rendered_span(format!("{} │", " ".repeat(padding))));
    lines.push(Line {
        spans,
        kind: LineKind::Code,
    });
}

fn push_code_border(lines: &mut Vec<Line>, prefix: &str, content_width: usize, top: bool) {
    let (left, right) = if top { ('╭', '╮') } else { ('╰', '╯') };
    lines.push(Line {
        spans: vec![plain_rendered_span(format!(
            "{prefix}{left}{}{right}",
            "─".repeat(content_width + 2)
        ))],
        kind: LineKind::CodeBorder,
    });
}

fn plain_rendered_span(text: String) -> RenderedSpan {
    RenderedSpan {
        text,
        style: TextStyle::default(),
        link: None,
        syntax: None,
    }
}

fn layout_spans(
    spans: &[Span],
    width: usize,
    kind: LineKind,
    first_prefix: &str,
    continuation_prefix: &str,
    lines: &mut Vec<Line>,
) {
    let available = available_width(width, continuation_prefix);
    let mut wrapped = wrap_spans(spans, available, kind);

    for (index, line) in wrapped.iter_mut().enumerate() {
        let prefix = if index == 0 {
            first_prefix
        } else {
            continuation_prefix
        };
        push_prefix(line, prefix);
    }

    lines.extend(wrapped);
}

fn wrap_spans(spans: &[Span], width: usize, kind: LineKind) -> Vec<Line> {
    let tokens = tokenize(spans);
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut characters = Vec::new();
    let mut line_width = 0;
    let mut pending_space = None;

    for token in tokens {
        match token {
            Token::Space(space) => {
                if !characters.is_empty() {
                    pending_space = Some(space);
                }
            }
            Token::Break => {
                flush_line(&mut lines, &mut characters, kind, true);
                line_width = 0;
                pending_space = None;
            }
            Token::Word(word) => {
                let word_width = display_width(&word);
                let separator_width = usize::from(pending_space.is_some());

                if !characters.is_empty() && line_width + separator_width + word_width > width {
                    flush_line(&mut lines, &mut characters, kind, false);
                    line_width = 0;
                    pending_space = None;
                }

                if let Some(space) = pending_space.take()
                    && !characters.is_empty()
                {
                    characters.push(space);
                    line_width += 1;
                }

                for character in word {
                    let character_width = terminal_width(character.character);

                    if !characters.is_empty() && line_width + character_width > width {
                        flush_line(&mut lines, &mut characters, kind, false);
                        line_width = 0;
                    }

                    line_width += character_width;
                    characters.push(character);
                }
            }
        }
    }

    flush_line(&mut lines, &mut characters, kind, false);
    lines
}

fn tokenize(spans: &[Span]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut word = Vec::new();
    let mut previous_was_space = false;

    for span in spans {
        for character in span.text.chars() {
            let styled = StyledCharacter {
                character,
                style: span.style,
                link: span.link.clone(),
            };

            if character == '\n' {
                push_word(&mut tokens, &mut word);
                tokens.push(Token::Break);
                previous_was_space = false;
            } else if character.is_whitespace() {
                push_word(&mut tokens, &mut word);
                if !previous_was_space {
                    tokens.push(Token::Space(StyledCharacter {
                        character: ' ',
                        ..styled
                    }));
                    previous_was_space = true;
                }
            } else {
                word.push(styled);
                previous_was_space = false;
            }
        }
    }

    push_word(&mut tokens, &mut word);
    tokens
}

fn push_word(tokens: &mut Vec<Token>, word: &mut Vec<StyledCharacter>) {
    if !word.is_empty() {
        tokens.push(Token::Word(std::mem::take(word)));
    }
}

fn display_width(characters: &[StyledCharacter]) -> usize {
    characters
        .iter()
        .map(|character| terminal_width(character.character))
        .sum()
}

fn terminal_width(character: char) -> usize {
    UnicodeWidthChar::width(character).unwrap_or(0)
}

fn available_width(width: usize, prefix: &str) -> usize {
    width.saturating_sub(UnicodeWidthStr::width(prefix)).max(1)
}

fn flush_line(
    lines: &mut Vec<Line>,
    characters: &mut Vec<StyledCharacter>,
    kind: LineKind,
    force: bool,
) {
    if characters.is_empty() && !force {
        return;
    }

    let mut spans = Vec::new();
    for character in characters.drain(..) {
        append_character(&mut spans, character);
    }

    lines.push(Line { spans, kind });
}

fn append_character(spans: &mut Vec<RenderedSpan>, character: StyledCharacter) {
    if let Some(previous) = spans.last_mut()
        && previous.style == character.style
        && previous.link == character.link
        && previous.syntax.is_none()
    {
        previous.text.push(character.character);
        return;
    }

    spans.push(RenderedSpan {
        text: character.character.to_string(),
        style: character.style,
        link: character.link,
        syntax: None,
    });
}

fn push_prefix(line: &mut Line, prefix: &str) {
    if prefix.is_empty() {
        return;
    }

    line.spans.insert(
        0,
        RenderedSpan {
            text: prefix.to_owned(),
            style: TextStyle::default(),
            link: None,
            syntax: None,
        },
    );
}

fn empty_line() -> Line {
    Line {
        spans: Vec::new(),
        kind: LineKind::Prose,
    }
}
