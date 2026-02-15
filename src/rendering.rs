use crate::parsing::Event;

/// Renderer for converting markdown events to terminal output
pub struct Renderer {
    /// Terminal width for line wrapping
    width: usize,
    /// Current cursor position (column)
    cursor_col: usize,
    /// Accumulated output lines
    lines: Vec<String>,
    /// Current line being built
    current_line: String,
    /// Current heading level (0 if not in a heading)
    heading_level: u8,
    /// Heading text collected so far
    heading_text: String,
    /// Whether we're currently in a code block
    in_code_block: bool,
    /// Code block info string (e.g., language)
    code_block_info: Option<String>,
    /// Whether we're currently in a block quote
    in_block_quote: bool,
    /// Prefix character for block quote
    block_quote_prefix: char,
    /// Whether we're currently in a list
    in_list: bool,
    /// Whether the current list is ordered (numbered) vs unordered (bullet)
    list_is_ordered: bool,
    /// Current item number for ordered lists
    list_item_number: u64,
    /// Current list nesting depth
    list_depth: usize,
}

impl Renderer {
    /// Create a new renderer with the specified terminal width
    pub fn new(width: usize) -> Self {
        Renderer {
            width,
            cursor_col: 0,
            lines: Vec::new(),
            current_line: String::new(),
            heading_level: 0,
            heading_text: String::new(),
            in_code_block: false,
            code_block_info: None,
            in_block_quote: false,
            block_quote_prefix: '|',
            in_list: false,
            list_is_ordered: false,
            list_item_number: 0,
            list_depth: 0,
        }
    }

    /// Render markdown events to a string
    pub fn render(&mut self, events: &[Event]) -> String {
        // Reset state
        self.lines.clear();
        self.current_line.clear();
        self.cursor_col = 0;
        self.heading_level = 0;
        self.heading_text.clear();
        self.in_code_block = false;
        self.code_block_info = None;
        self.in_block_quote = false;
        self.in_list = false;
        self.list_is_ordered = false;
        self.list_item_number = 0;
        self.list_depth = 0;

        // Combine consecutive Text events to properly handle escape sequences
        // like \*text\* which pulldown_cmark splits into multiple Text events
        let combined_events = Self::combine_text_events(events);

        for event in combined_events {
            self.process_event(&event);
        }

        // Flush any remaining content in the current line
        if !self.current_line.is_empty() {
            self.lines.push(self.current_line.clone());
        }

        self.lines.join("\n")
    }

    /// Combine consecutive Text events into a single Text event.
    /// This is needed because pulldown_cmark can emit multiple Text events
    /// for escape sequences like \*text\* (producing "*text" and "*" separately).
    fn combine_text_events(events: &[Event]) -> Vec<Event> {
        let mut result = Vec::new();
        let mut pending_text: Option<String> = None;

        for event in events {
            match event {
                Event::Text(text) => {
                    if let Some(ref mut pending) = pending_text {
                        pending.push_str(text);
                    } else {
                        pending_text = Some(text.clone());
                    }
                }
                _ => {
                    // Flush pending text if any
                    if let Some(text) = pending_text.take() {
                        result.push(Event::Text(text));
                    }
                    result.push(event.clone());
                }
            }
        }

        // Flush any remaining pending text
        if let Some(text) = pending_text {
            result.push(Event::Text(text));
        }

        result
    }

    /// Process a single markdown event
    fn process_event(&mut self, event: &Event) {
        match event {
            Event::Text(text) => self.render_text(text),
            Event::SoftBreak => self.render_space(),
            Event::HardBreak => self.render_newline(),
            Event::Rule => self.render_rule(),
            Event::Start(block) => self.render_block_start(block),
            Event::End(block) => self.render_block_end(block),
            Event::StrongStart => self.render_bold_start(),
            Event::StrongEnd => self.render_bold_end(),
            Event::EmphasisStart => self.render_italics_start(),
            Event::EmphasisEnd => self.render_italics_end(),
            Event::InlineCode(code) => self.render_inline_code(code),
            Event::Link { text, url } => self.render_link(text, url),
            Event::Image { alt, url: _ } => self.render_image(alt),
        }
    }

