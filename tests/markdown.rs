use mman::{
    document::{Block, Document, Link, ListItem, Span, TextStyle},
    markdown::parse_markdown,
};

fn plain(text: &str) -> Span {
    styled(text, false, false, false)
}

fn styled(text: &str, emphasis: bool, strong: bool, inline_code: bool) -> Span {
    Span {
        text: text.to_owned(),
        style: TextStyle {
            emphasis,
            strong,
            inline_code,
        },
        link: None,
    }
}

fn linked(text: &str, destination: &str, title: Option<&str>, style: TextStyle) -> Span {
    Span {
        text: text.to_owned(),
        style,
        link: Some(Link {
            destination: destination.to_owned(),
            title: title.map(str::to_owned),
        }),
    }
}

#[test]
fn parses_heading_and_paragraph_into_owned_blocks() {
    let document = parse_markdown("# Ownership\n\nRust moves values.\n");

    assert_eq!(
        document,
        Document {
            blocks: vec![
                Block::Heading {
                    level: 1,
                    spans: vec![plain("Ownership")],
                },
                Block::Paragraph {
                    spans: vec![plain("Rust moves values.")],
                },
            ],
        }
    );
}

#[test]
fn parses_inline_emphasis_strong_and_code_styles() {
    let document = parse_markdown("Use *borrowed*, **owned**, and `clone` values.\n");

    assert_eq!(
        document.blocks,
        vec![Block::Paragraph {
            spans: vec![
                plain("Use "),
                styled("borrowed", true, false, false),
                plain(", "),
                styled("owned", false, true, false),
                plain(", and "),
                styled("clone", false, false, true),
                plain(" values."),
            ],
        }]
    );
}

#[test]
fn combines_nested_inline_styles() {
    let document = parse_markdown("This is ***important***.\n");

    assert_eq!(
        document.blocks,
        vec![Block::Paragraph {
            spans: vec![
                plain("This is "),
                styled("important", true, true, false),
                plain("."),
            ],
        }]
    );
}

#[test]
fn converts_soft_line_break_to_space() {
    let document = parse_markdown("first line\nsecond line\n");

    assert_eq!(
        document.blocks,
        vec![Block::Paragraph {
            spans: vec![plain("first line second line")],
        }]
    );
}

#[test]
fn preserves_hard_line_break_in_paragraph() {
    let document = parse_markdown("first line  \nsecond line\n");

    assert_eq!(
        document.blocks,
        vec![Block::Paragraph {
            spans: vec![plain("first line\nsecond line")],
        }]
    );
}

#[test]
fn sanitizes_terminal_control_characters_in_prose() {
    let document = parse_markdown("safe \u{1b}[31mred\u{7}\n");

    assert_eq!(
        document.blocks,
        vec![Block::Paragraph {
            spans: vec![plain("safe �[31mred�")],
        }]
    );
}

#[test]
fn sanitizes_code_controls_but_preserves_layout_characters() {
    let document = parse_markdown("```text\n\tbefore\u{1b}after\n```\n");

    assert_eq!(
        document.blocks,
        vec![Block::CodeBlock {
            language: Some(String::from("text")),
            text: String::from("\tbefore�after\n"),
        }]
    );
}

#[test]
fn parses_link_destination_and_title() {
    let document =
        parse_markdown("See [ownership](../concepts/ownership.md \"Ownership guide\").\n");

    assert_eq!(
        document.blocks,
        vec![Block::Paragraph {
            spans: vec![
                plain("See "),
                linked(
                    "ownership",
                    "../concepts/ownership.md",
                    Some("Ownership guide"),
                    TextStyle::default(),
                ),
                plain("."),
            ],
        }]
    );
}

#[test]
fn link_preserves_nested_text_style() {
    let document = parse_markdown("Read [**Rust**](https://www.rust-lang.org/).\n");

    assert_eq!(
        document.blocks,
        vec![Block::Paragraph {
            spans: vec![
                plain("Read "),
                linked(
                    "Rust",
                    "https://www.rust-lang.org/",
                    None,
                    TextStyle {
                        strong: true,
                        ..TextStyle::default()
                    },
                ),
                plain("."),
            ],
        }]
    );
}

