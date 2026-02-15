use mdp::parsing::{parse_markdown, Event};
use mdp::rendering::Renderer;

/// Test basic plain text rendering
#[test]
fn test_render_plain_text() {
    let markdown = "Hello world";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain "Hello world"
    assert!(
        output.contains("Hello world"),
        "Expected output to contain 'Hello world', got: {:?}",
        output
    );
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
    assert!(
        max_line_length <= 80,
        "Expected max line length <= 80, got: {}. Lines: {:?}",
        max_line_length,
        lines
    );
}

/// Test rendering multiple paragraphs
#[test]
fn test_render_multiple_lines() {
    let markdown = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain all three paragraphs
    assert!(
        output.contains("First paragraph"),
        "Expected 'First paragraph' in output, got: {:?}",
        output
    );
    assert!(
        output.contains("Second paragraph"),
        "Expected 'Second paragraph' in output, got: {:?}",
        output
    );
    assert!(
        output.contains("Third paragraph"),
        "Expected 'Third paragraph' in output, got: {:?}",
        output
    );

    // Should have multiple lines (paragraphs separated by line breaks)
    let line_count = output.lines().count();
    assert!(
        line_count >= 3,
        "Expected at least 3 lines, got: {}",
        line_count
    );
}

/// Test bold text rendering with asterisks (**bold**)
#[test]
fn test_render_bold_asterisks() {
    let markdown = "**bold**";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain ANSI bold codes
    assert!(
        output.contains("\x1b[1m"),
        "Expected bold on code \\x1b[1m in output, got: {:?}",
        output
    );
    assert!(
        output.contains("\x1b[0m"),
        "Expected bold off code \\x1b[0m in output, got: {:?}",
        output
    );
    assert!(
        output.contains("bold"),
        "Expected 'bold' text in output, got: {:?}",
        output
    );
}

/// Test bold text rendering with underscores (__bold__)
#[test]
fn test_render_bold_underscores() {
    let markdown = "__bold__";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain ANSI bold codes
    assert!(
        output.contains("\x1b[1m"),
        "Expected bold on code \\x1b[1m in output, got: {:?}",
        output
    );
    assert!(
        output.contains("\x1b[0m"),
        "Expected bold off code \\x1b[0m in output, got: {:?}",
        output
    );
    assert!(
        output.contains("bold"),
        "Expected 'bold' text in output, got: {:?}",
        output
    );
}

/// Test mixed normal and bold text
#[test]
fn test_render_bold_mixed() {
    let markdown = "normal **bold** normal";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain normal text
    assert!(
        output.contains("normal"),
        "Expected 'normal' text in output, got: {:?}",
        output
    );
    // Should contain bold text
    assert!(
        output.contains("bold"),
        "Expected 'bold' text in output, got: {:?}",
        output
    );
    // Should contain bold formatting codes
    assert!(
        output.contains("\x1b[1m"),
        "Expected bold on code \\x1b[1m in output, got: {:?}",
        output
    );
    assert!(
        output.contains("\x1b[0m"),
        "Expected bold off code \\x1b[0m in output, got: {:?}",
        output
    );
}

#[test]
fn test_render_bold_label_colon_spacing() {
    let markdown = "- **Language**: Rust";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("Language: Rust"),
        "Expected no space before colon in bold label pattern, got: {:?}",
        output
    );
    assert!(
        !plain.contains("Language : Rust"),
        "Unexpected extra space before colon, got: {:?}",
        output
    );
}

/// Test italics text rendering with asterisks (*italic*)
#[test]
fn test_render_italics_asterisks() {
    let markdown = "*italic*";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain ANSI italics codes
    assert!(
        output.contains("\x1b[3m"),
        "Expected italics on code \\x1b[3m in output, got: {:?}",
        output
    );
    assert!(
        output.contains("\x1b[0m"),
        "Expected italics off code \\x1b[0m in output, got: {:?}",
        output
    );
    assert!(
        output.contains("italic"),
        "Expected 'italic' text in output, got: {:?}",
        output
    );
}

/// Test italics text rendering with underscores (_italic_)
#[test]
fn test_render_italics_underscores() {
    let markdown = "_italic_";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain ANSI italics codes
    assert!(
        output.contains("\x1b[3m"),
        "Expected italics on code \\x1b[3m in output, got: {:?}",
        output
    );
    assert!(
        output.contains("\x1b[0m"),
        "Expected italics off code \\x1b[0m in output, got: {:?}",
        output
    );
    assert!(
        output.contains("italic"),
        "Expected 'italic' text in output, got: {:?}",
        output
    );
}

