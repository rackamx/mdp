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

/// Test italics text rendering with asterisks (*italic*)
#[test]
fn test_render_italics_asterisks() {
    let markdown = "*italic*";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain ANSI italics codes
    assert!(output.contains("\x1b[3m"),
            "Expected italics on code \\x1b[3m in output, got: {:?}", output);
    assert!(output.contains("\x1b[0m"),
            "Expected italics off code \\x1b[0m in output, got: {:?}", output);
    assert!(output.contains("italic"),
            "Expected 'italic' text in output, got: {:?}", output);
}

/// Test italics text rendering with underscores (_italic_)
#[test]
fn test_render_italics_underscores() {
    let markdown = "_italic_";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain ANSI italics codes
    assert!(output.contains("\x1b[3m"),
            "Expected italics on code \\x1b[3m in output, got: {:?}", output);
    assert!(output.contains("\x1b[0m"),
            "Expected italics off code \\x1b[0m in output, got: {:?}", output);
    assert!(output.contains("italic"),
            "Expected 'italic' text in output, got: {:?}", output);
}

/// Test mixed normal and italics text
#[test]
fn test_render_italics_mixed() {
    let markdown = "normal *italic* normal";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain normal text
    assert!(output.contains("normal"),
            "Expected 'normal' text in output, got: {:?}", output);
    // Should contain italics text
    assert!(output.contains("italic"),
            "Expected 'italic' text in output, got: {:?}", output);
    // Should contain italics formatting codes
    assert!(output.contains("\x1b[3m"),
            "Expected italics on code \\x1b[3m in output, got: {:?}", output);
    assert!(output.contains("\x1b[0m"),
            "Expected italics off code \\x1b[0m in output, got: {:?}", output);
}

/// Test h1 heading rendering with === underline
#[test]
fn test_render_h1() {
    let markdown = "# Title";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the heading text
    assert!(output.contains("Title"),
            "Expected 'Title' in output, got: {:?}", output);
    // Should have === underline (at least 3 characters for a 5-letter word)
    assert!(output.contains("==="),
            "Expected '===' underline in output, got: {:?}", output);
}

/// Test h2 heading rendering with --- underline
#[test]
fn test_render_h2() {
    let markdown = "## Section";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the heading text
    assert!(output.contains("Section"),
            "Expected 'Section' in output, got: {:?}", output);
    // Should have --- underline
    assert!(output.contains("---"),
            "Expected '---' underline in output, got: {:?}", output);
}

/// Test h3-h6 heading rendering
#[test]
fn test_render_h3_to_h6() {
    // Test h3
    let markdown = "### Heading 3";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    assert!(output.contains("Heading 3"),
            "Expected 'Heading 3' in output, got: {:?}", output);
    // h3 uses ~ underline
    assert!(output.contains("~~~"),
            "Expected '~~~' underline for h3 in output, got: {:?}", output);

    // Test h4
    let markdown = "#### Heading 4";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    assert!(output.contains("Heading 4"),
            "Expected 'Heading 4' in output, got: {:?}", output);

    // Test h5
    let markdown = "##### Heading 5";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    assert!(output.contains("Heading 5"),
            "Expected 'Heading 5' in output, got: {:?}", output);

    // Test h6
    let markdown = "###### Heading 6";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    assert!(output.contains("Heading 6"),
            "Expected 'Heading 6' in output, got: {:?}", output);
}

/// Test heading text is bolded
#[test]
fn test_render_heading_bold() {
    let markdown = "# Bold Title";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the heading text
    assert!(output.contains("Bold Title"),
            "Expected 'Bold Title' in output, got: {:?}", output);
    // Should have bold formatting codes
    assert!(output.contains("\x1b[1m"),
            "Expected bold on code \\x1b[1m in output, got: {:?}", output);
    assert!(output.contains("\x1b[0m"),
            "Expected bold off code \\x1b[0m in output, got: {:?}", output);
}

/// Test fenced code block rendering
#[test]
fn test_render_fenced_code_block() {
    let markdown = "```rust\nlet x = 42;\n```";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the code
    assert!(output.contains("let x = 42;"),
            "Expected 'let x = 42;' in output, got: {:?}", output);
    // Should have fence markers (backticks)
    assert!(output.contains("```"),
            "Expected fence markers ``` in output, got: {:?}", output);
    // Should use monospace indication (ANSI codes for faint/dim or similar)
    // Using faint (code 2) for code blocks
    assert!(output.contains("\x1b[2m"),
            "Expected faint code \\x1b[2m in output, got: {:?}", output);
}

