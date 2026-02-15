use std::fs;
use std::process::Command;

#[test]
fn test_read_file_contents() {
    // Create a temporary markdown file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_markdown.md");
    let content = "# Hello World\n\nThis is a test file.";
    fs::write(&test_file, content).unwrap();

    // Run the mdless command with the file
    let output = Command::new(env!("CARGO_BIN_EXE_mdless"))
        .arg(&test_file)
        .output()
        .expect("Failed to execute mdless");

    // Clean up
    fs::remove_file(&test_file).ok();

    // Verify the output contains the content (we're not testing rendering yet, just loading)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello World") || stdout.contains("This is a test file"),
            "Output should contain file content, got: {}", stdout);
}

#[test]
fn test_file_not_found() {
    // Run mdless with a non-existent file
    let output = Command::new(env!("CARGO_BIN_EXE_mdless"))
        .arg("/nonexistent/file.md")
        .output()
        .expect("Failed to execute mdless");

    // Should fail with non-zero exit code
    assert!(!output.status.success(), "Command should fail for non-existent file");
}

#[test]
fn test_no_argument_prints_usage() {
    // Run mdless without any arguments
    let output = Command::new(env!("CARGO_BIN_EXE_mdless"))
        .output()
        .expect("Failed to execute mdless");

    // The output should contain usage information
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(combined.contains("Usage") || combined.contains("usage"),
            "Output should contain usage message, got: {}", combined);
}
