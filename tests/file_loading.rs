use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_read_file_contents() {
    // Create a temporary markdown file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_markdown.md");
    let content = "# Hello World\n\nThis is a test file.";
    fs::write(&test_file, content).unwrap();

    // Run the mdp command with the file
    let output = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg(&test_file)
        .output()
        .expect("Failed to execute mdp");

    // Clean up
    fs::remove_file(&test_file).ok();

    // Verify output contains rendered heading + body content.
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hello World") && stdout.contains("This is a test file"),
        "Output should contain rendered file content, got: {}",
        stdout
    );
    assert!(
        stdout.contains("\x1b[1m"),
        "Output should contain ANSI formatting for markdown, got: {}",
        stdout
    );
}

#[test]
fn test_file_not_found() {
    // Run mdp with a non-existent file
    let output = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg("/nonexistent/file.md")
        .output()
        .expect("Failed to execute mdp");

    // Should fail with non-zero exit code
    assert!(
        !output.status.success(),
        "Command should fail for non-existent file"
    );
}

#[test]
fn test_no_argument_reads_stdin() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute mdp");

    let input = "# Piped title\n\nBody from stdin.\n";
    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("Failed writing to stdin");

    let output = child.wait_with_output().expect("Failed waiting on mdp");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Piped title") && stdout.contains("Body from stdin."),
        "Output should include stdin content, got: {}",
        stdout
    );
}

#[test]
fn test_stdin_with_dash() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute mdp");

    let input = "From dash stdin\n";
    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("Failed writing to stdin");

    let output = child.wait_with_output().expect("Failed waiting on mdp");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("From dash stdin"),
        "Output should include stdin content with '-' arg, got: {}",
        stdout
    );
}

#[test]
fn test_empty_file() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("mdp_empty.md");
    fs::write(&test_file, "").expect("write empty file");

    let output = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg(&test_file)
        .output()
        .expect("run mdp");

    fs::remove_file(&test_file).ok();

    assert!(
        output.status.success(),
        "Empty file should be handled gracefully"
    );
}

#[test]
fn test_width_option() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("mdp_width.md");
    let content = "word1 word2 word3 word4 word5 word6 word7 word8 word9 word10";
    fs::write(&test_file, content).expect("write width file");

    let output = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg("--width")
        .arg("20")
        .arg(&test_file)
        .output()
        .expect("run mdp with width");
    fs::remove_file(&test_file).ok();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let max_len = stdout.lines().map(str::len).max().unwrap_or(0);
    assert!(
        max_len <= 20,
        "Expected wrapped output <= 20 columns with --width, got {max_len}: {stdout}"
    );
}

#[test]
fn test_no_flatten_wide_tables_option_keeps_table_borders() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("mdp_no_flatten_wide_table.md");
    let content = "| VeryLongColumnHeader | AnotherVeryLongColumnHeader |\n| --- | --- |\n| ExtremelyLongCellValue | AnotherExtremelyLongCellValue |";
    fs::write(&test_file, content).expect("write table file");

    let output = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg("--width")
        .arg("20")
        .arg("--no-flatten-wide-tables")
        .arg(&test_file)
        .output()
        .expect("run mdp with --no-flatten-wide-tables");
    fs::remove_file(&test_file).ok();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("┌") && stdout.contains("│") && stdout.contains("┘"),
        "Expected bordered table with --no-flatten-wide-tables, got: {stdout}"
    );
}

#[test]
fn test_help_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg("--help")
        .output()
        .expect("run mdp --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Usage") || stdout.contains("USAGE"),
        "Expected help output to contain usage, got: {stdout}"
    );
}

#[test]
fn test_exit_code_success() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("mdp_ok.md");
    fs::write(&test_file, "ok\n").expect("write ok file");

    let output = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg(&test_file)
        .output()
        .expect("run mdp");
    fs::remove_file(&test_file).ok();

    assert_eq!(
        output.status.code(),
        Some(0),
        "Expected success exit code 0"
    );
}

#[test]
fn test_exit_code_file_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg("/nonexistent/file.md")
        .output()
        .expect("run mdp");
    assert_eq!(
        output.status.code(),
        Some(1),
        "Expected file read error exit code 1"
    );
}

#[test]
fn test_directory_path_is_reported_as_file_read_error() {
    let temp_dir = std::env::temp_dir();
    let output = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg(&temp_dir)
        .output()
        .expect("run mdp");
    assert_eq!(
        output.status.code(),
        Some(1),
        "Expected file read error exit code for directory path"
    );
}

#[test]
fn test_benchmark_mode_reports_expected_fields_and_clamps_zero_iters() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg("--benchmark")
        .arg("--bench-iters")
        .arg("0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute mdp benchmark");

    let input = "# Bench\n\n- one\n- two\n";
    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("Failed writing benchmark stdin");

    let output = child.wait_with_output().expect("Failed waiting on mdp");
    assert_eq!(
        output.status.code(),
        Some(0),
        "Benchmark mode should exit successfully"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Benchmark (mdp)")
            && stdout.contains("Iterations: 1")
            && stdout.contains("Input bytes:")
            && stdout.contains("Parse+Render avg:")
            && stdout.contains("Search hits:"),
        "Unexpected benchmark output: {stdout}"
    );
}

#[test]
fn test_binary_file_rejected() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("mdp_binary.bin");
    fs::write(&test_file, [0_u8, 159, 146, 150]).expect("write binary file");

    let output = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg(&test_file)
        .output()
        .expect("run mdp");

    fs::remove_file(&test_file).ok();

    assert!(
        !output.status.success(),
        "Binary file should be rejected with non-zero exit"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Binary file detected"),
        "Expected binary detection error, got: {stderr}"
    );
}

#[test]
fn test_text_file_accepted() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("mdp_text.txt");
    fs::write(&test_file, "plain text file\n").expect("write text file");

    let output = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg(&test_file)
        .output()
        .expect("run mdp");

    fs::remove_file(&test_file).ok();

    assert!(output.status.success(), "Text file should be accepted");
}

#[test]
fn test_non_utf8_text_like_file_accepted() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("mdp_non_utf8.bin");
    fs::write(&test_file, b"hello \xFF world\n").expect("write bytes");

    let output = Command::new(env!("CARGO_BIN_EXE_mdp"))
        .arg(&test_file)
        .output()
        .expect("run mdp");
    fs::remove_file(&test_file).ok();

    assert!(
        output.status.success(),
        "Non-UTF8 text-like file should be rendered lossily"
    );
}