/// Test mixed normal and italics text
#[test]
fn test_render_italics_mixed() {
    let markdown = "normal *italic* normal";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain normal text
    assert!(
        output.contains("normal"),
        "Expected 'normal' text in output, got: {:?}",
        output
    );
    // Should contain italics text
    assert!(
        output.contains("italic"),
        "Expected 'italic' text in output, got: {:?}",
        output
    );
    // Should contain italics formatting codes
    assert!(
        output.contains("\x1b[3m"),
        "Expected italics on code \\x1b[3m in output, got: {:?}",
        output
    );
    assert!(
        output.contains("\x1b[0m"),
        "Expected italics off code \\x1b[0m in output, got: {:?}",
        output
    );
}

#[test]
fn test_render_intraword_emphasis_no_extra_spaces() {
    let markdown = "un*frigging*believable";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("unfriggingbelievable"),
        "Expected no extra spaces around intraword emphasis, got: {:?}",
        plain
    );
    assert!(
        !plain.contains("un frigging believable"),
        "Unexpected spaces inserted around intraword emphasis, got: {:?}",
        plain
    );
}

#[test]
fn test_render_nested_emphasis_keeps_word_spacing() {
    let markdown = "**nested *inner emphasis* text**";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("nested inner emphasis text"),
        "Expected normal spacing across nested emphasis boundaries, got: {:?}",
        plain
    );
    assert!(
        !plain.contains("nestedinner"),
        "Unexpected missing space around nested emphasis, got: {:?}",
        plain
    );
}

/// Test h1 heading rendering with box-drawing underline
#[test]
fn test_render_h1() {
    let markdown = "# Title";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the heading text
    assert!(
        output.contains("Title"),
        "Expected 'Title' in output, got: {:?}",
        output
    );
    // Should have Unicode heavy line underline
    assert!(
        output.contains("═══"),
        "Expected heavy underline in output, got: {:?}",
        output
    );
}

/// Test h2 heading rendering with --- underline
#[test]
fn test_render_h2() {
    let markdown = "## Section";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the heading text
    assert!(
        output.contains("Section"),
        "Expected 'Section' in output, got: {:?}",
        output
    );
    // Should have Unicode single line underline
    assert!(
        output.contains("───"),
        "Expected single-line underline in output, got: {:?}",
        output
    );
}

/// Test h3-h6 heading rendering
#[test]
fn test_render_h3_to_h6() {
    // Test h3
    let markdown = "### Heading 3";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    assert!(
        output.contains("Heading 3"),
        "Expected 'Heading 3' in output, got: {:?}",
        output
    );
    // h3 uses dotted-line underline
    assert!(
        output.contains("╌╌╌"),
        "Expected dotted underline for h3 in output, got: {:?}",
        output
    );

    // Test h4
    let markdown = "#### Heading 4";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    assert!(
        output.contains("Heading 4"),
        "Expected 'Heading 4' in output, got: {:?}",
        output
    );

    // Test h5
    let markdown = "##### Heading 5";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    assert!(
        output.contains("Heading 5"),
        "Expected 'Heading 5' in output, got: {:?}",
        output
    );

    // Test h6
    let markdown = "###### Heading 6";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    assert!(
        output.contains("Heading 6"),
        "Expected 'Heading 6' in output, got: {:?}",
        output
    );
}

/// Test heading text is bolded
#[test]
fn test_render_heading_bold() {
    let markdown = "# Bold Title";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the heading text
    assert!(
        output.contains("Bold Title"),
        "Expected 'Bold Title' in output, got: {:?}",
        output
    );
    // Should have bold formatting codes
    assert!(
        output.contains("\x1b[1m"),
        "Expected bold on code \\x1b[1m in output, got: {:?}",
        output
    );
    assert!(
        output.contains("\x1b[0m"),
        "Expected bold off code \\x1b[0m in output, got: {:?}",
        output
    );
}

