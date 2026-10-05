#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
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
    ThematicBreak,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub style: TextStyle,
    pub link: Option<Link>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub destination: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextStyle {
    pub emphasis: bool,
    pub strong: bool,
    pub inline_code: bool,
}
