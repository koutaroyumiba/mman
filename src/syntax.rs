use std::{str::FromStr, sync::OnceLock};

use syntect::{
    easy::HighlightLines,
    highlighting::{
        Color, FontStyle, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSettings,
    },
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxStyle {
    pub foreground: (u8, u8, u8),
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxSpan {
    pub text: String,
    pub style: SyntaxStyle,
}

pub fn highlight_code(source: &str, language: Option<&str>) -> Option<Vec<Vec<SyntaxSpan>>> {
    let language = language?.trim();
    if language.is_empty() {
        return None;
    }

    let syntaxes = syntax_set();
    let syntax = syntaxes.find_syntax_by_token(language)?;
    let mut highlighter = HighlightLines::new(syntax, rose_pine_theme());
    let mut lines = Vec::new();

    for line in LinesWithEndings::from(source) {
        let highlighted = highlighter.highlight_line(line, syntaxes).ok()?;
        let mut spans = Vec::new();

        for (style, text) in highlighted {
            let text = text.trim_end_matches(['\r', '\n']);
            if text.is_empty() {
                continue;
            }
            spans.push(SyntaxSpan {
                text: text.to_owned(),
                style: SyntaxStyle {
                    foreground: (style.foreground.r, style.foreground.g, style.foreground.b),
                    bold: style.font_style.contains(FontStyle::BOLD),
                    italic: style.font_style.contains(FontStyle::ITALIC),
                },
            });
        }

        lines.push(spans);
    }

    Some(lines)
}

fn syntax_set() -> &'static SyntaxSet {
    static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAXES.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn rose_pine_theme() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(|| Theme {
        name: Some(String::from("Rosé Pine")),
        author: Some(String::from("mman")),
        settings: ThemeSettings {
            foreground: Some(color(224, 222, 244)),
            background: Some(color(31, 29, 46)),
            ..ThemeSettings::default()
        },
        scopes: vec![
            theme_item("comment", color(110, 106, 134), FontStyle::ITALIC),
            theme_item("string", color(246, 193, 119), FontStyle::empty()),
            theme_item(
                "constant.numeric, constant.language",
                color(235, 188, 186),
                FontStyle::empty(),
            ),
            theme_item("keyword, storage", color(196, 167, 231), FontStyle::BOLD),
            theme_item(
                "entity.name.function, support.function",
                color(156, 207, 216),
                FontStyle::empty(),
            ),
            theme_item(
                "entity.name.type, entity.name.class, support.type",
                color(235, 188, 186),
                FontStyle::empty(),
            ),
            theme_item(
                "variable.parameter",
                color(224, 222, 244),
                FontStyle::ITALIC,
            ),
        ],
    })
}

fn theme_item(scope: &str, foreground: Color, font_style: FontStyle) -> ThemeItem {
    ThemeItem {
        scope: ScopeSelectors::from_str(scope).expect("static syntax scope selector is valid"),
        style: StyleModifier {
            foreground: Some(foreground),
            background: None,
            font_style: Some(font_style),
        },
    }
}

const fn color(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b, a: 255 }
}
