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

#[test]
fn test_parse_ordered_list_item_markers() {
    let markdown = "3. three\n8. eight\n";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let markers: Vec<u64> = events
        .iter()
        .filter_map(|e| match e {
            Event::OrderedListItemMarker(n) => Some(*n),
            _ => None,
        })
        .collect();
    assert_eq!(markers, vec![3, 8]);
}

#[test]
fn test_parse_reference_definition_line_as_reference_event() {
    let markdown = "[id]: https://example.com\n";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ReferenceDefinition(s) if s.contains("[id]: https://example.com"))),
        "Expected reference definition line preserved as reference event, got: {:?}",
        events
    );
}

#[test]
fn test_parse_consecutive_reference_definitions_as_distinct_events() {
    let markdown = "[id1]: https://example.com/one\n[id2]: https://example.com/two\n";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let defs = events
        .iter()
        .filter_map(|e| match e {
            Event::ReferenceDefinition(s) => Some(s),
            _ => None,
        })
        .count();

    assert_eq!(
        defs, 2,
        "Expected two reference definition events, got: {:?}",
        events
    );
}

#[test]
fn test_parse_emits_interblock_blank_line_count_for_multiple_blank_lines() {
    let markdown = "Text before.\n\n\nText after.";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::InterBlockBlankLines(2))),
        "Expected InterBlockBlankLines(2) event, got: {:?}",
        events
    );
}

#[test]
fn test_parse_marks_setext_heading_style() {
    let markdown = "Setext H1\n=========\n\nATX\n";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    assert!(
        events.iter().any(|e| matches!(e, Event::HeadingSetext(true))),
        "Expected HeadingSetext(true) event for setext heading, got: {:?}",
        events
    );
}

#[test]
fn test_parse_inline_code_with_inner_backticks_and_spaces() {
    let markdown = "Use `` `surrounded by spaces` `` style delimiters.";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let code_events: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            Event::InlineCode(code) => Some(code.clone()),
            _ => None,
        })
        .collect();
    assert!(
        code_events.iter().any(|c| c.contains("surrounded by spaces")),
        "Expected inline code event for spaced/backtick sample, got: {:?}",
        events
    );
}

#[test]
fn test_parse_link_source_range_is_preserved() {
    let markdown = "see [docs](https://example.com/docs) now";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let link = events.iter().find_map(|e| match e {
        Event::Link { source, .. } => Some(source.as_str()),
        _ => None,
    });
    assert_eq!(
        link,
        Some("[docs](https://example.com/docs)"),
        "Expected link source range to be preserved, got: {:?}",
        events
    );
}

#[test]
fn test_parse_escaped_image_like_link_preserves_literal_source() {
    let markdown = "\\![not image](x)";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let source = events.iter().find_map(|e| match e {
        Event::Link { source, .. } => Some(source.as_str()),
        _ => None,
    });
    assert_eq!(
        source,
        Some("![not image](x)"),
        "Expected escaped image-like link source to remain literal markdown form, got: {:?}",
        events
    );
}
