use mdless::pager::{Pager, PagerConfig};

/// Test page down - verify content scrolls down one page
#[test]
fn test_page_down() {
    // Create a pager with test content (more than one page)
    let lines: Vec<String> = (1..=50).map(|i| format!("Line {}", i)).collect();
    let mut pager = Pager::new(PagerConfig::default(), lines);

    // Initial state should show first page (lines 1-24 typically)
    let visible = pager.visible_lines();
    assert!(!visible.is_empty());
    let first_line = visible[0].clone();

    // Page down
    pager.page_down();

    // After page down, content should have scrolled
    let visible_after = pager.visible_lines();
    assert!(!visible_after.is_empty());

    // The first visible line should be different (later in the content)
    assert_ne!(first_line, visible_after[0]);
}

/// Test page up - verify content scrolls up one page
#[test]
fn test_page_up() {
    // Create a pager with test content
    let lines: Vec<String> = (1..=50).map(|i| format!("Line {}", i)).collect();
    let mut pager = Pager::new(PagerConfig::default(), lines);

    // Page down first to have content to scroll up
    pager.page_down();
    pager.page_down();

    let visible_before = pager.visible_lines();
    let first_visible_before = visible_before[0].clone();

    // Page up
    pager.page_up();

    // After page up, content should have scrolled back
    let visible_after = pager.visible_lines();
    let first_visible_after = visible_after[0].clone();

    // Should have scrolled back (first visible line should be earlier)
    assert_ne!(first_visible_before, first_visible_after);
}

/// Test progress indicator shows at bottom when there's more content
#[test]
fn test_progress_indicator() {
    // Create a pager with content that exceeds one page
    let lines: Vec<String> = (1..=50).map(|i| format!("Line {}", i)).collect();
    let config = PagerConfig {
        page_size: 24,
        ..Default::default()
    };
    let pager = Pager::new(config, lines);

    // Should show progress indicator when there's more content below
    let indicator = pager.progress_indicator();
    assert!(indicator.is_some(), "Should show progress indicator when more content below");

    // The indicator should contain "More" or similar text
    let indicator_text = indicator.unwrap();
    assert!(
        indicator_text.contains("More") || indicator_text.contains("more"),
        "Indicator should contain 'More', got: {}",
        indicator_text
    );
}

/// Test progress indicator shows line count
#[test]
fn test_progress_indicator_line_count() {
    // Create a pager with content
    let lines: Vec<String> = (1..=30).map(|i| format!("Line {}", i)).collect();
    let config = PagerConfig {
        page_size: 10,
        ..Default::default()
    };
    let pager = Pager::new(config, lines);

    let indicator = pager.progress_indicator();
    assert!(indicator.is_some());

    // Should show something like "10/30" or similar
    let indicator_text = indicator.unwrap();
    assert!(
        indicator_text.contains("10") && indicator_text.contains("30"),
        "Indicator should show position/total, got: {}",
        indicator_text
    );
}

/// Test no progress indicator when at end of content
#[test]
fn test_no_progress_indicator_at_end() {
    // Create a pager with small content that fits on one page
    let lines: Vec<String> = (1..=5).map(|i| format!("Line {}", i)).collect();
    let config = PagerConfig {
        page_size: 24,
        ..Default::default()
    };
    let pager = Pager::new(config, lines);

    // At end of content, should not show progress indicator
    let indicator = pager.progress_indicator();
    // Either None or shows "END"
    if let Some(text) = indicator {
        assert!(
            text.contains("END") || text.contains("All"),
            "Should show END when at bottom, got: {}",
            text
        );
    }
}

/// Test that page down at end of content doesn't cause issues
#[test]
fn test_page_down_at_end() {
    // Create a pager with small content
    let lines: Vec<String> = (1..=5).map(|i| format!("Line {}", i)).collect();
    let config = PagerConfig {
        page_size: 24,
        ..Default::default()
    };
    let mut pager = Pager::new(config, lines);

    // Try to page down at the end - should not panic
    pager.page_down();
    pager.page_down();

    // Should still show content
    let visible = pager.visible_lines();
    assert!(!visible.is_empty());
}

