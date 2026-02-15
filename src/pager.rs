/// Pager module - handles scrolling through rendered content
use std::borrow::Cow;
use std::cmp;
use std::collections::HashMap;

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
    /// List of (line_index, start_byte, end_byte) for search matches
    search_matches: Vec<(usize, usize, usize)>,
    /// Search matches indexed by line for efficient highlighting
    search_matches_by_line: HashMap<usize, Vec<(usize, usize, usize)>>,
    /// Index of current match in search_matches
    current_match_index: Option<usize>,
    /// Preprocessed searchable units for each line.
    search_index: Vec<Vec<SearchUnit>>,
}

#[derive(Clone, Copy)]
struct SearchUnit {
    ch: char,
    start: usize,
    end: usize,
}

impl Pager {
    fn max_scroll_position(&self) -> usize {
        self.lines.len().saturating_sub(self.config.page_size)
    }

    /// Create a new Pager with the given config and content lines
    pub fn new(config: PagerConfig, lines: Vec<String>) -> Self {
        let search_index = lines
            .iter()
            .map(|line| Self::build_search_units(line))
            .collect();
        Pager {
            lines,
            scroll_position: 0,
            config,
            search_pattern: None,
            search_matches: Vec::new(),
            search_matches_by_line: HashMap::new(),
            current_match_index: None,
            search_index,
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

    /// Returns the currently visible line range as [start, end).
    pub fn visible_range(&self) -> (usize, usize) {
        if self.scroll_position >= self.lines.len() {
            return (self.lines.len(), self.lines.len());
        }
        let end = (self.scroll_position + self.config.page_size).min(self.lines.len());
        (self.scroll_position, end)
    }

    pub fn page_size(&self) -> usize {
        self.config.page_size
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

    /// Search for a pattern in the content.
    /// Behavior matches pager expectations: search forward from the current
    /// scroll position, wrapping around the document if needed.
    /// Returns Some((line_index, column_index)) of selected match, or None if not found.
    pub fn search(&mut self, pattern: &str) -> Option<(usize, usize)> {
        if pattern.is_empty() {
            self.search_pattern = None;
            self.search_matches.clear();
            self.search_matches_by_line.clear();
            self.current_match_index = None;
            return None;
        }

        self.search_pattern = Some(pattern.to_string());
        self.search_matches.clear();
        self.search_matches_by_line.clear();
        let pat_chars: Vec<char> = pattern.chars().collect();

        // Find all matches
        for (line_idx, units) in self.search_index.iter().enumerate() {
            for (start, end) in Self::find_matches_in_units(units, &pat_chars) {
                self.search_matches.push((line_idx, start, end));
            }
        }

        if self.search_matches.is_empty() {
            self.current_match_index = None;
            return None;
        }

        // Select the first match after the current viewport start, wrapping to
        // the first match in the document if needed.
        let start_line = self.scroll_position.saturating_add(1);
        let selected_idx = self
            .search_matches
            .iter()
            .position(|(line_idx, _, _)| *line_idx >= start_line)
            .unwrap_or(0);

        self.current_match_index = Some(selected_idx);
        let (line_idx, col_idx, _end_idx) = self.search_matches[selected_idx];
        self.rebuild_match_line_index();

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

        let (line_idx, _, _) = self.search_matches[next_idx];
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

        let (line_idx, _, _) = self.search_matches[prev_idx];
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
        let (start, end) = self.visible_range();
        (start..end)
            .map(|line_num| self.display_line(line_num).into_owned())
            .collect()
    }

    pub fn display_line<'a>(&'a self, line_num: usize) -> Cow<'a, str> {
        let Some(line) = self.lines.get(line_num) else {
            return Cow::Borrowed("");
        };
        if self.search_pattern.is_none() || !self.search_matches_by_line.contains_key(&line_num) {
            return Cow::Borrowed(line);
        }
        Cow::Owned(self.highlight_pattern(line, line_num))
    }

    /// Highlight all occurrences of a pattern in a line
    fn highlight_pattern(&self, line: &str, line_num: usize) -> String {
        let Some(line_matches) = self.search_matches_by_line.get(&line_num) else {
            return line.to_string();
        };

        let mut result = String::new();
        let mut last_end = 0;

        for (match_idx, start, end) in line_matches {
            // Add text before the match
            if *start > last_end {
                result.push_str(&line[last_end..*start]);
            }
            // Highlight all matches in bold, and current match in reverse for visibility.
            if self.current_match_index == Some(*match_idx) {
                result.push_str("\x1b[7m\x1b[1m");
            } else {
                result.push_str("\x1b[1m");
            }
            result.push_str(&line[*start..*end]);
            result.push_str("\x1b[0m"); // Bold end
            last_end = *end;
        }

        // Add remaining text after last match
        if last_end < line.len() {
            result.push_str(&line[last_end..]);
        }

        result
    }

    fn find_matches_in_units(units: &[SearchUnit], pat_chars: &[char]) -> Vec<(usize, usize)> {
        if pat_chars.is_empty() {
            return Vec::new();
        }
        if units.len() < pat_chars.len() {
            return Vec::new();
        }

        let mut out = Vec::new();
        let mut start = 0usize;
        while start + pat_chars.len() <= units.len() {
            let mut matched = true;
            for (offset, pat_ch) in pat_chars.iter().enumerate() {
                if units[start + offset].ch != *pat_ch {
                    matched = false;
                    break;
                }
            }

            if matched {
                let start_byte = units[start].start;
                let end_byte = units[start + pat_chars.len() - 1].end;
                out.push((start_byte, end_byte));
            }
            start += 1;
        }

        out
    }

    fn build_search_units(line: &str) -> Vec<SearchUnit> {
        let mut units = Vec::new();
        let bytes = line.as_bytes();
        let mut i = 0usize;

        while i < bytes.len() {
            let ch = line[i..]
                .chars()
                .next()
                .expect("char boundary iteration should be valid");
            let ch_len = ch.len_utf8();

            // Skip ANSI CSI escapes like \x1b[...m
            if ch == '\x1b' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                i += 2;
                while i < bytes.len() {
                    let b = bytes[i];
                    i += 1;
                    if (0x40..=0x7E).contains(&b) {
                        break;
                    }
                }
                continue;
            }

            // Skip Unicode combining strikethrough overlay.
            if ch == '\u{0336}' {
                i += ch_len;
                continue;
            }

            units.push(SearchUnit {
                ch,
                start: i,
                end: i + ch_len,
            });
            i += ch_len;
        }

        units
    }

