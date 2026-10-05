use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::document::{Block, Document, Span, TextStyle};

enum BlockBuilder {
    Heading {
        level: u8,
        spans: Vec<Span>,
    },
    Paragraph {
        spans: Vec<Span>,
    },
    CodeBlock {
        language: Option<String>,
        text: String,
    },
}

impl BlockBuilder {
    fn spans_mut(&mut self) -> Option<&mut Vec<Span>> {
        match self {
            Self::Heading { spans, .. } | Self::Paragraph { spans } => Some(spans),
            Self::CodeBlock { .. } => None,
        }
    }

    fn push_text(&mut self, text: String, style: TextStyle) {
        match self {
            Self::Heading { spans, .. } | Self::Paragraph { spans } => {
                spans.push(Span { text, style });
            }
            Self::CodeBlock {
                text: code_text, ..
            } => code_text.push_str(&text),
        }
    }
}

#[derive(Default)]
struct InlineStyleState {
    emphasis_depth: usize,
    strong_depth: usize,
}

impl InlineStyleState {
    fn text_style(&self, inline_code: bool) -> TextStyle {
        TextStyle {
            emphasis: self.emphasis_depth > 0,
            strong: self.strong_depth > 0,
            inline_code,
        }
    }
}

pub fn parse_markdown(source: &str) -> Document {
    let mut document = Document::default();
    let mut current_block = None;
    let mut inline_style = InlineStyleState::default();

    for event in Parser::new(source) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_block = Some(BlockBuilder::Heading {
                    level: heading_level(level),
                    spans: Vec::new(),
                });
            }
            Event::Start(Tag::Paragraph) => {
                current_block = Some(BlockBuilder::Paragraph { spans: Vec::new() });
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let language = match kind {
                    CodeBlockKind::Fenced(info) => info
                        .split_whitespace()
                        .next()
                        .filter(|language| !language.is_empty())
                        .map(str::to_owned),
                    CodeBlockKind::Indented => None,
                };
                current_block = Some(BlockBuilder::CodeBlock {
                    language,
                    text: String::new(),
                });
            }
            Event::Start(Tag::Emphasis) => {
                inline_style.emphasis_depth += 1;
            }
            Event::Start(Tag::Strong) => {
                inline_style.strong_depth += 1;
            }
            Event::Text(text) => {
                if let Some(block) = current_block.as_mut() {
                    block.push_text(text.into_string(), inline_style.text_style(false));
                }
            }
            Event::Code(code) => {
                push_span(
                    &mut current_block,
                    code.into_string(),
                    inline_style.text_style(true),
                );
            }
            Event::End(TagEnd::Emphasis) => {
                inline_style.emphasis_depth = inline_style.emphasis_depth.saturating_sub(1);
            }
            Event::End(TagEnd::Strong) => {
                inline_style.strong_depth = inline_style.strong_depth.saturating_sub(1);
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(BlockBuilder::Heading { level, spans }) = current_block.take() {
                    document.blocks.push(Block::Heading { level, spans });
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if let Some(BlockBuilder::Paragraph { spans }) = current_block.take() {
                    document.blocks.push(Block::Paragraph { spans });
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(BlockBuilder::CodeBlock { language, text }) = current_block.take() {
                    document.blocks.push(Block::CodeBlock { language, text });
                }
            }
            Event::Rule => document.blocks.push(Block::ThematicBreak),
            _ => {}
        }
    }

    document
}

fn push_span(block: &mut Option<BlockBuilder>, text: String, style: TextStyle) {
    if let Some(spans) = block.as_mut().and_then(BlockBuilder::spans_mut) {
        spans.push(Span { text, style });
    }
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}
