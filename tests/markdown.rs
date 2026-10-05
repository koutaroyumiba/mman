use mman::{
    document::{Block, Document, Span, TextStyle},
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