#[test]
fn test_heading_underline_matches_visible_heading_width() {
    let markdown = "# Heading Width";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    let lines: Vec<&str> = output.lines().collect();
    assert!(lines.len() >= 2, "Expected heading and underline lines");

    let heading_line = lines[0];
    let underline_line = lines[1];

    let heading_visible = strip_ansi(heading_line);
    assert_eq!(
        heading_visible.chars().count(),
        underline_line.chars().count(),
        "Underline should match visible heading width: {:?}",
        output
    );
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for c in chars.by_ref() {
                if ('@'..='~').contains(&c) {
                    break;
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

/// Test fenced code block rendering
#[test]
fn test_render_fenced_code_block() {
    let markdown = "```rust\nlet x = 42;\n```";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the code
    assert!(
        output.contains("let x = 42;"),
        "Expected 'let x = 42;' in output, got: {:?}",
        output
    );
    // Should have fence markers (backticks)
    assert!(
        output.contains("```"),
        "Expected fence markers ``` in output, got: {:?}",
        output
    );
    // Should use monospace indication (ANSI codes for faint/dim or similar)
    // Using faint (code 2) for code blocks
    assert!(
        output.contains("\x1b[2m"),
        "Expected faint code \\x1b[2m in output, got: {:?}",
        output
    );
}

/// Test indented code block rendering (4 spaces)
#[test]
fn test_render_indented_code_block() {
    let markdown = "    let y = 100;\n    let z = 200;";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the code
    assert!(
        output.contains("let y = 100;"),
        "Expected 'let y = 100;' in output, got: {:?}",
        output
    );
    assert!(
        output.contains("let z = 200;"),
        "Expected 'let z = 200;' in output, got: {:?}",
        output
    );
    // Should use monospace indication
    assert!(
        output.contains("\x1b[2m"),
        "Expected faint code \\x1b[2m in output, got: {:?}",
        output
    );
}

/// Test inline code rendering with backticks
#[test]
fn test_render_inline_code() {
    let markdown = "Use `inline code` here";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the inline code text
    assert!(
        output.contains("inline code"),
        "Expected 'inline code' in output, got: {:?}",
        output
    );
    // Should preserve backticks
    assert!(
        output.contains("`"),
        "Expected backticks in output, got: {:?}",
        output
    );
    // Should use monospace for inline code (ANSI faint)
    assert!(
        output.contains("\x1b[2m"),
        "Expected faint code \\x1b[2m in output, got: {:?}",
        output
    );
}

#[test]
fn test_render_inline_code_with_backticks_inside() {
    let markdown = "Use ``code with `backtick` inside`` safely";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("``\x1b[2mcode with `backtick` inside\x1b[0m``"),
        "Expected renderer to use a longer backtick delimiter, got: {:?}",
        output
    );
}

/// Test block quote rendering with > prefix
#[test]
fn test_render_block_quote() {
    let markdown = "> quote";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the quote text
    assert!(
        output.contains("quote"),
        "Expected 'quote' in output, got: {:?}",
        output
    );
    // Should have quote prefix for block quote
    assert!(
        output.contains("│"),
        "Expected '│' prefix for block quote in output, got: {:?}",
        output
    );
}

/// Test multi-line block quote rendering
#[test]
fn test_render_block_quote_multiline() {
    let markdown = "> line one\n> line two\n> line three";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain all quote lines
    assert!(
        output.contains("line one"),
        "Expected 'line one' in output, got: {:?}",
        output
    );
    assert!(
        output.contains("line two"),
        "Expected 'line two' in output, got: {:?}",
        output
    );
    assert!(
        output.contains("line three"),
        "Expected 'line three' in output, got: {:?}",
        output
    );
    // Should have quote prefix for each line
    let pipe_count = output.matches('│').count();
    assert!(
        pipe_count >= 3,
        "Expected at least 3 '│' prefixes in output, got: {}",
        pipe_count
    );
}

#[test]
fn test_render_nested_block_quote_depth_prefix() {
    let markdown = "> > nested";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.lines().any(|line| line.starts_with("│ │ nested")),
        "Expected nested block quote to render with depth prefix '│ │ ', got: {:?}",
        plain
    );
}

#[test]
fn test_render_block_quote_preserves_blank_quoted_line() {
    let markdown = "> first\n>\n> second";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    let first_idx = lines
        .iter()
        .position(|line| line.contains("first"))
        .expect("Expected first quote line");
    let second_idx = lines
        .iter()
        .position(|line| line.contains("second"))
        .expect("Expected second quote line");
    let between = &lines[first_idx + 1..second_idx];

    assert!(
        between
            .iter()
            .any(|line| line.trim() == "│" || line.trim() == "│ │"),
        "Expected a quoted blank line marker between quote paragraphs, got: {:?}",
        plain
    );
}

/// Test bullet list rendering with - prefix
#[test]
fn test_render_bullet_list() {
    let markdown = "- item";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the item text
    assert!(
        output.contains("item"),
        "Expected 'item' in output, got: {:?}",
        output
    );
    // Should keep dash-like marker for dash list
    assert!(
        output.contains("- "),
        "Expected '- ' prefix for dash bullet list in output, got: {:?}",
        output
    );
}

/// Test bullet list with * prefix
#[test]
fn test_render_bullet_list_asterisk() {
    let markdown = "* item";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the item text
    assert!(
        output.contains("item"),
        "Expected 'item' in output, got: {:?}",
        output
    );
    // Star list should use bullet glyph
    assert!(
        output.contains("• "),
        "Expected '• ' prefix for star bullet list in output, got: {:?}",
        output
    );
}

/// Test bullet list with + prefix (should render as bullet glyph)
#[test]
fn test_render_bullet_list_plus() {
    let markdown = "+ item";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the item text
    assert!(
        output.contains("item"),
        "Expected 'item' in output, got: {:?}",
        output
    );
    // Should use bullet glyph for plus list
    assert!(
        output.contains("• "),
        "Expected '• ' prefix for plus bullet list in output, got: {:?}",
        output
    );
}

/// Test ordered list rendering with 1. prefix
#[test]
fn test_render_ordered_list() {
    let markdown = "1. item";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the item text
    assert!(
        output.contains("item"),
        "Expected 'item' in output, got: {:?}",
        output
    );
    // Should have 1. prefix for ordered list
    assert!(
        output.contains("1. "),
        "Expected '1. ' prefix for ordered list in output, got: {:?}",
        output
    );
}

/// Test ordered list with multiple items
#[test]
fn test_render_ordered_list_multiple_items() {
    let markdown = "1. first\n2. second\n3. third";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain all items
    assert!(
        output.contains("first"),
        "Expected 'first' in output, got: {:?}",
        output
    );
    assert!(
        output.contains("second"),
        "Expected 'second' in output, got: {:?}",
        output
    );
    assert!(
        output.contains("third"),
        "Expected 'third' in output, got: {:?}",
        output
    );
    // Should have numbered prefixes
    assert!(
        output.contains("1. "),
        "Expected '1. ' prefix in output, got: {:?}",
        output
    );
    assert!(
        output.contains("2. "),
        "Expected '2. ' prefix in output, got: {:?}",
        output
    );
    assert!(
        output.contains("3. "),
        "Expected '3. ' prefix in output, got: {:?}",
        output
    );
}

/// Test nested list rendering with indentation
#[test]
fn test_render_nested_list() {
    let markdown = "- outer\n  - inner";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain both items
    assert!(
        output.contains("outer"),
        "Expected 'outer' in output, got: {:?}",
        output
    );
    assert!(
        output.contains("inner"),
        "Expected 'inner' in output, got: {:?}",
        output
    );
    // Should have indentation for nested list (check for leading spaces)
    let lines: Vec<&str> = output.lines().collect();
    // At least one line should have leading spaces (indentation)
    let has_indentation = lines.iter().any(|l| l.starts_with(' '));
    assert!(
        has_indentation,
        "Expected indentation for nested list, got lines: {:?}",
        lines
    );
}

#[test]
fn test_render_nested_list_without_extra_blank_line() {
    let markdown = "- outer\n  - inner";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    let outer_idx = lines
        .iter()
        .position(|line| line.contains("outer"))
        .expect("Expected outer list item");
    let next = lines
        .get(outer_idx + 1)
        .expect("Expected nested list line after outer item");

    assert!(
        !next.is_empty(),
        "Unexpected blank line before nested item: {:?}",
        plain
    );
    assert!(
        next.contains("inner"),
        "Expected nested list item immediately after parent item: {:?}",
        plain
    );
}

#[test]
fn test_render_wrapped_list_item_keeps_continuation_indent() {
    let markdown = "- one two three four five six seven eight nine ten";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(20);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    assert!(
        lines.len() >= 2,
        "Expected wrapped list output across multiple lines, got: {:?}",
        plain
    );
    assert!(
        lines[0].starts_with("- "),
        "Expected first line to start with dash marker, got: {:?}",
        plain
    );
    assert!(
        lines[1].starts_with("  "),
        "Expected wrapped continuation line to keep hanging indent, got: {:?}",
        plain
    );
}

/// Test deeply nested list
#[test]
fn test_render_deeply_nested_list() {
    let markdown = "- level 1\n  - level 2\n    - level 3";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain all items
    assert!(
        output.contains("level 1"),
        "Expected 'level 1' in output, got: {:?}",
        output
    );
    assert!(
        output.contains("level 2"),
        "Expected 'level 2' in output, got: {:?}",
        output
    );
    assert!(
        output.contains("level 3"),
        "Expected 'level 3' in output, got: {:?}",
        output
    );
}

#[test]
fn test_render_tight_list_has_no_blank_lines_between_items() {
    let markdown = "- tight a\n- tight b";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    let first_idx = lines
        .iter()
        .position(|line| line.contains("tight a"))
        .expect("Expected first tight list item");
    let second_idx = lines
        .iter()
        .position(|line| line.contains("tight b"))
        .expect("Expected second tight list item");
    assert_eq!(
        second_idx,
        first_idx + 1,
        "Expected tight list items to be adjacent lines, got: {:?}",
        plain
    );
}

#[test]
fn test_render_list_with_blank_lines_between_items_stays_compact() {
    let markdown = "- loose a\n\n- loose b";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    let first_idx = lines
        .iter()
        .position(|line| line.contains("loose a"))
        .expect("Expected first loose list item");
    let second_idx = lines
        .iter()
        .position(|line| line.contains("loose b"))
        .expect("Expected second loose list item");

    assert_eq!(
        second_idx,
        first_idx + 1,
        "Expected compact list rendering (no extra blank line between items), got: {:?}",
        plain
    );
}

/// Test link rendering - [text](url) should render as "text (url)"
#[test]
fn test_render_link() {
    let markdown = "[link](http://example.com)";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the link text
    assert!(
        output.contains("link"),
        "Expected 'link' in output, got: {:?}",
        output
    );
    // Should contain the URL in parentheses
    assert!(
        output.contains("(http://example.com)"),
        "Expected '(http://example.com)' in output, got: {:?}",
        output
    );
    // Should render as "link (url)" format
    assert!(
        output.contains("link (http://example.com)"),
        "Expected 'link (http://example.com)' in output, got: {:?}",
        output
    );
}

/// Test reference link rendering - [text][ref] with missing reference should fallback
#[test]
fn test_render_reference_link() {
    let markdown = "[text][ref]";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the link text
    assert!(
        output.contains("text"),
        "Expected 'text' in output, got: {:?}",
        output
    );
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
    assert!(
        output.contains("alt text"),
        "Expected 'alt text' in output, got: {:?}",
        output
    );
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
    assert!(
        output.contains("https://example.com"),
        "Expected 'https://example.com' in output, got: {:?}",
        output
    );
    // Should render as "url (url)" format
    assert!(
        output.contains("https://example.com (https://example.com)"),
        "Expected 'https://example.com (https://example.com)' in output, got: {:?}",
        output
    );
}

/// Test email auto-link rendering - <user@example.com> should render as "user@example.com (user@example.com)"
#[test]
fn test_render_email_auto_link() {
    let markdown = "<user@example.com>";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the email
    assert!(
        output.contains("user@example.com"),
        "Expected 'user@example.com' in output, got: {:?}",
        output
    );
    // Should render as "email (email)" format
    assert!(
        output.contains("user@example.com (user@example.com)"),
        "Expected 'user@example.com (user@example.com)' in output, got: {:?}",
        output
    );
}

/// Test escape sequence: \* should render as *
#[test]
fn test_render_escape_asterisk() {
    let markdown = "\\*";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the literal asterisk (not interpreted as emphasis)
    assert!(
        output.contains("*"),
        "Expected '*' in output, got: {:?}",
        output
    );
    // Should NOT contain emphasis formatting codes (since it's escaped)
    assert!(
        !output.contains("\x1b[3m"),
        "Expected no italics code in output for escaped asterisk, got: {:?}",
        output
    );
}

/// Test escape sequence: \[ should render as [
#[test]
fn test_render_escape_bracket() {
    let markdown = "\\[";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the literal bracket
    assert!(
        output.contains("["),
        "Expected '[' in output, got: {:?}",
        output
    );
}

/// Test escape sequence: \\ should render as \
#[test]
fn test_render_escape_backslash() {
    let markdown = "\\\\";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the literal backslash
    assert!(
        output.contains("\\"),
        "Expected '\\' in output, got: {:?}",
        output
    );
}

/// Test multiple escape sequences: \*text\* should render as *text*
#[test]
fn test_render_multiple_escapes() {
    let markdown = "\\*text\\*";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should contain the literal text with asterisks
    assert!(
        output.contains("*text*"),
        "Expected '*text*' in output, got: {:?}",
        output
    );
    // Should NOT contain emphasis formatting codes
    assert!(
        !output.contains("\x1b[3m"),
        "Expected no italics code in output for escaped asterisks, got: {:?}",
        output
    );
}

/// Test horizontal rule rendering for '---'
#[test]
fn test_render_horizontal_rule_dashes() {
    let markdown = "---";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    let first_line = output.lines().next().unwrap_or("");
    assert_eq!(
        first_line.chars().count(),
        80,
        "Expected full-width horizontal rule, got: {:?}",
        output
    );
    assert!(
        first_line.chars().all(|c| c == '─'),
        "Expected Unicode line-only horizontal rule output, got: {:?}",
        output
    );
}

/// Test horizontal rule rendering for '***'
#[test]
fn test_render_horizontal_rule_asterisks() {
    let markdown = "***";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    let first_line = output.lines().next().unwrap_or("");
    assert_eq!(
        first_line.chars().count(),
        80,
        "Expected full-width rule, got: {:?}",
        output
    );
    assert!(first_line.chars().all(|c| c == '═'));
}

/// Test horizontal rule rendering for '___'
#[test]
fn test_render_horizontal_rule_underscores() {
    let markdown = "___";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    let first_line = output.lines().next().unwrap_or("");
    assert_eq!(
        first_line.chars().count(),
        80,
        "Expected full-width rule, got: {:?}",
        output
    );
    assert!(first_line.chars().all(|c| c == '┄'));
}

/// Test strikethrough rendering
#[test]
fn test_render_strikethrough() {
    let markdown = "~~text~~";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    renderer.set_strikethrough_fallback(false);
    let output = renderer.render(&events);

    assert!(
        output.contains("\x1b[9m"),
        "Expected strikethrough on code \\x1b[9m, got: {:?}",
        output
    );
    assert!(
        output.contains("\x1b[0m"),
        "Expected reset code \\x1b[0m, got: {:?}",
        output
    );
    assert!(
        output.contains("text"),
        "Expected strikethrough text content, got: {:?}",
        output
    );
}

/// Test mixed normal and strikethrough text
#[test]
fn test_render_strikethrough_mixed() {
    let markdown = "normal ~~strikethrough~~ normal";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    renderer.set_strikethrough_fallback(false);
    let output = renderer.render(&events);

    assert!(
        output.contains("normal"),
        "Expected normal text, got: {:?}",
        output
    );
    assert!(
        output.contains("strikethrough"),
        "Expected strikethrough text, got: {:?}",
        output
    );
    assert!(
        output.contains("\x1b[9m"),
        "Expected strikethrough formatting, got: {:?}",
        output
    );
}

#[test]
fn test_render_strikethrough_fallback_mode() {
    let markdown = "~~text~~";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    renderer.set_strikethrough_fallback(true);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("t\u{0336}e\u{0336}x\u{0336}t\u{0336}"),
        "Expected Unicode combining strikethrough fallback, got: {:?}",
        plain
    );
    assert!(
        !output.contains("\x1b[9m"),
        "Fallback mode should avoid ANSI strikethrough escape, got: {:?}",
        output
    );
}