#[test]
fn omits_inline_html_tags_but_keeps_readable_text() {
    let document = parse_markdown("before <b>bold</b> after\n");

    assert_eq!(
        document.blocks,
        vec![Block::Paragraph {
            spans: vec![plain("before bold after")],
        }]
    );
}

#[test]
fn omits_raw_html_blocks() {
    let document = parse_markdown("<script>danger()</script>\n\nvisible\n");

    assert_eq!(
        document.blocks,
        vec![Block::Paragraph {
            spans: vec![plain("visible")],
        }]
    );
}

#[test]
fn renders_image_alt_text_without_fetching_the_image() {
    let document = parse_markdown("See ![Ferris](https://example.com/ferris.png).\n");

    assert_eq!(
        document.blocks,
        vec![Block::Paragraph {
            spans: vec![plain("See Ferris.")],
        }]
    );
}

#[test]
fn parses_block_quote_with_nested_blocks() {
    let document = parse_markdown("> **Borrowing** avoids moving a value.\n");

    assert_eq!(
        document.blocks,
        vec![Block::BlockQuote {
            blocks: vec![Block::Paragraph {
                spans: vec![
                    styled("Borrowing", false, true, false),
                    plain(" avoids moving a value."),
                ],
            }],
        }]
    );
}

#[test]
fn parses_unordered_list_items() {
    let document = parse_markdown("- ownership\n- borrowing\n");

    assert_eq!(
        document.blocks,
        vec![Block::List {
            start: None,
            items: vec![
                ListItem {
                    blocks: vec![Block::Paragraph {
                        spans: vec![plain("ownership")],
                    }],
                },
                ListItem {
                    blocks: vec![Block::Paragraph {
                        spans: vec![plain("borrowing")],
                    }],
                },
            ],
        }]
    );
}

#[test]
fn parses_ordered_list_start_and_nested_list() {
    let document = parse_markdown("3. first\n4. second\n   - nested\n");

    assert_eq!(
        document.blocks,
        vec![Block::List {
            start: Some(3),
            items: vec![
                ListItem {
                    blocks: vec![Block::Paragraph {
                        spans: vec![plain("first")],
                    }],
                },
                ListItem {
                    blocks: vec![
                        Block::Paragraph {
                            spans: vec![plain("second")],
                        },
                        Block::List {
                            start: None,
                            items: vec![ListItem {
                                blocks: vec![Block::Paragraph {
                                    spans: vec![plain("nested")],
                                }],
                            }],
                        },
                    ],
                },
            ],
        }]
    );
}

#[test]
fn parses_fenced_code_block_and_preserves_whitespace() {
    let document = parse_markdown("```rust\nfn main() {\n    println!(\"hi\");\n}\n```\n");

    assert_eq!(
        document.blocks,
        vec![Block::CodeBlock {
            language: Some(String::from("rust")),
            text: String::from("fn main() {\n    println!(\"hi\");\n}\n"),
        }]
    );
}

#[test]
fn parses_indented_code_block_without_language() {
    let document = parse_markdown("    first line\n    second line\n");

    assert_eq!(
        document.blocks,
        vec![Block::CodeBlock {
            language: None,
            text: String::from("first line\nsecond line\n"),
        }]
    );
}

#[test]
fn parses_thematic_break() {
    let document = parse_markdown("before\n\n---\n\nafter\n");

    assert_eq!(
        document.blocks,
        vec![
            Block::Paragraph {
                spans: vec![plain("before")],
            },
            Block::ThematicBreak,
            Block::Paragraph {
                spans: vec![plain("after")],
            },
        ]
    );
}

#[test]
fn parsed_document_outlives_source_markdown() {
    let document = {
        let source = String::from("## Borrowing\n\nA borrow refers to a value.\n");
        parse_markdown(&source)
    };

    assert_eq!(
        document.blocks,
        vec![
            Block::Heading {
                level: 2,
                spans: vec![plain("Borrowing")],
            },
            Block::Paragraph {
                spans: vec![plain("A borrow refers to a value.")],
            },
        ]
    );
}