/// Test indented code block rendering (4 spaces)
#[test]
fn test_render_indented_code_block() {
    let markdown = "    let y = 100;\n    let z = 200;";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the code
    assert!(output.contains("let y = 100;"),
            "Expected 'let y = 100;' in output, got: {:?}", output);
    assert!(output.contains("let z = 200;"),
            "Expected 'let z = 200;' in output, got: {:?}", output);
    // Should use monospace indication
    assert!(output.contains("\x1b[2m"),
            "Expected faint code \\x1b[2m in output, got: {:?}", output);
}

/// Test inline code rendering with backticks
#[test]
fn test_render_inline_code() {
    let markdown = "Use `inline code` here";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the inline code text
    assert!(output.contains("inline code"),
            "Expected 'inline code' in output, got: {:?}", output);
    // Should preserve backticks
    assert!(output.contains("`"),
            "Expected backticks in output, got: {:?}", output);
    // Should use monospace for inline code (ANSI faint)
    assert!(output.contains("\x1b[2m"),
            "Expected faint code \\x1b[2m in output, got: {:?}", output);
}

/// Test block quote rendering with > prefix
#[test]
fn test_render_block_quote() {
    let markdown = "> quote";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the quote text
    assert!(output.contains("quote"),
            "Expected 'quote' in output, got: {:?}", output);
    // Should have | prefix for block quote
    assert!(output.contains("|"),
            "Expected '|' prefix for block quote in output, got: {:?}", output);
}

/// Test multi-line block quote rendering
#[test]
fn test_render_block_quote_multiline() {
    let markdown = "> line one\n> line two\n> line three";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain all quote lines
    assert!(output.contains("line one"),
            "Expected 'line one' in output, got: {:?}", output);
    assert!(output.contains("line two"),
            "Expected 'line two' in output, got: {:?}", output);
    assert!(output.contains("line three"),
            "Expected 'line three' in output, got: {:?}", output);
    // Should have | prefix for each line
    let pipe_count = output.matches('|').count();
    assert!(pipe_count >= 3,
            "Expected at least 3 '|' prefixes in output, got: {}", pipe_count);
}

/// Test bullet list rendering with - prefix
#[test]
fn test_render_bullet_list() {
    let markdown = "- item";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the item text
    assert!(output.contains("item"),
            "Expected 'item' in output, got: {:?}", output);
    // Should have * prefix for bullet list
    assert!(output.contains("* "),
            "Expected '* ' prefix for bullet list in output, got: {:?}", output);
}

/// Test bullet list with * prefix
#[test]
fn test_render_bullet_list_asterisk() {
    let markdown = "* item";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the item text
    assert!(output.contains("item"),
            "Expected 'item' in output, got: {:?}", output);
    // Should have * prefix for bullet list
    assert!(output.contains("* "),
            "Expected '* ' prefix for bullet list in output, got: {:?}", output);
}

/// Test bullet list with + prefix (should normalize to *)
#[test]
fn test_render_bullet_list_plus() {
    let markdown = "+ item";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the item text
    assert!(output.contains("item"),
            "Expected 'item' in output, got: {:?}", output);
    // Should have * prefix for bullet list (normalized from +)
    assert!(output.contains("* "),
            "Expected '* ' prefix for bullet list in output, got: {:?}", output);
}

/// Test ordered list rendering with 1. prefix
#[test]
fn test_render_ordered_list() {
    let markdown = "1. item";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the item text
    assert!(output.contains("item"),
            "Expected 'item' in output, got: {:?}", output);
    // Should have 1. prefix for ordered list
    assert!(output.contains("1. "),
            "Expected '1. ' prefix for ordered list in output, got: {:?}", output);
}

/// Test ordered list with multiple items
#[test]
fn test_render_ordered_list_multiple_items() {
    let markdown = "1. first\n2. second\n3. third";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain all items
    assert!(output.contains("first"),
            "Expected 'first' in output, got: {:?}", output);
    assert!(output.contains("second"),
            "Expected 'second' in output, got: {:?}", output);
    assert!(output.contains("third"),
            "Expected 'third' in output, got: {:?}", output);
    // Should have numbered prefixes
    assert!(output.contains("1. "),
            "Expected '1. ' prefix in output, got: {:?}", output);
    assert!(output.contains("2. "),
            "Expected '2. ' prefix in output, got: {:?}", output);
    assert!(output.contains("3. "),
            "Expected '3. ' prefix in output, got: {:?}", output);
}

