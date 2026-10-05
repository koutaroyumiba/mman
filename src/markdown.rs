use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::document::{Block, Document, Link, ListItem, Span, TextStyle};

enum Frame {
    Root {
        blocks: Vec<Block>,
    },
    BlockQuote {
        blocks: Vec<Block>,
    },
    List {
        start: Option<u64>,
        items: Vec<ListItem>,
    },
    Item {
        blocks: Vec<Block>,
    },
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

#[derive(Default)]
struct InlineStyleState {
    emphasis_depth: usize,
    strong_depth: usize,
    link: Option<Link>,
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
    let mut frames = vec![Frame::Root { blocks: Vec::new() }];
    let mut inline_style = InlineStyleState::default();

    for event in Parser::new(source) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => frames.push(Frame::Heading {
                level: heading_level(level),
                spans: Vec::new(),
            }),
            Event::Start(Tag::Paragraph) => {
                frames.push(Frame::Paragraph { spans: Vec::new() });
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let language = match kind {
                    CodeBlockKind::Fenced(info) => info
                        .split_whitespace()
                        .next()
                        .filter(|language| !language.is_empty())
                        .map(sanitize_text),
                    CodeBlockKind::Indented => None,
                };
                frames.push(Frame::CodeBlock {
                    language,
                    text: String::new(),
                });
            }
            Event::Start(Tag::BlockQuote(_)) => {
                frames.push(Frame::BlockQuote { blocks: Vec::new() });
            }
            Event::Start(Tag::List(start)) => {
                frames.push(Frame::List {
                    start,
                    items: Vec::new(),
                });
            }
            Event::Start(Tag::Item) => {
                frames.push(Frame::Item { blocks: Vec::new() });
            }
            Event::Start(Tag::Link {
                dest_url, title, ..
            }) => {
                inline_style.link = Some(Link {
                    destination: sanitize_text(&dest_url),
                    title: (!title.is_empty()).then(|| sanitize_text(&title)),
                });
            }
            Event::Start(Tag::Emphasis) => inline_style.emphasis_depth += 1,
            Event::Start(Tag::Strong) => inline_style.strong_depth += 1,
            Event::Text(text) => push_text(
                &mut frames,
                text.into_string(),
                inline_style.text_style(false),
                inline_style.link.clone(),
            ),
            Event::Code(code) => push_inline_span(
                &mut frames,
                code.into_string(),
                inline_style.text_style(true),
                inline_style.link.clone(),
            ),
            Event::SoftBreak => push_inline_span(
                &mut frames,
                String::from(" "),
                inline_style.text_style(false),
                inline_style.link.clone(),
            ),
            Event::HardBreak => push_inline_span(
                &mut frames,
                String::from("\n"),
                inline_style.text_style(false),
                inline_style.link.clone(),
            ),
            Event::End(TagEnd::Emphasis) => {
                inline_style.emphasis_depth = inline_style.emphasis_depth.saturating_sub(1);
            }
            Event::End(TagEnd::Strong) => {
                inline_style.strong_depth = inline_style.strong_depth.saturating_sub(1);
            }
            Event::End(TagEnd::Link) => inline_style.link = None,
            Event::End(TagEnd::Heading(_)) => {
                if let Some(Frame::Heading { level, spans }) = frames.pop() {
                    push_block(&mut frames, Block::Heading { level, spans });
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if let Some(Frame::Paragraph { spans }) = frames.pop() {
                    push_block(&mut frames, Block::Paragraph { spans });
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(Frame::CodeBlock { language, text }) = frames.pop() {
                    push_block(&mut frames, Block::CodeBlock { language, text });
                }
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                if let Some(Frame::BlockQuote { blocks }) = frames.pop() {
                    push_block(&mut frames, Block::BlockQuote { blocks });
                }
            }
            Event::End(TagEnd::Item) => {
                if let Some(Frame::Item { blocks }) = frames.pop()
                    && let Some(Frame::List { items, .. }) = frames.last_mut()
                {
                    items.push(ListItem { blocks });
                }
            }
            Event::End(TagEnd::List(_)) => {
                if let Some(Frame::List { start, items }) = frames.pop() {
                    push_block(&mut frames, Block::List { start, items });
                }
            }
            Event::Rule => push_block(&mut frames, Block::ThematicBreak),
            _ => {}
        }
    }

    match frames.pop() {
        Some(Frame::Root { blocks }) => Document { blocks },
        _ => Document::default(),
    }
}

fn push_block(frames: &mut [Frame], block: Block) {
    match frames.last_mut() {
        Some(Frame::Root { blocks })
        | Some(Frame::BlockQuote { blocks })
        | Some(Frame::Item { blocks }) => blocks.push(block),
        _ => {}
    }
}

fn push_text(frames: &mut [Frame], text: String, style: TextStyle, link: Option<Link>) {
    let text = sanitize_text(&text);

    if let Some(Frame::CodeBlock {
        text: code_text, ..
    }) = frames.last_mut()
    {
        code_text.push_str(&text);
    } else {
        push_inline_span(frames, text, style, link);
    }
}

fn push_inline_span(frames: &mut [Frame], text: String, style: TextStyle, link: Option<Link>) {
    let text = sanitize_text(&text);

    match frames.last_mut() {
        Some(Frame::Heading { spans, .. }) | Some(Frame::Paragraph { spans }) => {
            append_span(spans, text, style, link);
        }
        Some(Frame::Item { blocks }) => {
            if let Some(Block::Paragraph { spans }) = blocks.last_mut() {
                append_span(spans, text, style, link);
            } else {
                blocks.push(Block::Paragraph {
                    spans: vec![Span { text, style, link }],
                });
            }
        }
        _ => {}
    }
}

fn append_span(spans: &mut Vec<Span>, text: String, style: TextStyle, link: Option<Link>) {
    if let Some(previous) = spans.last_mut()
        && previous.style == style
        && previous.link == link
    {
        previous.text.push_str(&text);
        return;
    }

    spans.push(Span { text, style, link });
}

fn sanitize_text(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character == '\n' || character == '\t' || !character.is_control() {
                character
            } else {
                '�'
            }
        })
        .collect()
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