#[test]
fn test_render_strikethrough_fallback_crosses_spaces() {
    let markdown = "~~strikethrough extension~~";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    renderer.set_strikethrough_fallback(true);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains(" \u{0336}"),
        "Expected combining strikethrough on spaces too, got: {:?}",
        plain
    );
}

/// Test soft break handling (single newline in paragraph)
#[test]
fn test_render_soft_break() {
    let markdown = "line one\nline two";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("line one line two"),
        "Expected soft break rendered as space, got: {:?}",
        output
    );
}

/// Test hard break handling (two spaces + newline)
#[test]
fn test_render_hard_break() {
    let markdown = "line one  \nline two";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("line one\nline two"),
        "Expected hard break rendered as newline, got: {:?}",
        output
    );
}

/// Test very long line wrapping (> 200 chars)
#[test]
fn test_render_very_long_line() {
    let markdown = "This is a very long markdown line that intentionally exceeds two hundred characters to validate wrapping behavior in the renderer and ensure the output still respects the configured width for display in a terminal pager environment.";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    let lines: Vec<&str> = output.lines().collect();
    let max_line_length = lines.iter().map(|line| line.len()).max().unwrap_or(0);
    assert!(
        max_line_length <= 80,
        "Expected max line length <= 80, got {} with lines {:?}",
        max_line_length,
        lines
    );
}