    /// Render text content with word wrapping
    fn render_text(&mut self, text: &str) {
        for word in text.split_whitespace() {
            let word_len = word.len();

            // Check if we need to wrap to a new line
            if self.cursor_col > 0 && self.cursor_col + word_len + 1 > self.width {
                // Flush current line and start a new one
                if !self.current_line.is_empty() {
                    self.lines.push(self.current_line.clone());
                    self.current_line.clear();
                }
                self.cursor_col = 0;
            }

            // Add word to current line
            if self.cursor_col > 0 {
                self.current_line.push(' ');
                self.cursor_col += 1;
            }
            self.current_line.push_str(word);
            self.cursor_col += word_len;
        }
    }

    /// Render a space (soft break between words)
    fn render_space(&mut self) {
        // Within a block quote, soft breaks become new lines with prefix
        if self.in_block_quote {
            // Flush current line
            if !self.current_line.is_empty() {
                self.lines.push(self.current_line.clone());
                self.current_line.clear();
            }
            // Add block quote prefix for new line
            self.current_line.push(self.block_quote_prefix);
            self.current_line.push(' ');
            self.cursor_col = 2;
        } else if self.cursor_col > 0 && self.cursor_col < self.width {
            self.current_line.push(' ');
            self.cursor_col += 1;
        }
    }

    /// Render bold text start (ANSI bold on)
    fn render_bold_start(&mut self) {
        self.current_line.push_str("\x1b[1m");
        // Track escape sequence length for cursor position
        // \x1b[1m is 4 characters but doesn't count toward visible width
    }

    /// Render bold text end (ANSI bold off)
    fn render_bold_end(&mut self) {
        self.current_line.push_str("\x1b[0m");
    }

    /// Render italics text start (ANSI italics on)
    fn render_italics_start(&mut self) {
        self.current_line.push_str("\x1b[3m");
    }

    /// Render italics text end (ANSI italics off)
    fn render_italics_end(&mut self) {
        self.current_line.push_str("\x1b[0m");
    }

    /// Render inline code with backticks and monospace indication
    fn render_inline_code(&mut self, code: &str) {
        // Add backticks and use faint (dim) for monospace indication
        self.current_line.push('`');
        self.current_line.push_str("\x1b[2m");  // Faint for monospace
        self.current_line.push_str(code);
        self.current_line.push_str("\x1b[0m");  // Reset
        self.current_line.push('`');
        self.cursor_col += 2 + code.len() + 2 + 1;  // ` + ANSI + code + ANSI + `
    }

    /// Render a link as "text (url)"
    fn render_link(&mut self, text: &str, url: &str) {
        // Add the link text
        self.render_text(text);
        // Add space and URL in parentheses
        if !url.is_empty() {
            self.current_line.push_str(" (");
            self.current_line.push_str(url);
            self.current_line.push(')');
            self.cursor_col += 3 + url.len();  // " (" + url + ")"
        }
    }

    /// Render an image by showing just the alt text in brackets
    fn render_image(&mut self, alt: &str) {
        // Show alt text in brackets to indicate it's an image, not actual image rendering
        self.current_line.push('[');
        self.current_line.push_str(alt);
        self.current_line.push(']');
        self.cursor_col += 2 + alt.len();  // "[" + alt + "]"
    }

    /// Render a hard line break
    fn render_newline(&mut self) {
        // Flush current line
        if !self.current_line.is_empty() {
            self.lines.push(self.current_line.clone());
            self.current_line.clear();
        } else {
            // Empty line
            self.lines.push(String::new());
        }
        self.cursor_col = 0;

        // If we're in a block quote, add prefix to new line
        if self.in_block_quote {
            self.current_line.push(self.block_quote_prefix);
            self.current_line.push(' ');
            self.cursor_col = 2;
        }
    }

    /// Render a horizontal rule
    fn render_rule(&mut self) {
        // Flush current line first
        if !self.current_line.is_empty() {
            self.lines.push(self.current_line.clone());
            self.current_line.clear();
            self.cursor_col = 0;
        }

        // Add rule line
        let rule = "-".repeat(self.width);
        self.lines.push(rule);
    }

