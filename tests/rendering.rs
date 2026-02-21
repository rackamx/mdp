use mdp::parsing::{parse_markdown, Block, CellAlignment, Event};
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
        output.contains("\x1b[22m"),
        "Expected bold off code \\x1b[22m in output, got: {:?}",
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
        output.contains("\x1b[22m"),
        "Expected bold off code \\x1b[22m in output, got: {:?}",
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
        output.contains("\x1b[22m"),
        "Expected bold off code \\x1b[22m in output, got: {:?}",
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
        output.contains("\x1b[23m"),
        "Expected italics off code \\x1b[23m in output, got: {:?}",
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
        output.contains("\x1b[23m"),
        "Expected italics off code \\x1b[23m in output, got: {:?}",
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
        output.contains("\x1b[23m"),
        "Expected italics off code \\x1b[23m in output, got: {:?}",
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

#[test]
fn test_render_nested_inner_strong_keeps_outer_italics_state() {
    let markdown = "*nested **inner strong** text*";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("nested inner strong text"),
        "Expected text content preserved, got: {:?}",
        plain
    );
    assert!(
        output.contains("\x1b[3m") && output.contains("\x1b[23m"),
        "Expected italic start/end codes for outer emphasis, got: {:?}",
        output
    );
    assert!(
        output.contains("\x1b[1m") && output.contains("\x1b[22m"),
        "Expected bold start/end codes for inner strong span, got: {:?}",
        output
    );
}

#[test]
fn test_render_overlapping_emphasis_keeps_outer_italics_active() {
    let markdown = "*a **b* c**\n\n**a *b** c*";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("a b c\n\na b c"),
        "Expected overlapping emphasis text content, got: {:?}",
        plain
    );
    assert_eq!(
        output.matches("\x1b[3m").count(),
        2,
        "Expected one italic start per paragraph for overlapping emphasis, got: {:?}",
        output
    );
    assert_eq!(
        output.matches("\x1b[23m").count(),
        2,
        "Expected one italic end per paragraph for overlapping emphasis, got: {:?}",
        output
    );
}

#[test]
fn test_render_wrapped_italics_reapplies_style_on_continuation_lines() {
    let markdown = "*test this very long sentence test this very long sentence test this very long sentence test this very long sentence*";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(40);
    let output = renderer.render(&events);
    let lines: Vec<&str> = output.lines().collect();

    assert!(
        lines.len() > 1,
        "Expected wrapped output across multiple lines, got: {:?}",
        output
    );
    assert!(
        lines[0].starts_with("\x1b[3m"),
        "Expected first wrapped line to start with italics code, got: {:?}",
        lines[0]
    );
    assert!(
        lines.iter().skip(1).all(|line| line.starts_with("\x1b[3m")),
        "Expected continuation lines to reapply italics code, got lines: {:?}",
        lines
    );
    assert!(
        lines.last().is_some_and(|line| line.contains("\x1b[23m")),
        "Expected final line to close italics style, got lines: {:?}",
        lines
    );
}

/// Test h1 heading rendering
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
    assert!(
        strip_ansi(&output).contains("# Title"),
        "Expected Markdown heading marker in output, got: {:?}",
        output
    );
    // Heading styling should not rely on underline anymore
    assert!(
        !output.contains("\x1b[4m") && !output.contains("\x1b[24m"),
        "Expected no ANSI underline codes for headings, got: {:?}",
        output
    );
}

/// Test h2 heading rendering
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
    assert!(
        strip_ansi(&output).contains("## Section"),
        "Expected Markdown heading marker in output, got: {:?}",
        output
    );
    assert!(
        !output.contains("\x1b[4m") && !output.contains("\x1b[24m"),
        "Expected no ANSI underline codes for headings, got: {:?}",
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
    assert!(
        !output.contains("\x1b[4m") && !output.contains("\x1b[24m"),
        "Expected no ANSI underline for h3 heading, got: {:?}",
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
        output.contains("\x1b[22m"),
        "Expected bold off code \\x1b[22m in output, got: {:?}",
        output
    );
}