/// Test Unicode UTF-8 text rendering
#[test]
fn test_render_unicode_text() {
    let markdown = "Café naïve résumé";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("Café") && output.contains("naïve") && output.contains("résumé"),
        "Expected accented Unicode text to render, got: {:?}",
        output
    );
}

/// Test emoji rendering
#[test]
fn test_render_emoji() {
    let markdown = "Status: ✅ done 🚀";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("✅") && output.contains("🚀"),
        "Expected emoji characters to render, got: {:?}",
        output
    );
}

/// Test CJK rendering and wrapping with double-width chars
#[test]
fn test_render_cjk_characters() {
    let markdown = "你好世界 你好世界 你好世界";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(10);
    let output = renderer.render(&events);

    let lines: Vec<&str> = output.lines().collect();
    assert!(
        lines.len() >= 2,
        "Expected wrapping for CJK with small width, got: {:?}",
        lines
    );
    assert!(
        output.contains("你好世界"),
        "Expected CJK text to render, got: {:?}",
        output
    );
}

/// Test unchecked task list rendering
#[test]
fn test_render_task_list_unchecked() {
    let markdown = "- [ ] todo item";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("[ ]"),
        "Expected unchecked task marker '[ ]', got: {:?}",
        output
    );
    assert!(
        output.contains("todo item"),
        "Expected task content, got: {:?}",
        output
    );
}

