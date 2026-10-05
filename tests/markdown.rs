use mman::{
    document::{Block, Document, Span, TextStyle},
    markdown::parse_markdown,
};

fn plain(text: &str) -> Span {
    Span {
        text: text.to_owned(),
        style: TextStyle::default(),
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