#[test]
fn test_heading_renders_as_single_line_without_underline() {
    let markdown = "# Heading Width";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.lines().count() == 1,
        "Expected compact single-line heading rendering, got: {:?}",
        output
    );
    assert!(
        !output.contains("\x1b[4m") && !output.contains("\x1b[24m"),
        "Expected no ANSI underline for heading, got: {:?}",
        output
    );
}

#[test]
fn test_render_consecutive_atx_headings_have_no_blank_lines_between() {
    let markdown = "# H1\n## H2\n### H3";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    assert!(
        lines.len() >= 3,
        "Expected at least three heading lines, got: {:?}",
        plain
    );
    let h1 = lines
        .iter()
        .position(|l| l.contains("# H1"))
        .expect("Missing H1");
    let h2 = lines
        .iter()
        .position(|l| l.contains("## H2"))
        .expect("Missing H2");
    let h3 = lines
        .iter()
        .position(|l| l.contains("### H3"))
        .expect("Missing H3");

    assert_eq!(
        h2,
        h1 + 1,
        "Unexpected blank line between H1 and H2: {:?}",
        plain
    );
    assert_eq!(
        h3,
        h2 + 1,
        "Unexpected blank line between H2 and H3: {:?}",
        plain
    );
}

#[test]
fn test_render_setext_heading_keeps_underline() {
    let markdown = "Setext H1\n=========\n\nSetext H2\n---------";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let lines: Vec<&str> = output.lines().collect();

    assert!(
        lines
            .iter()
            .any(|line| line.contains("\x1b[4m# Setext H1") && line.contains("\x1b[24m")),
        "Expected setext h1 heading to be underlined, got: {:?}",
        output
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("\x1b[4m## Setext H2") && line.contains("\x1b[24m")),
        "Expected setext h2 heading to be underlined, got: {:?}",
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

fn strip_combining_marks(input: &str) -> String {
    input
        .chars()
        .filter(|&c| c != '\u{0332}' && c != '\u{0333}' && c != '\u{0336}')
        .collect()
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
    assert!(
        !output.contains("```"),
        "Expected indented code block to render without fenced markers, got: {:?}",
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

#[test]
fn test_render_html_comment_is_dimmed() {
    let markdown = "<!-- hidden note -->";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("\x1b[2m<!-- hidden note -->\x1b[22m"),
        "Expected HTML comment to be dimmed, got: {:?}",
        output
    );
}

#[test]
fn test_render_non_comment_html_is_not_dimmed() {
    let markdown = "<span>visible</span>";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("<span>visible</span>"),
        "Expected raw HTML to render as-is, got: {:?}",
        output
    );
    assert!(
        !output.contains("\x1b[2m<span>visible</span>\x1b[22m"),
        "Expected non-comment HTML to avoid comment dimming, got: {:?}",
        output
    );
}

#[test]
fn test_render_html_comment_block_has_no_extra_empty_comment_line() {
    let markdown = "before\n\n<!-- hidden -->\nafter";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        !plain.contains("\n\n\n"),
        "Expected no extra blank line introduced by HTML comment block, got: {:?}",
        plain
    );
    assert!(
        !output.contains("\x1b[2m\x1b[22m"),
        "Expected no empty styled line for HTML comment block, got: {:?}",
        output
    );
}

#[test]
fn test_render_html_block_boundaries_preserve_neighboring_text() {
    let markdown = "before\n\n<div>block</div>\n\nafter";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("before") && plain.contains("<div>block</div>") && plain.contains("after"),
        "Expected html block to preserve surrounding text boundaries, got: {:?}",
        plain
    );
    assert!(
        !plain.contains("before<div>") && !plain.contains("</div>after"),
        "Expected html block to stay separated from neighboring text, got: {:?}",
        plain
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
    // Soft breaks inside a quote paragraph should render inline.
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("line one line two line three"),
        "Expected quote soft breaks to remain inline, got: {:?}",
        plain
    );

    // Quote prefix should still be present.
    let pipe_count = output.matches('│').count();
    assert!(
        pipe_count >= 1,
        "Expected at least 1 '│' prefix in output, got: {}",
        pipe_count
    );
}

