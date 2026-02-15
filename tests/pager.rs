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