/// Test checked task list rendering
#[test]
fn test_render_task_list_checked() {
    let markdown = "- [x] done item";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("[x]"),
        "Expected checked task marker '[x]', got: {:?}",
        output
    );
    assert!(
        output.contains("done item"),
        "Expected task content, got: {:?}",
        output
    );
}

/// Test mixed task list rendering
#[test]
fn test_render_task_list_mixed() {
    let markdown = "- [ ] first\n- [x] second\n- [X] third";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("[ ]"),
        "Expected unchecked task marker in mixed list, got: {:?}",
        output
    );
    let checked_count = output.matches("[x]").count();
    assert!(
        checked_count >= 2,
        "Expected at least two checked markers '[x]' (for x/X), got {checked_count} in {:?}",
        output
    );
}

/// Test simple table rendering
#[test]
fn test_render_simple_table() {
    let markdown = "| Name | Age |\n| --- | --- |\n| Alice | 30 |\n| Bob | 25 |";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("│ Name"),
        "Expected table header, got: {:?}",
        output
    );
    assert!(
        output.contains("Alice"),
        "Expected table row, got: {:?}",
        output
    );
    assert!(
        output.contains("┌") && output.contains("│") && output.contains("┘"),
        "Expected bordered table output, got: {:?}",
        output
    );
}

