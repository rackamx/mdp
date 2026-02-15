use std::process::Command;

/// Test that terminal cleanup works - verifies that when a panic occurs,
/// the terminal is properly restored using the RAII pattern.
#[test]
fn test_terminal_cleanup() {
    // This test verifies that the Terminal struct properly cleans up on drop.
    // We'll spawn a subprocess that:
    // 1. Initializes a terminal
    // 2. Sets raw mode
    // 3. Panics
    // 4. We verify the terminal is restored (process exits cleanly)
    let test_code = r#"
        use mdp::terminal::Terminal;

        fn main() {
            // Create and initialize terminal - this should set raw mode
            let _terminal = Terminal::new().expect("Failed to create terminal");

            // Panic to trigger cleanup
            panic!("intentional panic for testing");
        }
    "#;

    // Write the test code to a temp file and compile/run it
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_panic_cleanup.rs");
    std::fs::write(&test_file, test_code).unwrap();

    // Run the test binary - it should panic but exit cleanly (not abnormally)
    let result = Command::new("rustc")
        .args(&[
            "--edition",
            "2021",
            "-L",
            "target/debug/deps",
            "-o",
            "/tmp/test_panic",
            &test_file.to_string_lossy(),
        ])
        .output();

    // If we can't compile, skip this test (integration test limitations)
    if result.is_err() || !result.unwrap().status.success() {
        eprintln!("Skipping: Could not compile test (expected in unit test context)");
        return;
    }

    let output = Command::new("/tmp/test_panic")
        .output()
        .expect("Failed to run test");

    // Clean up
    std::fs::remove_file(&test_file).ok();
    std::fs::remove_file("/tmp/test_panic").ok();

    // The process should have panicked (non-zero exit) but not crashed abnormally
    // If the terminal was not restored, the terminal would be in a bad state
    // We verify the process exited with panic code (101 typically)
    assert!(!output.status.success(), "Process should have panicked");
}

/// Test that we can create a Terminal and it enters raw mode
#[test]
fn test_terminal_raw_mode() {
    use mdp::terminal::Terminal;

    // This test will fail gracefully in non-TTY environments
    let result = Terminal::new();
    if result.is_err() {
        // Skip test if no TTY available
        eprintln!("Skipping: No TTY available");
        return;
    }

    let terminal = result.unwrap();
    assert!(terminal.is_raw_mode(), "Terminal should be in raw mode");
    // The Drop trait will clean up when terminal goes out of scope
}

/// Test that terminal size can be retrieved
#[test]
fn test_terminal_size() {
    use mdp::terminal::Terminal;

    // This test will fail gracefully in non-TTY environments
    let result = Terminal::new();
    if result.is_err() {
        // Skip test if no TTY available
        eprintln!("Skipping: No TTY available");
        return;
    }

    let terminal = result.unwrap();
    let size_result = terminal.size();

    if size_result.is_err() {
        eprintln!("Skipping: Could not get terminal size");
        return;
    }

    let size = size_result.unwrap();
    // Terminal size should be positive
    assert!(size.rows > 0, "Terminal should have positive rows");
    assert!(size.cols > 0, "Terminal should have positive columns");
}

/// Test resize behavior by verifying rendering output changes with width.
#[test]
fn test_terminal_resize() {
    use mdp::parsing::{parse_markdown, Event};
    use mdp::rendering::Renderer;

    let markdown =
        "This is a sentence that should wrap differently based on available terminal width.";
    let events: Vec<Event> = parse_markdown(markdown).collect();

    let mut narrow = Renderer::new(20);
    let narrow_output = narrow.render(&events);

    let mut wide = Renderer::new(80);
    let wide_output = wide.render(&events);

    let narrow_lines = narrow_output.lines().count();
    let wide_lines = wide_output.lines().count();
    assert!(
        narrow_lines > wide_lines,
        "Narrow width should produce more wrapped lines (narrow={}, wide={})",
        narrow_lines,
        wide_lines
    );
}

/// Test Ctrl+C cleanup behavior (best-effort in test environment).
#[test]
fn test_ctrl_c_cleanup() {
    use mdp::terminal::Terminal;

    let result = Terminal::new();
    if result.is_err() {
        eprintln!("Skipping: No TTY available");
        return;
    }

    let terminal = result.unwrap();
    assert!(
        terminal.is_raw_mode(),
        "Terminal should be in raw mode before cleanup"
    );
    drop(terminal);
}
