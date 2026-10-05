use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::document::{Block, Document, Span, TextStyle};

enum BlockBuilder {
    Heading { level: u8, spans: Vec<Span> },
    Paragraph { spans: Vec<Span> },
}

impl BlockBuilder {
    fn spans_mut(&mut self) -> &mut Vec<Span> {
        match self {
            Self::Heading { spans, .. } | Self::Paragraph { spans } => spans,
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
            Event::Start(Tag::Emphasis) => {
                inline_style.emphasis_depth += 1;
            }
            Event::Start(Tag::Strong) => {
                inline_style.strong_depth += 1;
            }
            Event::Text(text) => {
                push_span(
                    &mut current_block,
                    text.into_string(),
                    inline_style.text_style(false),
                );
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
            _ => {}
        }
    }

    document
}

fn push_span(block: &mut Option<BlockBuilder>, text: String, style: TextStyle) {
    if let Some(block) = block.as_mut() {
        block.spans_mut().push(Span { text, style });
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