/// Test table alignment rendering
#[test]
fn test_render_table_alignment() {
    let markdown = "| Left | Right | Center |\n| :--- | ---: | :---: |\n| a | 1 | x |";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        !output.contains(':'),
        "Expected no visible alignment colons in rendered table, got: {:?}",
        output
    );
    assert!(
        output.contains(" 1 "),
        "Expected right-aligned numeric cell formatting, got: {:?}",
        output
    );
}

/// Test table fallback rendering when table is too wide
#[test]
fn test_render_table_fallback() {
    let markdown = "| VeryLongColumnHeader | AnotherVeryLongColumnHeader |\n| --- | --- |\n| ExtremelyLongCellValue | AnotherExtremelyLongCellValue |";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(20);
    let output = renderer.render(&events);

    assert!(
        output.contains("VeryLongColumnHeader"),
        "Expected fallback text to include header content, got: {:?}",
        output
    );
    assert!(
        !output.contains("│"),
        "Expected plain-text fallback (no table borders) for narrow width, got: {:?}",
        output
    );
}

/// Test footnote reference rendering
#[test]
fn test_render_footnote_reference() {
    let markdown = "Text with footnote[^1].\n\n[^1]: Note.";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("[^1]"),
        "Expected footnote reference marker, got: {:?}",
        output
    );
}

/// Test footnote definition rendering
#[test]
fn test_render_footnote_definition() {
    let markdown = "[^1]: definition text";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("[^1]") && output.contains("definition text"),
        "Expected rendered footnote definition, got: {:?}",
        output
    );
    assert!(
        output.contains("Footnotes"),
        "Expected footnote section separator heading, got: {:?}",
        output
    );
    assert!(
        output.contains("\x1b[2m"),
        "Expected dim style for footnotes, got: {:?}",
        output
    );
}

#[test]
fn test_render_footnote_definition_inline_after_label() {
    let markdown = "[^1]: definition text";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("[^1] definition text"),
        "Expected footnote text on same line as label, got: {:?}",
        plain
    );
}

#[test]
fn test_render_footnote_separator_once_for_multiple_definitions() {
    let markdown = "[^1]: one\n[^2]: two";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert_eq!(
        output.matches("Footnotes").count(),
        1,
        "Expected one footnote section separator, got: {:?}",
        output
    );
}

/// Test definition list rendering
#[test]
fn test_render_definition_list() {
    let markdown = "Term\n: Definition text";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("Term") && output.contains("Definition text"),
        "Expected definition list term and definition, got: {:?}",
        output
    );
}

/// Test smart punctuation ellipsis
#[test]
fn test_render_ellipsis() {
    let markdown = "Wait...";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    assert!(
        output.contains("Wait..."),
        "Expected literal ellipsis dots to be preserved, got: {:?}",
        output
    );
}

/// Test smart punctuation en-dash
#[test]
fn test_render_endash() {
    let markdown = "A -- B";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    assert!(
        output.contains("A -- B"),
        "Expected literal double dash to be preserved, got: {:?}",
        output
    );
}

/// Test smart punctuation em-dash
#[test]
fn test_render_emdash() {
    let markdown = "A --- B";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    assert!(
        output.contains("A --- B"),
        "Expected literal triple dash to be preserved, got: {:?}",
        output
    );
}

#[test]
fn test_render_smart_punctuation_preserves_quoted_literals() {
    let markdown = "Use \"---\", \"--\", and \"...\" literally.";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("\"---\""),
        "Expected literal triple dash inside quotes, got: {:?}",
        plain
    );
    assert!(
        plain.contains("\"--\""),
        "Expected literal double dash inside quotes, got: {:?}",
        plain
    );
    assert!(
        plain.contains("\"...\""),
        "Expected literal ellipsis dots inside quotes, got: {:?}",
        plain
    );
}

#[test]
fn test_render_block_quote_prefixes_content_lines() {
    let markdown = "> quoted text\n>\n> second paragraph";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    let quote_text_line = lines
        .iter()
        .find(|line| line.contains("quoted text"))
        .expect("Expected quoted text line");
    assert!(
        quote_text_line.starts_with("│ "),
        "Expected quote content line to keep '│ ' prefix, got: {:?}",
        plain
    );
}