/// Test nested list rendering with indentation
#[test]
fn test_render_nested_list() {
    let markdown = "- outer\n  - inner";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain both items
    assert!(output.contains("outer"),
            "Expected 'outer' in output, got: {:?}", output);
    assert!(output.contains("inner"),
            "Expected 'inner' in output, got: {:?}", output);
    // Should have indentation for nested list (check for leading spaces)
    let lines: Vec<&str> = output.lines().collect();
    // At least one line should have leading spaces (indentation)
    let has_indentation = lines.iter().any(|l| l.starts_with(' '));
    assert!(has_indentation,
            "Expected indentation for nested list, got lines: {:?}", lines);
}

/// Test deeply nested list
#[test]
fn test_render_deeply_nested_list() {
    let markdown = "- level 1\n  - level 2\n    - level 3";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain all items
    assert!(output.contains("level 1"),
            "Expected 'level 1' in output, got: {:?}", output);
    assert!(output.contains("level 2"),
            "Expected 'level 2' in output, got: {:?}", output);
    assert!(output.contains("level 3"),
            "Expected 'level 3' in output, got: {:?}", output);
}

/// Test link rendering - [text](url) should render as "text (url)"
#[test]
fn test_render_link() {
    let markdown = "[link](http://example.com)";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the link text
    assert!(output.contains("link"),
            "Expected 'link' in output, got: {:?}", output);
    // Should contain the URL in parentheses
    assert!(output.contains("(http://example.com)"),
            "Expected '(http://example.com)' in output, got: {:?}", output);
    // Should render as "link (url)" format
    assert!(output.contains("link (http://example.com)"),
            "Expected 'link (http://example.com)' in output, got: {:?}", output);
}

/// Test reference link rendering - [text][ref] with missing reference should fallback
#[test]
fn test_render_reference_link() {
    let markdown = "[text][ref]";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the link text
    assert!(output.contains("text"),
            "Expected 'text' in output, got: {:?}", output);
    // For undefined reference, it should fall back to showing just the text
    // or text with empty parentheses since the reference is not defined
}

/// Test image rendering - ![alt](url) should show alt text
#[test]
fn test_render_image_alt_text() {
    let markdown = "![alt text](http://example.com/image.png)";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the alt text
    assert!(output.contains("alt text"),
            "Expected 'alt text' in output, got: {:?}", output);
    // Should render as "[alt text]" format (not actual image)
}

/// Test URL auto-link rendering - <https://example.com> should render as "https://example.com (https://example.com)"
#[test]
fn test_render_url_auto_link() {
    let markdown = "<https://example.com>";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the URL
    assert!(output.contains("https://example.com"),
            "Expected 'https://example.com' in output, got: {:?}", output);
    // Should render as "url (url)" format
    assert!(output.contains("https://example.com (https://example.com)"),
            "Expected 'https://example.com (https://example.com)' in output, got: {:?}", output);
}

/// Test email auto-link rendering - <user@example.com> should render as "user@example.com (user@example.com)"
#[test]
fn test_render_email_auto_link() {
    let markdown = "<user@example.com>";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the email
    assert!(output.contains("user@example.com"),
            "Expected 'user@example.com' in output, got: {:?}", output);
    // Should render as "email (email)" format
    assert!(output.contains("user@example.com (user@example.com)"),
            "Expected 'user@example.com (user@example.com)' in output, got: {:?}", output);
}

/// Test escape sequence: \* should render as *
#[test]
fn test_render_escape_asterisk() {
    let markdown = "\\*";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the literal asterisk (not interpreted as emphasis)
    assert!(output.contains("*"),
            "Expected '*' in output, got: {:?}", output);
    // Should NOT contain emphasis formatting codes (since it's escaped)
    assert!(!output.contains("\x1b[3m"),
            "Expected no italics code in output for escaped asterisk, got: {:?}", output);
}

/// Test escape sequence: \[ should render as [
#[test]
fn test_render_escape_bracket() {
    let markdown = "\\[";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the literal bracket
    assert!(output.contains("["),
            "Expected '[' in output, got: {:?}", output);
}

/// Test escape sequence: \\ should render as \
#[test]
fn test_render_escape_backslash() {
    let markdown = "\\\\";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the literal backslash
    assert!(output.contains("\\"),
            "Expected '\\' in output, got: {:?}", output);
}

/// Test multiple escape sequences: \*text\* should render as *text*
#[test]
fn test_render_multiple_escapes() {
    let markdown = "\\*text\\*";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the literal text with asterisks
    assert!(output.contains("*text*"),
            "Expected '*text*' in output, got: {:?}", output);
    // Should NOT contain emphasis formatting codes
    assert!(!output.contains("\x1b[3m"),
            "Expected no italics code in output for escaped asterisks, got: {:?}", output);
}