    /// Handle the start of a block element
    fn render_block_start(&mut self, block: &crate::parsing::Block) {
        // Flush any pending content
        if !self.current_line.is_empty() {
            self.lines.push(self.current_line.clone());
            self.current_line.clear();
            self.cursor_col = 0;
        }

        // Add a blank line before block elements (except first)
        if !self.lines.is_empty() {
            self.lines.push(String::new());
        }

        match block {
            crate::parsing::Block::Heading { level, text: _ } => {
                // Track heading level and start bold
                self.heading_level = *level;
                self.heading_text.clear();
                // Start bold for heading
                self.render_bold_start();
            }
            crate::parsing::Block::Paragraph => {
                // Start of paragraph - no special handling needed
            }
            crate::parsing::Block::BlockQuote => {
                // Track block quote state
                self.in_block_quote = true;
                // Add block quote prefix
                self.current_line.push(self.block_quote_prefix);
                self.current_line.push(' ');
                self.cursor_col = 2;
            }
            crate::parsing::Block::CodeBlock { info } => {
                // Track code block state
                self.in_code_block = true;
                self.code_block_info = info.clone();
                // Add opening fence with info string if present
                if let Some(ref lang) = info {
                    self.current_line.push_str("```");
                    self.current_line.push_str(lang);
                    self.cursor_col = 3 + lang.len();
                } else {
                    self.current_line.push_str("```");
                    self.cursor_col = 3;
                }
                // Use faint for monospace indication
                self.current_line.push_str("\x1b[2m");
            }
            crate::parsing::Block::List { start } => {
                // Track list state
                if self.in_list {
                    // We're already in a list, this is a nested list
                    self.list_depth += 1;
                }
                self.in_list = true;
                self.list_is_ordered = start.is_some();
                self.list_item_number = start.unwrap_or(1);
            }
            crate::parsing::Block::ListItem => {
                // Add list item prefix with proper indentation
                // Indentation is 2 spaces per depth level
                let indentation = "  ".repeat(self.list_depth);
                self.current_line.push_str(&indentation);

                // Add the marker
                if self.list_is_ordered {
                    // Ordered list: "1. ", "2. ", etc.
                    self.current_line.push_str(&format!("{}. ", self.list_item_number));
                    self.list_item_number += 1;
                } else {
                    // Unordered list: use "* " for - and + markers (standard)
                    self.current_line.push_str("* ");
                }

                // Update cursor position (accounting for indentation and marker)
                self.cursor_col = indentation.len() + if self.list_is_ordered {
                    // Number + ". "
                    self.list_item_number.to_string().len() + 2
                } else {
                    // "* " = 2
                    2
                };
            }
        }
    }

    /// Handle the end of a block element
    fn render_block_end(&mut self, block: &crate::parsing::Block) {
        match block {
            crate::parsing::Block::Heading { level: _, text: _ } => {
                // End bold for heading
                self.render_bold_end();

                // Get the heading text from the current line (strip ANSI codes)
                let heading_text = self.current_line.clone();
                let text_len = heading_text
                    .chars()
                    .filter(|c| !c.is_ascii_control())
                    .count();

                if !self.current_line.is_empty() {
                    self.lines.push(self.current_line.clone());
                    self.current_line.clear();
                }

                // Add underline based on heading level (use stored level)
                let underline = match self.heading_level {
                    1 => "=",
                    2 => "-",
                    _ => "~",
                };
                let underline_str = underline.repeat(text_len.min(self.width));
                self.lines.push(underline_str);
                self.cursor_col = 0;

                // Reset heading state
                self.heading_level = 0;
            }
            crate::parsing::Block::CodeBlock { info: _ } => {
                // End monospace
                self.current_line.push_str("\x1b[0m");
                // Add closing fence
                self.current_line.push_str("```");
                // Flush the line
                if !self.current_line.is_empty() {
                    self.lines.push(self.current_line.clone());
                    self.current_line.clear();
                    self.cursor_col = 0;
                }
                // Reset code block state
                self.in_code_block = false;
                self.code_block_info = None;
            }
            crate::parsing::Block::BlockQuote => {
                // Reset block quote state
                self.in_block_quote = false;
                // Flush current line if not empty
                if !self.current_line.is_empty() {
                    self.lines.push(self.current_line.clone());
                    self.current_line.clear();
                    self.cursor_col = 0;
                }
            }
            crate::parsing::Block::List { start: _ } => {
                // Handle end of list - decrease depth if nested
                if self.list_depth > 0 {
                    self.list_depth -= 1;
                } else {
                    // We're exiting the top-level list
                    self.in_list = false;
                    self.list_is_ordered = false;
                    self.list_item_number = 0;
                }
            }
            _ => {
                // For other blocks, just flush current line
                if !self.current_line.is_empty() {
                    self.lines.push(self.current_line.clone());
                    self.current_line.clear();
                    self.cursor_col = 0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer::new(80);
        assert_eq!(renderer.width, 80);
    }

    #[test]
    fn test_render_empty() {
        let events: Vec<Event> = vec![];
        let mut renderer = Renderer::new(80);
        let output = renderer.render(&events);
        assert_eq!(output, "");
    }
}