#[test]
fn test_render_block_quote_softbreak_stays_inline() {
    let markdown = "> \"Good abstractions are not vague; they create a level where exact reasoning is possible\n> and concrete decisions stay clear.\"";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(200);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("where exact reasoning is possible and concrete decisions stay clear."),
        "Expected block-quote soft break to render inline as a space, got: {:?}",
        plain
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
fn test_render_list_with_blank_lines_between_items_preserves_gap() {
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

    assert!(
        second_idx >= first_idx + 2,
        "Expected blank line preserved between loose list items, got: {:?}",
        plain
    );
    assert!(
        lines[first_idx + 1].trim().is_empty(),
        "Expected explicit blank line between loose list items, got: {:?}",
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
    let plain = strip_combining_marks(&strip_ansi(&output));

    // Should contain the URL in parentheses
    assert!(
        plain.contains("(http://example.com)"),
        "Expected '(http://example.com)' in output, got: {:?}",
        plain
    );
    // Should render in markdown link form
    assert!(
        plain.contains("[link](http://example.com)"),
        "Expected '[link](http://example.com)' in output, got: {:?}",
        plain
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

#[test]
fn test_render_link_with_inline_code_label_no_duplicate_code_prefix() {
    let markdown = "[link with `code` inside](https://example.com)";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("[link with `code` inside](https://example.com)"),
        "Expected markdown-like link output, got: {:?}",
        plain
    );
    assert!(
        !plain.starts_with("`code` "),
        "Unexpected duplicated inline code prefix before link, got: {:?}",
        plain
    );
}

#[test]
fn test_render_reference_definitions_without_blank_lines_and_with_underlined_urls() {
    let markdown =
        "[id1]: https://example.com/ref \"Ref One\"\n[id2]: https://example.com/collapsed\n";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);
    let plain = strip_combining_marks(&strip_ansi(&output));
    let lines: Vec<&str> = plain.lines().collect();

    let id1_idx = lines
        .iter()
        .position(|line| line.contains("[id1]: https://example.com/ref"))
        .expect("Expected first reference definition line");
    let id2_idx = lines
        .iter()
        .position(|line| line.contains("[id2]: https://example.com/collapsed"))
        .expect("Expected second reference definition line");

    assert_eq!(
        id2_idx,
        id1_idx + 1,
        "Expected consecutive reference definition lines without blank line, got: {:?}",
        plain
    );
    assert!(
        output.contains("[id1]: \x1b[4mhttps://example.com/ref\x1b[24m \"Ref One\""),
        "Expected underlined URL in first reference definition, got: {:?}",
        output
    );
    assert!(
        output.contains("[id2]: \x1b[4mhttps://example.com/collapsed\x1b[24m"),
        "Expected underlined URL in second reference definition, got: {:?}",
        output
    );
}

#[test]
fn test_render_reference_definitions_keep_blank_line_after_section_heading() {
    let markdown = "## 18. Reference Definitions\n\n[id1]: https://example.com/ref \"Ref One\"\n[id2]: https://example.com/collapsed\n";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    let heading_idx = lines
        .iter()
        .position(|line| line.contains("## 18. Reference Definitions"))
        .expect("Expected heading line");
    let first_ref_idx = lines
        .iter()
        .position(|line| line.contains("[id1]: https://example.com/ref"))
        .expect("Expected first reference definition line");
    assert_eq!(
        first_ref_idx,
        heading_idx + 2,
        "Expected exactly one blank line after heading before definitions, got: {:?}",
        plain
    );
}

#[test]
fn test_render_wraps_before_inline_code_when_near_line_end() {
    let markdown = "Task runners only expose what the implementation anticipated. If you have `task_alpha()` and `task_beta()`, the engine can run alpha and beta.";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    assert!(
        lines.iter().any(|line| line.ends_with("If you have")),
        "Expected a wrap before inline code span near line end, got: {:?}",
        plain
    );
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("`task_alpha()` and `task_beta()`,")),
        "Expected inline-code span to start the continuation line, got: {:?}",
        plain
    );
}

