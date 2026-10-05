use unicode_width::UnicodeWidthChar;

use crate::document::{Block, Document, Link, Span, TextStyle};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Layout {
    pub lines: Vec<Line>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub spans: Vec<Span>,
    pub kind: LineKind,
}

impl Line {
    pub fn plain_text(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Prose,
    Code,
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

    for block in &document.blocks {
        if let Block::Paragraph { spans } = block {
            layout_paragraph(spans, width, &mut lines);
        }
    }

    Layout { lines }
}

fn layout_paragraph(spans: &[Span], width: usize, lines: &mut Vec<Line>) {
    let tokens = tokenize(spans);
    let width = width.max(1);
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
                flush_line(lines, &mut characters, true);
                line_width = 0;
                pending_space = None;
            }
            Token::Word(word) => {
                let word_width = display_width(&word);
                let separator_width = usize::from(pending_space.is_some());

                if !characters.is_empty() && line_width + separator_width + word_width > width {
                    flush_line(lines, &mut characters, false);
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
                        flush_line(lines, &mut characters, false);
                        line_width = 0;
                    }

                    line_width += character_width;
                    characters.push(character);
                }
            }
        }
    }

    flush_line(lines, &mut characters, false);
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

fn flush_line(lines: &mut Vec<Line>, characters: &mut Vec<StyledCharacter>, force: bool) {
    if characters.is_empty() && !force {
        return;
    }

    let mut spans = Vec::new();
    for character in characters.drain(..) {
        append_character(&mut spans, character);
    }

    lines.push(Line {
        spans,
        kind: LineKind::Prose,
    });
}

fn append_character(spans: &mut Vec<Span>, character: StyledCharacter) {
    if let Some(previous) = spans.last_mut()
        && previous.style == character.style
        && previous.link == character.link
    {
        previous.text.push(character.character);
        return;
    }

    spans.push(Span {
        text: character.character.to_string(),
        style: character.style,
        link: character.link,
    });
}
