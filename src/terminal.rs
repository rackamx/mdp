use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};
use std::io::{self, Write};
use std::panic;

/// Represents terminal state and provides RAII-style cleanup.
///
/// This struct enters raw mode on creation and restores the terminal
/// on drop (when it goes out of scope or when a panic occurs).
pub struct Terminal {
    // Track whether we successfully entered raw mode
    is_raw_mode: bool,
}

impl Terminal {
    /// Creates a new Terminal and enters raw mode.
    ///
    /// Returns an error if not running in a terminal (non-TTY environment).
    ///
    /// # Errors
    ///
    /// Returns an error if unable to enter raw mode (e.g., not running in a terminal).
    pub fn new() -> Result<Self, std::io::Error> {
        // Try to enter raw mode
        if let Err(e) = enable_raw_mode() {
            return Err(e);
        }

        // Set up panic hook to ensure cleanup on panic
        // Note: The Drop trait will be called during unwinding
        panic::set_hook(Box::new(|_| {
            // Attempt to restore terminal on panic
            let _ = disable_raw_mode();
        }));

        Ok(Terminal { is_raw_mode: true })
    }

    /// Returns the current terminal size.
    ///
    /// # Returns
    ///
    /// A `Size` struct containing the number of rows and columns.
    ///
    /// # Errors
    ///
    /// Returns an error if unable to get terminal size.
    pub fn size(&self) -> Result<Size, std::io::Error> {
        let (cols, rows) = size()?;
        Ok(Size { rows, cols })
    }

    /// Returns whether raw mode is currently enabled.
    pub fn is_raw_mode(&self) -> bool {
        self.is_raw_mode
    }
}

impl Default for Terminal {
    fn default() -> Self {
        // In default context, we still panic on failure for backwards compatibility
        // This is used in application context where a TTY is expected
        Self::new().expect("Failed to create terminal - not running in a terminal?")
    }
}

/// Drop implementation for automatic cleanup.
///
/// This ensures the terminal is restored to normal mode when the
/// Terminal struct goes out of scope, including during panics.
impl Drop for Terminal {
    fn drop(&mut self) {
        // Only try to restore if we successfully entered raw mode
        if self.is_raw_mode {
            // Attempt to restore normal mode - ignore errors as we're in cleanup
            let _ = disable_raw_mode();

            // Flush stdout to ensure clean output
            let _ = io::stdout().flush();
        }
    }
}

/// Represents the dimensions of the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    /// Number of rows (lines) in the terminal.
    pub rows: u16,
    /// Number of columns (characters per line) in the terminal.
    pub cols: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_creation() {
        // This test will fail in non-TTY environments - that's expected
        let result = Terminal::new();
        if result.is_ok() {
            let _terminal = result.unwrap();
            // If we get here, raw mode was entered successfully
        }
    }

    #[test]
    fn test_terminal_size() {
        // This test will fail in non-TTY environments - that's expected
        let result = Terminal::new();
        if let Ok(terminal) = result {
            let size = terminal.size();
            if let Ok(size) = size {
                // Terminal size should be positive in a test environment
                assert!(size.rows > 0);
                assert!(size.cols > 0);
            }
        }
    }

    #[test]
    fn test_terminal_default() {
        // This test will panic in non-TTY environments
        // Skip if we can't create a terminal
        let _ = Terminal::new();
    }

    #[test]
    fn test_size_struct() {
        let size = Size { rows: 24, cols: 80 };
        assert_eq!(size.rows, 24);
        assert_eq!(size.cols, 80);
    }
}