#[test]
fn test_render_wraps_before_read_file_inline_code_in_tool_vocabulary_sentence() {
    let markdown = "A fixed command set gives a constrained vocabulary: `alpha_cmd()`, `beta_cmd()`,\n`gamma_cmd()`. Each command has a strict signature.";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    assert!(
        lines
            .iter()
            .any(|line| line.ends_with("`beta_cmd()`,") || line.ends_with("beta_cmd(),")),
        "Expected first line to wrap after beta_cmd() marker, got: {:?}",
        plain
    );
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("`gamma_cmd()`. Each command")),
        "Expected continuation line to start with gamma_cmd() inline code, got: {:?}",
        plain
    );
}

#[test]
fn test_render_keeps_open_paren_with_inline_code_when_wrapping() {
    let markdown = "This setup lets you pass a compact callback as a handler. The adapter\n(`_prepare_handler()`) handles the wiring.";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    assert!(
        !lines.iter().any(|line| line.ends_with(" (") || line.ends_with("(")),
        "Expected opening parenthesis to stay attached to inline code on wrap, got: {:?}",
        plain
    );
    assert!(
        plain.contains("(`_prepare_handler()`) handles the wiring."),
        "Expected parenthesized inline code phrase preserved, got: {:?}",
        plain
    );
}

/// Test image rendering - ![alt](url) should show alt text
#[test]
fn test_render_image_alt_text() {
    let markdown = "![alt text](http://example.com/image.png)";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    // Should preserve image markdown form
    assert!(
        output.contains("![alt text](http://example.com/image.png)"),
        "Expected inline image markdown form in output, got: {:?}",
        output
    );
}

#[test]
fn test_render_reference_image_preserves_markdown_form() {
    let markdown = "[img1]: https://example.com/ref-image.png\n\n![ref image][img1]";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("![ref image][img1]"),
        "Expected reference image markdown form in output, got: {:?}",
        plain
    );
}

/// Test URL auto-link rendering - <https://example.com> should render once
#[test]
fn test_render_url_auto_link() {
    let markdown = "<https://example.com>";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    let plain = strip_combining_marks(&strip_ansi(&output));
    // Should render URL once (autolink should not duplicate as url(url))
    assert!(
        plain.contains("<https://example.com>"),
        "Expected URL in output, got: {:?}",
        plain
    );
    assert_eq!(
        plain.matches("https://example.com").count(),
        1,
        "Expected URL autolink to appear once, got: {:?}",
        plain
    );
}

#[test]
fn test_render_plain_text_url_is_underlined() {
    let markdown = "Visit https://example.com/docs for details.";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);

    assert!(
        output.contains("\x1b[4mhttps://example.com/docs\x1b[24m"),
        "Expected plain-text URL to be underlined, got: {:?}",
        output
    );
}

#[test]
fn test_render_long_plain_text_url_wraps_without_losing_following_words() {
    let markdown = "See https://example.com/this/is/a/very/long/url/path/that/previously/overflowed/the/render/width and keep these trailing words visible.";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(40);
    let output = renderer.render(&events);
    let plain = strip_combining_marks(&strip_ansi(&output));
    let normalized = plain.split_whitespace().collect::<Vec<_>>().join(" ");

    assert!(
        normalized.contains("and keep these trailing words visible."),
        "Expected trailing words after long URL to remain visible, got: {:?}",
        plain
    );
    for line in plain.lines() {
        assert!(
            line.chars().count() <= 40,
            "Expected wrapped line width <= 40, got line {:?} (len={})",
            line,
            line.chars().count()
        );
    }
}

