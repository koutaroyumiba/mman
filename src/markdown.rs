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

pub fn parse_markdown(source: &str) -> Document {
    let mut document = Document::default();
    let mut current_block = None;

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
            Event::Text(text) => {
                if let Some(block) = current_block.as_mut() {
                    block.spans_mut().push(Span {
                        text: text.into_string(),
                        style: TextStyle::default(),
                    });
                }
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