    fn rebuild_match_line_index(&mut self) {
        self.search_matches_by_line.clear();
        for (match_idx, (line, start, end)) in self.search_matches.iter().enumerate() {
            self.search_matches_by_line
                .entry(*line)
                .or_default()
                .push((match_idx, *start, *end));
        }
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
        self.search_matches_by_line.clear();
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

    #[test]
    fn test_search_status_progresses_with_next_previous() {
        let lines = vec![
            "alpha beta".to_string(),
            "beta gamma".to_string(),
            "delta beta".to_string(),
        ];
        let mut pager = Pager::new(PagerConfig::default(), lines);

        let first = pager.search("beta");
        assert!(first.is_some(), "Expected to find at least one match");
        assert_eq!(pager.search_status_message(), "Search: 'beta' (2/3)");

        pager.search_next();
        assert_eq!(pager.search_status_message(), "Search: 'beta' (3/3)");

        pager.search_previous();
        assert_eq!(pager.search_status_message(), "Search: 'beta' (2/3)");
    }

    #[test]
    fn test_current_search_match_is_visually_distinct() {
        let lines = vec!["foo foo".to_string()];
        let mut pager = Pager::new(PagerConfig::default(), lines);
        let _ = pager.search("foo");

        let rendered = pager.visible_lines_with_highlight();
        let line = &rendered[0];
        let reverse_count = line.matches("\x1b[7m").count();
        assert_eq!(
            reverse_count, 1,
            "Expected exactly one active match highlight, got: {:?}",
            line
        );

        pager.search_next();
        let rendered_next = pager.visible_lines_with_highlight();
        let line_next = &rendered_next[0];
        let reverse_count_next = line_next.matches("\x1b[7m").count();
        assert_eq!(
            reverse_count_next, 1,
            "Expected exactly one active match highlight after next, got: {:?}",
            line_next
        );
    }

    #[test]
    fn test_search_starts_from_current_position_and_wraps() {
        let lines = vec![
            "target at top".to_string(), // line 0
            "line 1".to_string(),
            "line 2".to_string(),
            "line 3".to_string(),
            "line 4".to_string(),
            "target after viewport".to_string(), // line 5
        ];
        let mut pager = Pager::new(
            PagerConfig {
                page_size: 2,
                cols: 80,
            },
            lines,
        );
        pager.go_to_line(2); // current viewport starts at line 2

        let first = pager.search("target").expect("expected search match");
        assert_eq!(
            first.0, 5,
            "Expected first forward match from current position, got: {:?}",
            first
        );

        pager.search_next();
        assert_eq!(
            pager.search_status_message(),
            "Search: 'target' (1/2)",
            "Expected wrapped next match after reaching end"
        );
    }

    #[test]
    fn test_search_matches_unicode_combining_strikethrough_text() {
        let struck = "s\u{0336}t\u{0336}r\u{0336}i\u{0336}k\u{0336}e\u{0336}";
        let mut pager = Pager::new(
            PagerConfig {
                page_size: 5,
                cols: 80,
            },
            vec![format!("this is {struck} text")],
        );

        let found = pager.search("strike");
        assert!(found.is_some(), "Expected plain query to match struck text");
        assert_eq!(pager.search_status_message(), "Search: 'strike' (1/1)");
    }
}