/// Test scroll down by one line
#[test]
fn test_scroll_down_line() {
    // Create a pager with test content (more than one page)
    let lines: Vec<String> = (1..=50).map(|i| format!("Line {}", i)).collect();
    let mut pager = Pager::new(PagerConfig::default(), lines);

    // Initial state should show first page (lines 1-24 typically)
    let visible = pager.visible_lines();
    let first_line = visible[0].clone();

    // Scroll down by one line
    pager.scroll_down();
    let visible_after = pager.visible_lines();

    // Should have scrolled by exactly one line
    assert_eq!(visible_after[0], "Line 2");
    assert_ne!(first_line, visible_after[0]);

    // Scroll down another line
    pager.scroll_down();
    let visible_after_two = pager.visible_lines();
    assert_eq!(visible_after_two[0], "Line 3");
}

/// Test scroll up by one line
#[test]
fn test_scroll_up_line() {
    // Create a pager with test content
    let lines: Vec<String> = (1..=50).map(|i| format!("Line {}", i)).collect();
    let mut pager = Pager::new(PagerConfig::default(), lines);

    // Move down a few lines first
    pager.scroll_down();
    pager.scroll_down();
    pager.scroll_down();

    let visible_before = pager.visible_lines();
    let first_visible_before = visible_before[0].clone();
    assert_eq!(first_visible_before, "Line 4");

    // Scroll up by one line
    pager.scroll_up();
    let visible_after = pager.visible_lines();

    // Should have scrolled up by exactly one line
    assert_eq!(visible_after[0], "Line 3");
}

/// Test scroll to end of content
#[test]
fn test_scroll_to_end() {
    // Create a pager with test content
    let lines: Vec<String> = (1..=50).map(|i| format!("Line {}", i)).collect();
    let mut pager = Pager::new(PagerConfig::default(), lines);

    // Go to end
    pager.go_to_end();

    // Should show last lines (within page size)
    let visible = pager.visible_lines();
    let last_line = visible.last().unwrap().clone();

    // The last visible line should be near the end of content
    assert!(last_line.contains("Line 50") || last_line.contains("Line 49"));

    // has_more_below should be false at end
    assert!(!pager.has_more_below());
}

/// Test scroll to beginning of content
#[test]
fn test_scroll_to_beginning() {
    // Create a pager with test content
    let lines: Vec<String> = (1..=50).map(|i| format!("Line {}", i)).collect();
    let mut pager = Pager::new(PagerConfig::default(), lines);

    // First move away from the beginning
    pager.go_to_end();
    assert!(pager.scroll_position() > 0);

    // Go to beginning
    pager.go_to_beginning();

    // Should be back at start
    assert_eq!(pager.scroll_position(), 0);

    // First visible line should be Line 1
    let visible = pager.visible_lines();
    assert_eq!(visible[0], "Line 1");

    // has_more_above should be false at beginning
    assert!(!pager.has_more_above());
}

/// Test search forward - search for pattern, verify match found
#[test]
fn test_search_forward() {
    // Create a pager with test content containing a specific pattern
    let lines: Vec<String> = vec![
        "First line".to_string(),
        "Hello world".to_string(),
        "Another line".to_string(),
        "Hello again".to_string(),
    ];
    let mut pager = Pager::new(PagerConfig::default(), lines);

    // Search for "Hello"
    let result = pager.search("Hello");

    // Should find matches
    assert!(result.is_some(), "Should find matches for 'Hello'");

    // Should be at the first match position
    let (line_idx, _) = result.unwrap();
    assert_eq!(line_idx, 1, "First match should be at line 1");
}

