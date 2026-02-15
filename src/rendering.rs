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

        for event in events {
            self.process_event(event);
        }

        // Flush any remaining content in the current line
        if !self.current_line.is_empty() {
            self.lines.push(self.current_line.clone());
        }

        self.lines.join("\n")
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
        if self.cursor_col > 0 && self.cursor_col < self.width {
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
                // Block quote handling
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
            crate::parsing::Block::List { start: _ } => {
                // List handling
            }
            crate::parsing::Block::ListItem => {
                // List item handling
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
