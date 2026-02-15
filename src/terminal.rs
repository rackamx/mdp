use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};
use std::io::{self, Write};
use std::panic;
use std::sync::Arc;

trait TerminalOps: Send + Sync {
    fn enable_raw_mode(&self) -> Result<(), io::Error>;
    fn disable_raw_mode(&self) -> Result<(), io::Error>;
    fn size(&self) -> Result<(u16, u16), io::Error>;
    fn flush_stdout(&self) -> Result<(), io::Error>;
    fn install_panic_hook(&self, cleanup: Box<dyn Fn() + Send + Sync + 'static>);
}

struct SystemTerminalOps;

impl TerminalOps for SystemTerminalOps {
    fn enable_raw_mode(&self) -> Result<(), io::Error> {
        enable_raw_mode()
    }

    fn disable_raw_mode(&self) -> Result<(), io::Error> {
        disable_raw_mode()
    }

    fn size(&self) -> Result<(u16, u16), io::Error> {
        size()
    }

    fn flush_stdout(&self) -> Result<(), io::Error> {
        io::stdout().flush()
    }

    fn install_panic_hook(&self, cleanup: Box<dyn Fn() + Send + Sync + 'static>) {
        panic::set_hook(Box::new(move |_| cleanup()));
    }
}

/// Represents terminal state and provides RAII-style cleanup.
///
/// This struct enters raw mode on creation and restores the terminal
/// on drop (when it goes out of scope or when a panic occurs).
pub struct Terminal {
    // Track whether we successfully entered raw mode
    is_raw_mode: bool,
    ops: Arc<dyn TerminalOps>,
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
        Self::new_with_ops(Arc::new(SystemTerminalOps))
    }

    fn new_with_ops(ops: Arc<dyn TerminalOps>) -> Result<Self, std::io::Error> {
        ops.enable_raw_mode()?;

        let cleanup_ops = Arc::clone(&ops);
        ops.install_panic_hook(Box::new(move || {
            let _ = cleanup_ops.disable_raw_mode();
        }));

        Ok(Terminal {
            is_raw_mode: true,
            ops,
        })
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
        let (cols, rows) = self.ops.size()?;
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
            let _ = self.ops.disable_raw_mode();

            // Flush stdout to ensure clean output
            let _ = self.ops.flush_stdout();
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
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Mutex;

    struct MockTerminalOps {
        fail_enable: AtomicBool,
        fail_size: AtomicBool,
        disable_calls: AtomicUsize,
        flush_calls: AtomicUsize,
        hook: Mutex<Option<Box<dyn Fn() + Send + Sync + 'static>>>,
    }

    impl MockTerminalOps {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                fail_enable: AtomicBool::new(false),
                fail_size: AtomicBool::new(false),
                disable_calls: AtomicUsize::new(0),
                flush_calls: AtomicUsize::new(0),
                hook: Mutex::new(None),
            })
        }

        fn trigger_panic_cleanup(&self) {
            if let Some(hook) = self.hook.lock().expect("lock hook").as_ref() {
                hook();
            }
        }
    }

    impl TerminalOps for MockTerminalOps {
        fn enable_raw_mode(&self) -> Result<(), io::Error> {
            if self.fail_enable.load(Ordering::SeqCst) {
                return Err(io::Error::other("enable failed"));
            }
            Ok(())
        }

        fn disable_raw_mode(&self) -> Result<(), io::Error> {
            self.disable_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn size(&self) -> Result<(u16, u16), io::Error> {
            if self.fail_size.load(Ordering::SeqCst) {
                return Err(io::Error::other("size failed"));
            }
            Ok((120, 40))
        }

        fn flush_stdout(&self) -> Result<(), io::Error> {
            self.flush_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn install_panic_hook(&self, cleanup: Box<dyn Fn() + Send + Sync + 'static>) {
            *self.hook.lock().expect("lock hook") = Some(cleanup);
        }
    }

    #[test]
    fn test_terminal_creation() {
        let result = Terminal::new();
        if result.is_ok() {
            let _terminal = result.expect("terminal should exist");
        }
    }

    #[test]
    fn test_terminal_size() {
        let result = Terminal::new();
        if let Ok(terminal) = result {
            let size = terminal.size();
            if let Ok(size) = size {
                assert!(size.rows > 0);
                assert!(size.cols > 0);
            }
        }
    }

    #[test]
    fn test_terminal_default() {
        let _ = Terminal::new();
    }

    #[test]
    fn test_size_struct() {
        let size = Size { rows: 24, cols: 80 };
        assert_eq!(size.rows, 24);
        assert_eq!(size.cols, 80);
    }

    #[test]
    fn test_new_with_ops_propagates_enable_failure() {
        let ops = MockTerminalOps::new();
        ops.fail_enable.store(true, Ordering::SeqCst);
        let result = Terminal::new_with_ops(ops);
        assert!(result.is_err(), "Expected terminal creation failure");
    }

    #[test]
    fn test_drop_invokes_disable_and_flush() {
        let ops = MockTerminalOps::new();
        let terminal = Terminal::new_with_ops(ops.clone()).expect("terminal should be created");
        drop(terminal);

        assert_eq!(
            ops.disable_calls.load(Ordering::SeqCst),
            1,
            "Expected one disable call during Drop"
        );
        assert_eq!(
            ops.flush_calls.load(Ordering::SeqCst),
            1,
            "Expected one flush call during Drop"
        );
    }

    #[test]
    fn test_size_error_path_is_deterministic_without_tty() {
        let ops = MockTerminalOps::new();
        ops.fail_size.store(true, Ordering::SeqCst);
        let terminal = Terminal::new_with_ops(ops).expect("terminal should be created");
        let result = terminal.size();
        assert!(result.is_err(), "Expected deterministic size failure");
    }

    #[test]
    fn test_panic_hook_cleanup_uses_disable_path_without_tty() {
        let ops = MockTerminalOps::new();
        let terminal = Terminal::new_with_ops(ops.clone()).expect("terminal should be created");

        ops.trigger_panic_cleanup();
        assert_eq!(
            ops.disable_calls.load(Ordering::SeqCst),
            1,
            "Expected panic cleanup hook to call disable_raw_mode once"
        );

        drop(terminal);
    }

    #[test]
    fn test_is_raw_mode_true_after_successful_creation() {
        let ops = MockTerminalOps::new();
        let terminal = Terminal::new_with_ops(ops).expect("terminal should be created");
        assert!(terminal.is_raw_mode(), "expected raw mode flag after creation");
    }

    #[test]
    fn test_size_reads_values_from_ops() {
        let ops = MockTerminalOps::new();
        let terminal = Terminal::new_with_ops(ops).expect("terminal should be created");
        let size = terminal.size().expect("size should succeed");
        assert_eq!(size.cols, 120);
        assert_eq!(size.rows, 40);
    }

    #[test]
    fn test_system_terminal_ops_methods_are_invocable_best_effort() {
        let ops = SystemTerminalOps;

        let _ = ops.enable_raw_mode();
        let _ = ops.disable_raw_mode();
        let _ = ops.size();
        let _ = ops.flush_stdout();

        let previous_hook = panic::take_hook();
        ops.install_panic_hook(Box::new(|| {}));
        panic::set_hook(previous_hook);
    }
}
