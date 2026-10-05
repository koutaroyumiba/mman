use mman::{
    layout::{LineKind, layout_document},
    markdown::parse_markdown,
};

#[test]
fn wraps_paragraph_at_word_boundaries() {
    let document = parse_markdown("alpha beta gamma\n");

    let layout = layout_document(&document, 10);
    let lines: Vec<String> = layout.lines.iter().map(|line| line.plain_text()).collect();

    assert_eq!(lines, vec!["alpha beta", "gamma"]);
    assert!(layout.lines.iter().all(|line| line.kind == LineKind::Prose));
}

#[test]
fn preserves_inline_style_across_wrapped_lines() {
    let document = parse_markdown("one *two three* four\n");

    let layout = layout_document(&document, 10);
    let lines: Vec<String> = layout.lines.iter().map(|line| line.plain_text()).collect();

    assert_eq!(lines, vec!["one two", "three four"]);
    assert!(
        layout.lines[0]
            .spans
            .iter()
            .any(|span| span.text == "two" && span.style.emphasis)
    );
    assert!(
        layout.lines[1]
            .spans
            .iter()
            .any(|span| span.text == "three" && span.style.emphasis)
    );
}

#[test]
fn wraps_using_unicode_display_width() {
    let document = parse_markdown("ab界cd\n");

    let layout = layout_document(&document, 4);
    let lines: Vec<String> = layout.lines.iter().map(|line| line.plain_text()).collect();

    assert_eq!(lines, vec!["ab界", "cd"]);
}

#[test]
fn explicit_hard_break_starts_a_new_rendered_line() {
    let document = parse_markdown("first  \nsecond\n");

    let layout = layout_document(&document, 80);
    let lines: Vec<String> = layout.lines.iter().map(|line| line.plain_text()).collect();

    assert_eq!(lines, vec!["first", "second"]);
}
