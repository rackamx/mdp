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
    /// Current search pattern
    search_pattern: Option<String>,
    /// List of (line_index, column_index) for search matches
    search_matches: Vec<(usize, usize)>,
    /// Index of current match in search_matches
    current_match_index: Option<usize>,
}

impl Pager {
    fn max_scroll_position(&self) -> usize {
        self.lines.len().saturating_sub(self.config.page_size)
    }

    /// Create a new Pager with the given config and content lines
    pub fn new(config: PagerConfig, lines: Vec<String>) -> Self {
        Pager {
            lines,
            scroll_position: 0,
            config,
            search_pattern: None,
            search_matches: Vec::new(),
            current_match_index: None,
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
        self.scroll_position = cmp::min(new_position, self.max_scroll_position());
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
        Some(format!("--- More --- ({}/{})", current_pos, total_lines))
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
        self.scroll_position = cmp::min(line, self.max_scroll_position());
    }

    /// Go to the beginning of the content
    pub fn go_to_beginning(&mut self) {
        self.scroll_position = 0;
    }

    /// Go to the end of the content
    pub fn go_to_end(&mut self) {
        self.scroll_position = self.max_scroll_position();
    }

    /// Scroll down by one line
    pub fn scroll_down(&mut self) {
        self.scroll_position = cmp::min(self.scroll_position + 1, self.max_scroll_position());
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

    /// Search for a pattern in the content
    /// Returns Some((line_index, column_index)) of first match, or None if not found
    pub fn search(&mut self, pattern: &str) -> Option<(usize, usize)> {
        if pattern.is_empty() {
            self.search_pattern = None;
            self.search_matches.clear();
            self.current_match_index = None;
            return None;
        }

        self.search_pattern = Some(pattern.to_string());
        self.search_matches.clear();

        // Find all matches
        for (line_idx, line) in self.lines.iter().enumerate() {
            let mut search_start = 0;
            while let Some(pos) = line[search_start..].find(pattern) {
                let absolute_pos = search_start + pos;
                self.search_matches.push((line_idx, absolute_pos));
                search_start = absolute_pos + 1;
            }
        }

        if self.search_matches.is_empty() {
            self.current_match_index = None;
            return None;
        }

        // Set current match to first match
        self.current_match_index = Some(0);
        let (line_idx, col_idx) = self.search_matches[0];

        // Scroll to the match
        self.scroll_to_show_line(line_idx);

        Some((line_idx, col_idx))
    }

    /// Go to the next search match
    pub fn search_next(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        let current_idx = self.current_match_index.unwrap_or(0);
        let next_idx = (current_idx + 1) % self.search_matches.len();
        self.current_match_index = Some(next_idx);

        let (line_idx, _) = self.search_matches[next_idx];
        self.scroll_to_show_line(line_idx);
    }

    /// Go to the previous search match
    pub fn search_previous(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        let current_idx = self.current_match_index.unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            self.search_matches.len() - 1
        } else {
            current_idx - 1
        };
        self.current_match_index = Some(prev_idx);

        let (line_idx, _) = self.search_matches[prev_idx];
        self.scroll_to_show_line(line_idx);
    }

    /// Scroll to show a specific line
    fn scroll_to_show_line(&mut self, line_idx: usize) {
        let page_size = self.config.page_size;

        // If the line is before current view, scroll up
        if line_idx < self.scroll_position {
            self.scroll_position = line_idx;
        }
        // If the line is after current view, scroll down
        else if line_idx >= self.scroll_position + page_size {
            // Scroll so that the line is near the top of the page
            self.scroll_position = line_idx.saturating_sub(page_size / 2);
        }

        self.scroll_position = cmp::min(self.scroll_position, self.max_scroll_position());
    }

    /// Get visible lines with search matches highlighted
    pub fn visible_lines_with_highlight(&self) -> Vec<String> {
        let pattern = match &self.search_pattern {
            Some(p) => p,
            None => return self.visible_lines(),
        };

        let visible = self.visible_lines();
        let start_line = self.scroll_position;

        visible
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                let line_num = start_line + idx;
                self.highlight_pattern(line, pattern, line_num)
            })
            .collect()
    }

    /// Highlight all occurrences of a pattern in a line
    fn highlight_pattern(&self, line: &str, pattern: &str, line_num: usize) -> String {
        if !self.search_matches.iter().any(|(l, _)| *l == line_num) {
            return line.to_string();
        }

        let mut result = String::new();
        let mut last_end = 0;

        // Get all matches on this line
        let line_matches: Vec<usize> = self
            .search_matches
            .iter()
            .filter(|(l, _)| *l == line_num)
            .map(|(_, col)| *col)
            .collect();

        for col in line_matches {
            // Add text before the match
            if col > last_end {
                result.push_str(&line[last_end..col]);
            }
            // Add the highlighted match
            result.push_str("\x1b[1m"); // Bold start
            result.push_str(&line[col..col + pattern.len()]);
            result.push_str("\x1b[0m"); // Bold end
            last_end = col + pattern.len();
        }

        // Add remaining text after last match
        if last_end < line.len() {
            result.push_str(&line[last_end..]);
        }

        result
    }

    /// Get the search status message
    pub fn search_status_message(&self) -> String {
        match &self.search_pattern {
            Some(pattern) => {
                if self.search_matches.is_empty() {
                    format!("Pattern '{}' not found", pattern)
                } else if let Some(idx) = self.current_match_index {
                    format!(
                        "Search: '{}' ({}/{})",
                        pattern,
                        idx + 1,
                        self.search_matches.len()
                    )
                } else {
                    format!("Search: '{}'", pattern)
                }
            }
            None => String::new(),
        }
    }

    /// Clear the current search
    pub fn clear_search(&mut self) {
        self.search_pattern = None;
        self.search_matches.clear();
        self.current_match_index = None;
    }

    /// Get the help text displaying all available keybindings
    pub fn help_text(&self) -> Vec<String> {
        vec![
            "Keybindings:".to_string(),
            "".to_string(),
            "Navigation:".to_string(),
            "  j, down, Enter  - Move down one line".to_string(),
            "  k, up           - Move up one line".to_string(),
            "  space, f, PageDown - Page down".to_string(),
            "  b, PageUp       - Page up".to_string(),
            "  g, Home         - Go to beginning".to_string(),
            "  G, End          - Go to end".to_string(),
            "".to_string(),
            "Search:".to_string(),
            "  /               - Search forward".to_string(),
            "  n               - Next search match".to_string(),
            "  N               - Previous search match".to_string(),
            "".to_string(),
            "Other:".to_string(),
            "  r               - Reload file from disk".to_string(),
            "  h, ?            - Show this help".to_string(),
            "  q, Q, ZZ        - Quit".to_string(),
        ]
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

    #[test]
    fn test_help_mentions_reload_key() {
        let pager = Pager::new(PagerConfig::default(), vec!["line".to_string()]);
        let help = pager.help_text();
        assert!(
            help.iter()
                .any(|line| line.contains("r") && line.contains("Reload")),
            "Expected reload keybinding in help text, got: {:?}",
            help
        );
    }
}
