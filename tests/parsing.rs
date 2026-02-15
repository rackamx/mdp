use mdp::parsing::{parse_markdown, Event};

#[test]
fn test_parse_paragraph() {
    let markdown = "Hello World";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    // Should contain a Text event with "Hello World"
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Text(s) if s.contains("Hello World"))),
        "Expected Text event containing 'Hello World', got: {:?}",
        events
    );
}

#[test]
fn test_parse_heading() {
    let markdown = "# Heading";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    // Should contain a Start(Heading) event
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::Start(mdp::parsing::Block::Heading { level: 1, .. })
        )),
        "Expected Start(Heading) event, got: {:?}",
        events
    );
}

#[test]
fn test_parse_multiple_blocks() {
    let markdown = "# Title\n\nThis is a paragraph.";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    // Should have at least 2 events: heading and paragraph
    assert!(
        events.len() >= 2,
        "Expected at least 2 events, got: {:?}",
        events
    );

    // Should have a heading
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Start(mdp::parsing::Block::Heading { .. }))),
        "Expected Heading event, got: {:?}",
        events
    );

    // Should have a text event
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Text(s) if s.contains("paragraph"))),
        "Expected Text event containing 'paragraph', got: {:?}",
        events
    );
}

#[test]
fn test_parse_unclosed_fence() {
    let markdown = "```rust\nfn main() {\n";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    assert!(
        !events.is_empty(),
        "Parser should not panic on unclosed fence"
    );
}

#[test]
fn test_parse_mismatched_brackets() {
    let markdown = "[text";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    assert!(
        events.iter().any(|e| matches!(e, Event::Text(_))),
        "Parser should still emit text for mismatched brackets"
    );
}

#[test]
fn test_parse_incomplete_emphasis() {
    let markdown = "**incomplete";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Text(s) if s.contains("incomplete"))),
        "Parser should keep text for incomplete emphasis"
    );
}

#[test]
fn test_parse_raw_html_events() {
    let markdown = "<span>inline</span>\n\n<div>block</div>";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::RawHtml(s) if s.contains("<span>"))),
        "Expected inline HTML event, got: {:?}",
        events
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::RawHtml(s) if s.contains("<div>"))),
        "Expected block HTML event, got: {:?}",
        events
    );
}

#[test]
fn test_parse_thematic_break_marker_types() {
    let markdown = "---\n\n***\n\n___\n";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let markers: Vec<char> = events
        .iter()
        .filter_map(|e| match e {
            Event::Rule(ch) => Some(*ch),
            _ => None,
        })
        .collect();

    assert_eq!(markers, vec!['-', '*', '_']);
}

#[test]
fn test_parse_unordered_list_item_markers() {
    let markdown = "- dash\n* star\n+ plus\n";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let markers: Vec<char> = events
        .iter()
        .filter_map(|e| match e {
            Event::ListItemMarker(ch) => Some(*ch),
            _ => None,
        })
        .collect();
    assert_eq!(markers, vec!['-', '*', '+']);
}

#[test]
fn test_parse_unordered_list_item_markers_with_deep_nesting() {
    let markdown = "- root\n  - child\n    - grandchild\n* star\n+ plus\n";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let markers: Vec<char> = events
        .iter()
        .filter_map(|e| match e {
            Event::ListItemMarker(ch) => Some(*ch),
            _ => None,
        })
        .collect();
    assert_eq!(markers, vec!['-', '-', '-', '*', '+']);
}
