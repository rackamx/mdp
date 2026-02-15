use mdless::parsing::{parse_markdown, Event};

#[test]
fn test_parse_paragraph() {
    let markdown = "Hello World";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    // Should contain a Text event with "Hello World"
    assert!(events.iter().any(|e| matches!(e, Event::Text(s) if s.contains("Hello World"))),
            "Expected Text event containing 'Hello World', got: {:?}", events);
}

#[test]
fn test_parse_heading() {
    let markdown = "# Heading";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    // Should contain a Start(Heading) event
    assert!(events.iter().any(|e| matches!(e, Event::Start(mdless::parsing::Block::Heading { level: 1, .. }))),
            "Expected Start(Heading) event, got: {:?}", events);
}

#[test]
fn test_parse_multiple_blocks() {
    let markdown = "# Title\n\nThis is a paragraph.";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    // Should have at least 2 events: heading and paragraph
    assert!(events.len() >= 2, "Expected at least 2 events, got: {:?}", events);

    // Should have a heading
    assert!(events.iter().any(|e| matches!(e, Event::Start(mdless::parsing::Block::Heading { .. }))),
            "Expected Heading event, got: {:?}", events);

    // Should have a text event
    assert!(events.iter().any(|e| matches!(e, Event::Text(s) if s.contains("paragraph"))),
            "Expected Text event containing 'paragraph', got: {:?}", events);
}
