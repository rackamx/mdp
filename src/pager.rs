/// Pager module - handles scrolling through rendered content
use std::cmp;

/// Configuration for the pager
#[derive(Debug, Clone)]
pub struct PagerConfig {
    /// Number of rows per page
    pub page_size: usize,
    /// Terminal columns (for indicator width)
    pub cols: usize,
}

impl Default for PagerConfig {
    fn default() -> Self {
        PagerConfig {
            page_size: 24,
            cols: 80,
        }
    }
}

/// Pager struct for handling page-based navigation
pub struct Pager {
    /// All content lines
    lines: Vec<String>,
    /// Current scroll position (line index of first visible line)
    scroll_position: usize,
    /// Configuration
    config: PagerConfig,
}

impl Pager {
    /// Create a new Pager with the given config and content lines
    pub fn new(config: PagerConfig, lines: Vec<String>) -> Self {
        Pager {
            lines,
            scroll_position: 0,
            config,
        }
    }

    /// Get the lines that are currently visible
    pub fn visible_lines(&self) -> Vec<String> {
        let end = std::cmp::min(
            self.scroll_position + self.config.page_size,
            self.lines.len(),
        );

        if self.scroll_position >= self.lines.len() {
            return vec![];
        }

        self.lines[self.scroll_position..end].to_vec()
    }

    /// Scroll down by one page
    pub fn page_down(&mut self) {
        let new_position = self.scroll_position + self.config.page_size;
        self.scroll_position = cmp::min(new_position, self.lines.len().saturating_sub(1));
    }

    /// Scroll up by one page
    pub fn page_up(&mut self) {
        let new_position = self.scroll_position.saturating_sub(self.config.page_size);
        self.scroll_position = new_position;
    }

    /// Get the progress indicator text, if applicable
    /// Returns None if there's no more content to show
    pub fn progress_indicator(&self) -> Option<String> {
        let total_lines = self.lines.len();
        if total_lines == 0 {
            return None;
        }

        // Calculate current position (1-indexed for display)
        let current_pos = cmp::min(self.scroll_position + self.config.page_size, total_lines);

        // If we're at or past the end, show "END"
        if self.scroll_position + self.config.page_size >= total_lines {
            return Some(format!("[END - {} lines]", total_lines));
        }

        // Otherwise show progress like "--- More --- (10/50)"
        Some(format!(
            "--- More --- ({}/{})",
            current_pos, total_lines
        ))
    }

    /// Check if there's more content below
    pub fn has_more_below(&self) -> bool {
        self.scroll_position + self.config.page_size < self.lines.len()
    }

    /// Check if there's more content above
    pub fn has_more_above(&self) -> bool {
        self.scroll_position > 0
    }

    /// Go to a specific line
    pub fn go_to_line(&mut self, line: usize) {
        self.scroll_position = cmp::min(line, self.lines.len().saturating_sub(1));
    }

    /// Go to the beginning of the content
    pub fn go_to_beginning(&mut self) {
        self.scroll_position = 0;
    }

    /// Go to the end of the content
    pub fn go_to_end(&mut self) {
        self.scroll_position = self.lines.len().saturating_sub(self.config.page_size);
        self.scroll_position = cmp::max(0, self.scroll_position);
    }

    /// Scroll down by one line
    pub fn scroll_down(&mut self) {
        self.scroll_position = cmp::min(self.scroll_position + 1, self.lines.len().saturating_sub(1));
    }

    /// Scroll up by one line
    pub fn scroll_up(&mut self) {
        self.scroll_position = self.scroll_position.saturating_sub(1);
    }

    /// Get the total number of lines
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// Get the current scroll position (0-indexed)
    pub fn scroll_position(&self) -> usize {
        self.scroll_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pager_creation() {
        let lines = vec!["line1".to_string(), "line2".to_string()];
        let pager = Pager::new(PagerConfig::default(), lines);
        assert_eq!(pager.total_lines(), 2);
    }

    #[test]
    fn test_visible_lines_initial() {
        let lines: Vec<String> = (1..=30).map(|i| format!("Line {}", i)).collect();
        let pager = Pager::new(PagerConfig::default(), lines);
        let visible = pager.visible_lines();
        assert_eq!(visible.len(), 24); // Default page size
        assert_eq!(visible[0], "Line 1");
    }

    #[test]
    fn test_page_down() {
        let lines: Vec<String> = (1..=50).map(|i| format!("Line {}", i)).collect();
        let mut pager = Pager::new(PagerConfig::default(), lines);

        let first = pager.visible_lines()[0].clone();
        pager.page_down();
        let second = pager.visible_lines()[0].clone();

        assert_eq!(first, "Line 1");
        assert!(second.contains("Line 2") || second.contains("Line 2"));
    }

    #[test]
    fn test_page_up() {
        let lines: Vec<String> = (1..=50).map(|i| format!("Line {}", i)).collect();
        let mut pager = Pager::new(PagerConfig::default(), lines);

        pager.page_down();
        pager.page_down();
        let pos_before = pager.scroll_position();

        pager.page_up();
        let pos_after = pager.scroll_position();

        assert!(pos_after < pos_before);
    }
}