/// Test email auto-link rendering - <user@example.com> should render once
#[test]
fn test_render_email_auto_link() {
    let markdown = "<user@example.com>";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    let plain = strip_combining_marks(&strip_ansi(&output));
    // Should render email once (autolink should not duplicate as email(email))
    assert!(
        plain.contains("<user@example.com>"),
        "Expected 'user@example.com' in output, got: {:?}",
        plain
    );
    assert_eq!(
        plain.matches("user@example.com").count(),
        1,
        "Expected email autolink to appear once, got: {:?}",
        plain
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

#[test]
fn test_render_escaped_image_like_syntax_keeps_bracketed_link_form() {
    let markdown = "\\![not image](x)";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_combining_marks(&strip_ansi(&output));

    assert!(
        plain.contains("![not image](x)"),
        "Expected escaped image-like syntax to keep markdown-like form, got: {:?}",
        plain
    );
    assert!(
        !output.contains("\x1b[4m"),
        "Expected no URL underlining for escaped image-like syntax, got: {:?}",
        output
    );
}

#[test]
fn test_render_escaped_punctuation_preserves_token_separation() {
    let markdown = "Escaped punctuation: \\[ \\] \\( \\) \\.";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("[ ] ( ) ."),
        "Expected escaped punctuation tokens to stay separated by spaces, got: {:?}",
        plain
    );
}

#[test]
fn test_render_literal_escaped_brackets_in_links_section_style() {
    let markdown = "Literal brackets in text: \\[not a link\\].";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("[not a link]"),
        "Expected escaped literal brackets to render without backslashes, got: {:?}",
        plain
    );
    assert!(
        !plain.contains("\\[not a link\\]"),
        "Unexpected backslashes in literal bracket output, got: {:?}",
        plain
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
    let plain = strip_ansi(first_line);
    let trimmed = plain.trim_start();
    assert_eq!(
        trimmed.chars().count(),
        40,
        "Expected half-width horizontal rule, got: {:?}",
        output
    );
    assert_eq!(
        plain.len().saturating_sub(trimmed.len()),
        20,
        "Expected centered rule with left padding, got: {:?}",
        output
    );
    assert!(
        trimmed.chars().all(|c| c == '─'),
        "Expected Unicode line-only horizontal rule output, got: {:?}",
        output
    );
    assert!(
        first_line.contains("\x1b[2m") && first_line.contains("\x1b[0m"),
        "Expected dim styling on thematic break, got: {:?}",
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
    let plain = strip_ansi(first_line);
    let trimmed = plain.trim_start();
    assert_eq!(
        trimmed.chars().count(),
        40,
        "Expected half-width rule, got: {:?}",
        output
    );
    assert_eq!(
        plain.len().saturating_sub(trimmed.len()),
        20,
        "Expected centered rule with left padding, got: {:?}",
        output
    );
    assert!(trimmed.chars().all(|c| c == '═'));
    assert!(
        first_line.contains("\x1b[2m") && first_line.contains("\x1b[0m"),
        "Expected dim styling on thematic break, got: {:?}",
        output
    );
}

/// Test horizontal rule rendering for '___'
#[test]
fn test_render_horizontal_rule_underscores() {
    let markdown = "___";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    let first_line = output.lines().next().unwrap_or("");
    let plain = strip_ansi(first_line);
    let trimmed = plain.trim_start();
    assert_eq!(
        trimmed.chars().count(),
        40,
        "Expected half-width rule, got: {:?}",
        output
    );
    assert_eq!(
        plain.len().saturating_sub(trimmed.len()),
        20,
        "Expected centered rule with left padding, got: {:?}",
        output
    );
    assert!(trimmed.chars().all(|c| c == '┄'));
    assert!(
        first_line.contains("\x1b[2m") && first_line.contains("\x1b[0m"),
        "Expected dim styling on thematic break, got: {:?}",
        output
    );
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
        output.contains("\x1b[29m"),
        "Expected reset code \\x1b[29m, got: {:?}",
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

#[test]
fn test_render_nested_emphasis_strong_strikethrough_wraps_with_ansi() {
    let markdown = "*prefix **nested ~~verylongtokenverylongtokenverylongtoken~~ suffix** trailer*";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(32);
    renderer.set_strikethrough_fallback(false);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let normalized = plain.split_whitespace().collect::<Vec<_>>().join(" ");

    assert!(
        normalized.contains("prefix nested verylongtokenverylongtokenverylongtoken suffix trailer"),
        "Expected nested formatted text to be preserved across wrapping, got: {:?}",
        plain
    );
    assert!(
        output.contains("\x1b[3m")
            && output.contains("\x1b[1m")
            && output.contains("\x1b[9m")
            && output.lines().count() > 1,
        "Expected italics, bold, strikethrough ANSI codes with wrapped output, got: {:?}",
        output
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

#[test]
fn test_render_link_with_empty_source_autolink_branch() {
    let events = vec![Event::Link {
        text: "https://example.com".to_string(),
        url: "https://example.com".to_string(),
        source: String::new(),
    }];
    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);
    assert!(
        output.contains("<\x1b[4mhttps://example.com\x1b[24m>"),
        "Expected autolink rendering path for empty source, got: {:?}",
        output
    );
}

#[test]
fn test_render_link_with_empty_source_bracket_branch() {
    let events = vec![Event::Link {
        text: "docs".to_string(),
        url: "https://example.com/docs".to_string(),
        source: String::new(),
    }];
    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);
    assert!(
        output.contains("[docs](\x1b[4mhttps://example.com/docs\x1b[24m)"),
        "Expected bracket link rendering path for empty source, got: {:?}",
        output
    );
}

#[test]
fn test_render_table_cell_link_image_inline_code_and_html_paths() {
    let events = vec![
        Event::Start(Block::Table {
            alignments: vec![CellAlignment::Left],
        }),
        Event::Start(Block::TableHead),
        Event::Start(Block::TableRow),
        Event::Start(Block::TableCell),
        Event::Text("Col".to_string()),
        Event::End(Block::TableCell),
        Event::End(Block::TableRow),
        Event::End(Block::TableHead),
        Event::Start(Block::TableRow),
        Event::Start(Block::TableCell),
        Event::Link {
            text: "label".to_string(),
            url: "https://e.test".to_string(),
            source: String::new(),
        },
        Event::Text(" ".to_string()),
        Event::Image {
            alt: "pic".to_string(),
            url: "https://img.test".to_string(),
            source: String::new(),
        },
        Event::Text(" ".to_string()),
        Event::InlineCode("`x`".to_string()),
        Event::Text(" ".to_string()),
        Event::RawHtml("<b>h</b>".to_string()),
        Event::End(Block::TableCell),
        Event::End(Block::TableRow),
        Event::End(Block::Table {
            alignments: vec![CellAlignment::Left],
        }),
    ];

    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let compact = plain.replace(' ', "");

    assert!(
        compact.contains("[label](https://e.test)[pic]```x```<b>h</b>")
            || compact.contains("[label](https://e.test)[pic]```x```")
            || compact.contains("[label](https://e.test)[pic]"),
        "Expected table-cell inline rendering branches to execute, got: {:?}",
        plain
    );
}

#[test]
fn test_render_inline_code_inside_strikethrough_fallback() {
    let markdown = "~~`code`~~";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    renderer.set_strikethrough_fallback(true);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("\u{0336}"),
        "Expected combining-strikethrough fallback to affect inline code path, got: {:?}",
        plain
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
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("- [ ] todo item"),
        "Expected unchecked task marker '[ ]', got: {:?}",
        plain
    );
    assert!(
        plain.contains("todo item"),
        "Expected task content, got: {:?}",
        plain
    );
}

/// Test checked task list rendering
#[test]
fn test_render_task_list_checked() {
    let markdown = "- [x] done item";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("[✔]"),
        "Expected checked task marker '[✔]', got: {:?}",
        plain
    );
    assert!(
        plain.contains("done item"),
        "Expected task content, got: {:?}",
        plain
    );
}

/// Test mixed task list rendering
#[test]
fn test_render_task_list_mixed() {
    let markdown = "- [ ] first\n- [x] second\n- [X] third";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("- [ ] first"),
        "Expected unchecked task marker '[ ]', got: {:?}",
        plain
    );
    let checked_count = plain.matches("[✔]").count();
    assert!(
        checked_count >= 2,
        "Expected at least two checked markers '[✔]' (for x/X), got {checked_count} in {:?}",
        plain
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
    renderer.set_flatten_wide_tables(true);
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

#[test]
fn test_render_table_fallback_uses_structured_row_layout() {
    let markdown = "| | M0 | M1 |\n| --- | --- | --- |\n| Scope | manages one very long area description here | drives another long scope description there |";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(50);
    renderer.set_flatten_wide_tables(true);
    let output = renderer.render(&events);

    assert!(
        !output.contains("│"),
        "Expected fallback mode (no box borders), got: {:?}",
        output
    );
    assert!(
        output.contains("Scope"),
        "Expected row title in structured fallback, got: {:?}",
        output
    );
    assert!(
        output.contains("M0:") && output.contains("M1:"),
        "Expected header-keyed fallback entries, got: {:?}",
        output
    );
}

#[test]
fn test_render_wide_table_mode_is_default() {
    let markdown = "| VeryLongColumnHeader | AnotherVeryLongColumnHeader |\n| --- | --- |\n| ExtremelyLongCellValue | AnotherExtremelyLongCellValue |";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(20);
    let output = renderer.render(&events);

    assert!(
        output.contains("┌") && output.contains("│") && output.contains("┘"),
        "Expected bordered table when wide-table flattening is disabled, got: {:?}",
        output
    );
    let max_visible_width = output
        .lines()
        .map(|line| strip_ansi(line).chars().count())
        .max()
        .unwrap_or(0);
    assert!(
        max_visible_width <= 20,
        "Expected rendered table lines to fit width=20, got max width {} in {:?}",
        max_visible_width,
        output
    );
}

#[test]
fn test_render_table_with_link_uses_visible_width_not_ansi_bytes() {
    let markdown = "| Link | Text |\n| --- | --- |\n| [test](https://toto.html) | plain |";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(45);
    let output = renderer.render(&events);

    assert!(
        output.contains("┌") && output.contains("│") && output.contains("┘"),
        "Expected bordered table (no fallback) when visible width fits, got: {:?}",
        output
    );
}

#[test]
fn test_render_table_with_emphasis_does_not_emit_stray_ansi_line() {
    let markdown = "| A | B |\n| --- | --- |\n| **x** | y |\n\nAfter";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        !output.contains("\x1b[1m\x1b[22m\n\nAfter"),
        "Expected no standalone bold on/off line after table, got: {:?}",
        output
    );
    assert!(
        output.contains("After"),
        "Expected trailing paragraph to render, got: {:?}",
        output
    );
}

#[test]
fn test_render_table_fallback_with_emphasis_does_not_emit_stray_ansi_line() {
    let markdown = "| VeryLongColumnHeader | AnotherVeryLongColumnHeader |\n| --- | --- |\n| **x** | y |\n\nAfter";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(20);
    renderer.set_flatten_wide_tables(true);
    let output = renderer.render(&events);

    assert!(
        !output.contains("\x1b[1m\x1b[22m\n\nAfter"),
        "Expected no standalone bold on/off line after fallback table, got: {:?}",
        output
    );
    assert!(
        output.contains("After"),
        "Expected trailing paragraph to render, got: {:?}",
        output
    );
}

#[test]
fn test_render_table_cell_preserves_inline_emphasis() {
    let markdown = "| A | B |\n| --- | --- |\n| **x** and *y* | z |";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);

    assert!(
        output.contains("\x1b[1mx\x1b[22m"),
        "Expected bold emphasis to be preserved in table cell, got: {:?}",
        output
    );
    assert!(
        output.contains("\x1b[3m") && output.contains("y\x1b[23m"),
        "Expected italic emphasis to be preserved in table cell, got: {:?}",
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
fn test_render_preserves_two_blank_lines_between_paragraphs() {
    let markdown = "Text before.\n\n\nText after two blank lines.";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("Text before.\n\n\nText after two blank lines."),
        "Expected two blank lines preserved between paragraphs, got: {:?}",
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
fn test_render_inline_code_surrounded_by_spaces_variant() {
    let markdown = "Use `` `surrounded by spaces` `` style delimiters.";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);

    assert!(
        plain.contains("`` `surrounded by spaces` ``"),
        "Expected spaced two-backtick serialization preserved, got: {:?}",
        plain
    );
}

#[test]
fn test_render_blockquote_heading_keeps_quote_prefix() {
    let markdown = "> ## Quoted";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    assert!(
        lines.iter().any(|line| line.starts_with("│ ## Quoted")),
        "Expected quoted heading line, got: {:?}",
        plain
    );
    assert!(
        !output.contains("\x1b[4m") && !output.contains("\x1b[24m"),
        "Expected no ANSI underline within quoted heading, got: {:?}",
        output
    );
}

#[test]
fn test_render_blockquote_paragraph_to_heading_has_no_extra_blank_line_without_source_gap() {
    let markdown = "> Quote with heading\n> ## Quoted heading";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    let quote_idx = lines
        .iter()
        .position(|line| line.trim() == "│ Quote with heading")
        .expect("Expected first quote line");
    let heading_idx = lines
        .iter()
        .position(|line| line.trim() == "│ ## Quoted heading")
        .expect("Expected quoted heading line");
    assert_eq!(
        heading_idx,
        quote_idx + 1,
        "Unexpected blank line between quote paragraph and heading: {:?}",
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
        idx > 0 && lines[idx - 1].trim().is_empty(),
        "Expected blank line before second paragraph in same list item, got: {:?}",
        plain
    );
    assert!(
        lines[idx].starts_with("  "),
        "Expected second paragraph to keep list continuation indent, got: {:?}",
        plain
    );
}

#[test]
fn test_render_ordered_list_preserves_source_markers() {
    let markdown = "3. starts at three\n4. next item";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    assert!(
        lines
            .iter()
            .any(|line| line.trim_start().starts_with("3. starts at three")),
        "Expected first ordered item to keep source marker 3., got: {:?}",
        plain
    );
    assert!(
        lines
            .iter()
            .any(|line| line.trim_start().starts_with("4. next item")),
        "Expected second ordered item to keep source marker 4., got: {:?}",
        plain
    );
}

#[test]
fn test_render_ordered_list_preserves_blank_lines_between_items() {
    let markdown = "3. three\n\n4. next item\n";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(80);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    let idx_three = lines
        .iter()
        .position(|line| line.trim_start().starts_with("3. three"))
        .expect("Expected first ordered item");
    let idx_next = lines
        .iter()
        .position(|line| line.trim_start().starts_with("4. next item"))
        .expect("Expected second ordered item");
    assert!(
        idx_next >= idx_three + 2,
        "Expected at least one blank line between ordered items, got: {:?}",
        plain
    );
    assert!(
        lines[idx_three + 1].trim().is_empty(),
        "Expected explicit blank line between ordered items, got: {:?}",
        plain
    );
}

#[test]
fn test_render_mixed_list_content_preserves_softbreak_line_in_item() {
    let markdown = "- item with paragraph continuation\n  still same list item paragraph";
    let events: Vec<Event> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(120);
    let output = renderer.render(&events);
    let plain = strip_ansi(&output);
    let lines: Vec<&str> = plain.lines().collect();

    let first_idx = lines
        .iter()
        .position(|line| {
            line.trim_start()
                .starts_with("- item with paragraph continuation")
        })
        .expect("Expected first list line");
    let second_idx = lines
        .iter()
        .position(|line| {
            line.trim_start()
                .starts_with("still same list item paragraph")
        })
        .expect("Expected continuation list line");
    assert_eq!(
        second_idx,
        first_idx + 1,
        "Expected soft break in list item to remain a new continuation line, got: {:?}",
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
