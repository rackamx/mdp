use mdless::parsing::{parse_markdown, Event};
use mdless::rendering::Renderer;

/// Test basic plain text rendering
#[test]
fn test_render_plain_text() {
    let markdown = "Hello world";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain "Hello world"
    assert!(output.contains("Hello world"),
            "Expected output to contain 'Hello world', got: {:?}", output);
}

/// Test text wrapping at 80 characters
#[test]
fn test_render_text_with_wrapping() {
    // Create a long line that exceeds 80 characters
    let long_text = "This is a very long line that should be wrapped at eighty characters when rendered to fit within the terminal width constraints.";
    let events: Vec<Event> = parse_markdown(long_text).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Find the longest line in the output
    let lines: Vec<&str> = output.lines().collect();
    let max_line_length = lines.iter().map(|l| str::len(l)).max().unwrap_or(0);

    // All lines should be at most 80 characters
    assert!(max_line_length <= 80,
            "Expected max line length <= 80, got: {}. Lines: {:?}", max_line_length, lines);
}

/// Test rendering multiple paragraphs
#[test]
fn test_render_multiple_lines() {
    let markdown = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain all three paragraphs
    assert!(output.contains("First paragraph"),
            "Expected 'First paragraph' in output, got: {:?}", output);
    assert!(output.contains("Second paragraph"),
            "Expected 'Second paragraph' in output, got: {:?}", output);
    assert!(output.contains("Third paragraph"),
            "Expected 'Third paragraph' in output, got: {:?}", output);

    // Should have multiple lines (paragraphs separated by line breaks)
    let line_count = output.lines().count();
    assert!(line_count >= 3,
            "Expected at least 3 lines, got: {}", line_count);
}

/// Test bold text rendering with asterisks (**bold**)
#[test]
fn test_render_bold_asterisks() {
    let markdown = "**bold**";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain ANSI bold codes
    assert!(output.contains("\x1b[1m"),
            "Expected bold on code \\x1b[1m in output, got: {:?}", output);
    assert!(output.contains("\x1b[0m"),
            "Expected bold off code \\x1b[0m in output, got: {:?}", output);
    assert!(output.contains("bold"),
            "Expected 'bold' text in output, got: {:?}", output);
}

/// Test bold text rendering with underscores (__bold__)
#[test]
fn test_render_bold_underscores() {
    let markdown = "__bold__";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain ANSI bold codes
    assert!(output.contains("\x1b[1m"),
            "Expected bold on code \\x1b[1m in output, got: {:?}", output);
    assert!(output.contains("\x1b[0m"),
            "Expected bold off code \\x1b[0m in output, got: {:?}", output);
    assert!(output.contains("bold"),
            "Expected 'bold' text in output, got: {:?}", output);
}

/// Test mixed normal and bold text
#[test]
fn test_render_bold_mixed() {
    let markdown = "normal **bold** normal";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain normal text
    assert!(output.contains("normal"),
            "Expected 'normal' text in output, got: {:?}", output);
    // Should contain bold text
    assert!(output.contains("bold"),
            "Expected 'bold' text in output, got: {:?}", output);
    // Should contain bold formatting codes
    assert!(output.contains("\x1b[1m"),
            "Expected bold on code \\x1b[1m in output, got: {:?}", output);
    assert!(output.contains("\x1b[0m"),
            "Expected bold off code \\x1b[0m in output, got: {:?}", output);
}