#[test]
fn test_render_code_block_preserves_lines() {
    let markdown = "```rust\nlet x = 1;\nlet y = 2;\n```";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    assert!(
        lines.iter().any(|line| line == &"```rust"),
        "Expected opening fence line, got: {:?}",
        plain
    );
    assert!(
        lines.iter().any(|line| line.contains("let x = 1;")),
        "Expected first code line, got: {:?}",
        plain
    );
    assert!(
        lines.iter().any(|line| line.contains("let y = 2;")),
        "Expected second code line, got: {:?}",
        plain
    );
    assert!(
        lines.iter().any(|line| line == &"```"),
        "Expected closing fence line, got: {:?}",
        plain
    );
    let closing_idx = lines
        .iter()
        .rposition(|line| *line == "```")
        .expect("Expected closing fence");
    let prev_line = lines
        .get(closing_idx.saturating_sub(1))
        .copied()
        .unwrap_or_default();
    assert!(
        !prev_line.is_empty(),
        "Expected no empty line immediately before closing fence, got: {:?}",
        plain
    );
}

#[test]
fn test_render_ordered_list_numbering_after_nested_list() {
    let markdown = "1. top\n   1. nested\n2. next";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("1. top"),
        "Expected first parent item numbering, got: {:?}",
        plain
    );
    assert!(
        plain.contains("  1. nested"),
        "Expected nested numbering to start at 1, got: {:?}",
        plain
    );
    assert!(
        plain.contains("2. next"),
        "Expected parent numbering to resume at 2 after nested list, got: {:?}",
        plain
    );
}

#[test]
fn test_render_escaped_triple_dash_preserved() {
    let markdown = "Escaped: \\---";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("Escaped: ---"),
        "Expected escaped triple dash to stay literal, got: {:?}",
        plain
    );
}

#[test]
fn test_render_html_inline_and_block_passthrough() {
    let markdown = "Inline <span>x</span>\n\n<div>\nblock\n</div>";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("<span>") && plain.contains("</span>"),
        "Expected inline HTML tags to be preserved, got: {:?}",
        plain
    );
    assert!(
        plain.contains("<div>") && plain.contains("</div>"),
        "Expected HTML block tags to be preserved, got: {:?}",
        plain
    );
}

#[test]
fn test_render_inline_html_keeps_inner_spacing() {
    let markdown = "Inline HTML: <span>x</span> end";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("<span>x</span>"),
        "Expected no inserted spaces inside inline HTML boundaries, got: {:?}",
        plain
    );
}

#[test]
fn test_render_blockquote_heading_underline_keeps_quote_prefix() {
    let markdown = "> ## Quoted";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    assert!(
        lines.iter().any(|line| line.starts_with("│ Quoted")),
        "Expected quoted heading line, got: {:?}",
        plain
    );
    assert!(
        lines.iter().any(|line| line.starts_with("│ ─")),
        "Expected heading underline to stay inside quote context, got: {:?}",
        plain
    );

    let heading_line = lines
        .iter()
        .find(|line| line.starts_with("│ Quoted"))
        .expect("Expected quoted heading line");
    let underline_line = lines
        .iter()
        .find(|line| line.starts_with("│ ─"))
        .expect("Expected quoted heading underline line");

    let heading_text = heading_line.strip_prefix("│ ").unwrap_or(heading_line);
    let underline_text = underline_line.strip_prefix("│ ").unwrap_or(underline_line);
    assert_eq!(
        heading_text.chars().count(),
        underline_text.chars().count(),
        "Quoted heading underline should match heading text width, got: {:?}",
        plain
    );
}

#[test]
fn test_render_list_second_paragraph_keeps_continuation_indent() {
    let markdown = "- item first paragraph\n\n  second paragraph";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();
    let idx = lines
        .iter()
        .position(|line| line.contains("second paragraph"))
        .expect("Expected second paragraph line");
    assert!(
        lines[idx].starts_with("  "),
        "Expected second paragraph to keep list continuation indent, got: {:?}",
        plain
    );
}

#[test]
fn test_render_blockquote_inside_list_keeps_list_indent() {
    let markdown = "- item\n\n  > quoted";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    assert!(
        plain.lines().any(|line| line.starts_with("  │ quoted")),
        "Expected nested block quote to keep list indentation, got: {:?}",
        plain
    );
}

#[test]
fn test_render_codeblock_inside_list_keeps_closing_fence_indent() {
    let markdown = "- item\n\n  ```text\n  code line\n  ```";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    let open_idx = lines
        .iter()
        .position(|line| line.starts_with("  ```text"))
        .expect("Expected indented opening code fence inside list item");
    let close_idx = lines
        .iter()
        .position(|line| line == &"  ```")
        .expect("Expected indented closing code fence inside list item");
    assert!(
        close_idx > open_idx,
        "Expected closing fence to appear after opening fence, got: {:?}",
        plain
    );
}