/// Test search highlight - verify match is highlighted with bold
#[test]
fn test_search_highlight() {
    // Create a pager with test content
    let lines: Vec<String> = vec![
        "First line".to_string(),
        "Hello world".to_string(),
        "Another line".to_string(),
    ];
    let mut pager = Pager::new(PagerConfig::default(), lines);

    // Search for "Hello"
    pager.search("Hello");

    // Get visible lines with highlighting
    let visible = pager.visible_lines_with_highlight();

    // At least one line should contain the bold escape sequence
    let has_highlight = visible.iter().any(|line| line.contains("\x1b[1m"));
    assert!(has_highlight, "Search match should be highlighted with bold");

    // The highlighted line should contain the search pattern
    let highlighted_line = visible.iter().find(|line| line.contains("Hello")).unwrap();
    assert!(
        highlighted_line.contains("\x1b[1m") && highlighted_line.contains("\x1b[0m"),
        "Highlight should use ANSI bold codes: {}",
        highlighted_line
    );
}

/// Test search no match - verify "Pattern not found" message
#[test]
fn test_search_no_match() {
    // Create a pager with test content
    let lines: Vec<String> = vec![
        "First line".to_string(),
        "Hello world".to_string(),
        "Another line".to_string(),
    ];
    let mut pager = Pager::new(PagerConfig::default(), lines);

    // Search for a pattern that doesn't exist
    let result = pager.search("nonexistent");

    // Should return None (no match found)
    assert!(result.is_none(), "Should return None when pattern not found");

    // The search status message should indicate no match
    let status = pager.search_status_message();
    assert!(
        status.contains("not found") || status.contains("Pattern"),
        "Should show 'Pattern not found' message, got: {}",
        status
    );
}

/// Test search next - verify 'n' key finds next match
#[test]
fn test_search_next() {
    // Create a pager with test content with multiple occurrences
    let lines: Vec<String> = vec![
        "First line".to_string(),
        "Hello world".to_string(),
        "Another line".to_string(),
        "Hello again".to_string(),
        "Goodbye".to_string(),
    ];
    let mut pager = Pager::new(PagerConfig::default(), lines);

    // Search for "Hello"
    let first_result = pager.search("Hello");
    assert!(first_result.is_some());
    let (first_line, _) = first_result.unwrap();
    assert_eq!(first_line, 1, "First match should be at line 1");

    // Search for next match (n key)
    pager.search_next();

    // After search_next, the current match index should be 1 (second match)
    // and the visible lines should show both "Hello world" and "Hello again"
    // but the current match is the second one
    let visible = pager.visible_lines_with_highlight();

    // There should be 2 highlighted lines (both "Hello" occurrences)
    let highlighted_count = visible.iter().filter(|line| line.contains("\x1b[1m")).count();
    assert_eq!(highlighted_count, 2, "Should have 2 highlighted lines with 'Hello'");

    // The search status should show we're on match 2 of 2
    let status = pager.search_status_message();
    assert!(
        status.contains("2/2") || status.contains("(2/2)"),
        "Should show current match position (2/2), got: {}",
        status
    );
}

/// Test search previous - verify 'N' key finds previous match
#[test]
fn test_search_previous() {
    // Create a pager with test content with multiple occurrences
    let lines: Vec<String> = vec![
        "First line".to_string(),
        "Hello world".to_string(),
        "Another line".to_string(),
        "Hello again".to_string(),
        "Hello final".to_string(),
    ];
    let mut pager = Pager::new(PagerConfig::default(), lines);

    // Search for "Hello" - starts at first match
    pager.search("Hello");

    // Go to the last match
    for _ in 0..10 {
        pager.search_next();
    }

    // Now search for previous match (N key)
    pager.search_previous();

    // Should be on the previous match
    let visible = pager.visible_lines_with_highlight();
    // At least one line should still contain the match highlighted
    let has_highlight = visible.iter().any(|line| line.contains("Hello") && line.contains("\x1b[1m"));
    assert!(has_highlight, "Should still have highlighted match after search_previous");
}
