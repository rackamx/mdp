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
}

impl Renderer {
    /// Create a new renderer with the specified terminal width
    pub fn new(width: usize) -> Self {
        Renderer {
            width,
            cursor_col: 0,
            lines: Vec::new(),
            current_line: String::new(),
        }
    }

    /// Render markdown events to a string
    pub fn render(&mut self, events: &[Event]) -> String {
        // Reset state
        self.lines.clear();
        self.current_line.clear();
        self.cursor_col = 0;

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
            crate::parsing::Block::Heading { level: _, text: _ } => {
                // Heading will be rendered when we see the text
            }
            crate::parsing::Block::Paragraph => {
                // Start of paragraph - no special handling needed
            }
            crate::parsing::Block::BlockQuote => {
                // Block quote handling
            }
            crate::parsing::Block::CodeBlock { info: _ } => {
                // Code block handling
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
            crate::parsing::Block::Heading { level, text } => {
                // Render heading with underline
                let text = text.clone();
                if !self.current_line.is_empty() {
                    self.lines.push(self.current_line.clone());
                    self.current_line.clear();
                }

                // Add underline based on heading level
                let underline = match level {
                    1 => "=",
                    2 => "-",
                    _ => "~",
                };
                let underline_str = underline.repeat(text.len().min(self.width));
                self.lines.push(underline_str);
                self.cursor_col = 0;
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
