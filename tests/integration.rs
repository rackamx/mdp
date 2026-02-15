use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

use mdp::pager::{Pager, PagerConfig};

#[test]
fn test_full_render_sample_snapshot() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("mdp_integration_sample.md");
    fs::write(&test_file, "# Snapshot Title\n\nPlain text.\n").expect("write sample file");

    let output = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg(&test_file)
        .output()
        .expect("run mdp");

    fs::remove_file(&test_file).ok();

    assert!(output.status.success(), "mdp should exit successfully");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = "\x1b[1m# Snapshot Title\x1b[22m\n\nPlain text.\n";
    assert_eq!(stdout, expected, "rendered output snapshot mismatch");
}

#[test]
fn test_stdin_render_snapshot() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn mdp");

    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(b"**bold** text\n")
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait for mdp");
    assert!(output.status.success(), "mdp should exit successfully");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = "\x1b[1mbold\x1b[22m text\n";
    assert_eq!(stdout, expected, "stdin output snapshot mismatch");
}

#[test]
fn test_navigation_sample() {
    let lines: Vec<String> = (1..=60).map(|i| format!("Line {i}")).collect();
    let config = PagerConfig {
        page_size: 10,
        ..Default::default()
    };
    let mut pager = Pager::new(config, lines);

    assert_eq!(
        pager.visible_lines().first().map(String::as_str),
        Some("Line 1")
    );

    pager.page_down();
    assert_eq!(
        pager.visible_lines().first().map(String::as_str),
        Some("Line 11")
    );

    pager.scroll_down();
    assert_eq!(
        pager.visible_lines().first().map(String::as_str),
        Some("Line 12")
    );

    pager.go_to_end();
    assert_eq!(
        pager.visible_lines().last().map(String::as_str),
        Some("Line 60")
    );

    pager.go_to_beginning();
    assert_eq!(
        pager.visible_lines().first().map(String::as_str),
        Some("Line 1")
    );
}
