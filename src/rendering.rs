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
    /// Whether to use Unicode fallback for strikethrough rendering
    strikethrough_fallback: bool,
    /// Whether we're currently inside strikethrough span
    in_strikethrough: bool,
    /// Nested bold depth
    bold_depth: usize,
    /// Nested italics depth
    italics_depth: usize,
    /// Nested strikethrough depth
    strikethrough_depth: usize,
    /// Code block info string (e.g., language)
    code_block_info: Option<String>,
    /// Extra indentation to apply to code content lines (e.g., indented code blocks)
    code_block_content_indent: usize,
    /// Whether we're currently in a block quote
    in_block_quote: bool,
    /// Current block quote nesting depth
    block_quote_depth: usize,
    /// Prefix character for block quote
    block_quote_prefix: char,
    /// Whether we're currently in a list
    in_list: bool,
    /// Whether the current list is ordered (numbered) vs unordered (bullet)
    list_is_ordered: bool,
    /// Current item number for ordered lists
    list_item_number: u64,
    /// Stack of list state for nested lists: (ordered, next item number)
    list_stack: Vec<(bool, u64)>,
    /// Current list nesting depth
    list_depth: usize,
    /// Marker for the next unordered list item ('-', '*', '+')
    pending_unordered_list_marker: Option<char>,
    /// Number marker for the next ordered list item (e.g. `3.`)
    pending_ordered_list_number: Option<u64>,
    /// Whether to preserve a source blank line before the next list item
    pending_list_item_blank_before: bool,
    /// Number of source blank lines to insert before the next block
    pending_interblock_blank_lines: usize,
    /// Display width used to indent wrapped lines within the current list item
    list_item_continuation_indent: usize,
    /// Number of paragraph starts seen in current list item
    list_item_paragraph_count: usize,
    /// Whether we're currently inside a table
    in_table: bool,
    /// Current table alignments
    table_alignments: Vec<crate::parsing::CellAlignment>,
    /// Accumulated table rows
    table_rows: Vec<Vec<String>>,
    /// Row currently being built
    current_table_row: Vec<String>,
    /// Cell currently being built
    current_table_cell: String,
    /// Whether we're currently in a table cell
    in_table_cell: bool,
    /// Whether we're inside a definition list
    in_definition_list: bool,
    /// Whether we're rendering a definition entry
    in_definition_entry: bool,
    /// Whether we're currently inside a footnote definition block
    in_footnote_definition: bool,
    /// Whether the footnote section separator has already been rendered
    footnote_section_started: bool,
    /// Whether we're currently inside an HTML block
    in_html_block: bool,
    /// Whether the next heading should be rendered underlined (Setext source)
    pending_heading_setext: bool,
    /// Whether the current heading is underlined
    current_heading_underlined: bool,
}

impl Renderer {
    fn needs_space_before_token(token: &str, prev_visible_char: Option<char>) -> bool {
        // Punctuation should "stick" to the previous token.
        match token.chars().next() {
            Some(':') | Some(';') | Some(',') | Some('.') | Some(')') | Some(']') | Some('}')
            | Some('%') => !prev_visible_char
                .map(|c| c.is_alphanumeric())
                .unwrap_or(false),
            _ => true,
        }
    }

    fn maybe_insert_space_before_inline(&mut self) {
        if self.cursor_col == 0 || self.current_line.ends_with(' ') {
            return;
        }
        // Don't inject a space after opening punctuation.
        if self.current_line.ends_with('(')
            || self.current_line.ends_with('[')
            || self.current_line.ends_with('{')
            || self.current_line.ends_with('/')
            || self.current_line.ends_with('!')
        {
            return;
        }
        self.current_line.push(' ');
        self.cursor_col += 1;
    }

    fn table_cell_has_visible_content(&self) -> bool {
        !Self::strip_ansi(&self.current_table_cell).is_empty()
    }

    fn table_cell_ends_with_visible_space(&self) -> bool {
        Self::strip_ansi(&self.current_table_cell).ends_with(' ')
    }

    fn block_quote_prefix_string(&self) -> String {
        if self.block_quote_depth == 0 {
            String::new()
        } else {
            let mut s = String::new();
            for _ in 0..self.block_quote_depth {
                s.push(self.block_quote_prefix);
                s.push(' ');
            }
            s
        }
    }

    fn push_block_quote_prefix(&mut self) {
        let prefix = self.block_quote_prefix_string();
        self.current_line.push_str(&prefix);
        self.cursor_col += Self::display_width(&prefix);
    }

    fn ends_with_open_html_tag(text: &str) -> bool {
        let cleaned = Self::strip_ansi(text);
        let trimmed = cleaned.trim_end();
        if !trimmed.ends_with('>') {
            return false;
        }
        let Some(last_lt) = trimmed.rfind('<') else {
            return false;
        };
        let Some(last_gt) = trimmed.rfind('>') else {
            return false;
        };
        if last_lt >= last_gt {
            return false;
        }
        let tag = &trimmed[last_lt..=last_gt];
        tag.starts_with('<')
            && !tag.starts_with("</")
            && !tag.starts_with("<!")
            && !tag.starts_with("<?")
    }

    fn display_width(text: &str) -> usize {
        text.chars().map(Self::char_display_width).sum()
    }

    fn char_display_width(ch: char) -> usize {
        if ch == '\u{00AD}' || ch == '\u{200B}' {
            return 0;
        }
        let c = ch as u32;
        if (0x0300..=0x036F).contains(&c)
            || (0x1AB0..=0x1AFF).contains(&c)
            || (0x1DC0..=0x1DFF).contains(&c)
            || (0x20D0..=0x20FF).contains(&c)
            || (0xFE20..=0xFE2F).contains(&c)
        {
            return 0;
        }
        if ch.is_ascii() {
            return 1;
        }
        if (0x1100..=0x115F).contains(&c)
            || (0x2329..=0x232A).contains(&c)
            || (0x2E80..=0xA4CF).contains(&c)
            || (0xAC00..=0xD7A3).contains(&c)
            || (0xF900..=0xFAFF).contains(&c)
            || (0xFE10..=0xFE19).contains(&c)
            || (0xFE30..=0xFE6F).contains(&c)
            || (0xFF00..=0xFF60).contains(&c)
            || (0xFFE0..=0xFFE6).contains(&c)
            || (0x1F300..=0x1FAFF).contains(&c)
            || (0x20000..=0x3FFFD).contains(&c)
        {
            2
        } else {
            1
        }
    }

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
            strikethrough_fallback: true,
            in_strikethrough: false,
            bold_depth: 0,
            italics_depth: 0,
            strikethrough_depth: 0,
            code_block_info: None,
            code_block_content_indent: 0,
            in_block_quote: false,
            block_quote_depth: 0,
            block_quote_prefix: '│',
            in_list: false,
            list_is_ordered: false,
            list_item_number: 0,
            list_stack: Vec::new(),
            list_depth: 0,
            pending_unordered_list_marker: None,
            pending_ordered_list_number: None,
            pending_list_item_blank_before: false,
            pending_interblock_blank_lines: 0,
            list_item_continuation_indent: 0,
            list_item_paragraph_count: 0,
            in_table: false,
            table_alignments: Vec::new(),
            table_rows: Vec::new(),
            current_table_row: Vec::new(),
            current_table_cell: String::new(),
            in_table_cell: false,
            in_definition_list: false,
            in_definition_entry: false,
            in_footnote_definition: false,
            footnote_section_started: false,
            in_html_block: false,
            pending_heading_setext: false,
            current_heading_underlined: false,
        }
    }

    fn reset_state(&mut self) {
        // Reset state
        self.lines.clear();
        self.current_line.clear();
        self.cursor_col = 0;
        self.heading_level = 0;
        self.heading_text.clear();
        self.in_code_block = false;
        self.in_strikethrough = false;
        self.bold_depth = 0;
        self.italics_depth = 0;
        self.strikethrough_depth = 0;
        self.code_block_info = None;
        self.code_block_content_indent = 0;
        self.in_block_quote = false;
        self.block_quote_depth = 0;
        self.in_list = false;
        self.list_is_ordered = false;
        self.list_item_number = 0;
        self.list_stack.clear();
        self.list_depth = 0;
        self.pending_unordered_list_marker = None;
        self.pending_ordered_list_number = None;
        self.pending_list_item_blank_before = false;
        self.pending_interblock_blank_lines = 0;
        self.list_item_continuation_indent = 0;
        self.list_item_paragraph_count = 0;
        self.in_table = false;
        self.table_alignments.clear();
        self.table_rows.clear();
        self.current_table_row.clear();
        self.current_table_cell.clear();
        self.in_table_cell = false;
        self.in_definition_list = false;
        self.in_definition_entry = false;
        self.in_footnote_definition = false;
        self.footnote_section_started = false;
        self.in_html_block = false;
        self.pending_heading_setext = false;
        self.current_heading_underlined = false;
    }

    /// Render markdown events to lines.
    pub fn render_lines(&mut self, events: &[Event]) -> Vec<String> {
        self.reset_state();
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

        self.lines.clone()
    }

    /// Render markdown events to a string
    pub fn render(&mut self, events: &[Event]) -> String {
        self.render_lines(events).join("\n")
    }

    /// Enable or disable Unicode fallback for strikethrough.
    pub fn set_strikethrough_fallback(&mut self, enabled: bool) {
        self.strikethrough_fallback = enabled;
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
        if self.in_html_block {
            match event {
                Event::Text(text) => {
                    if !self.current_line.is_empty() {
                        self.render_newline();
                    }
                    self.render_html_block_text(text);
                    return;
                }
                Event::SoftBreak | Event::HardBreak => {
                    self.render_newline();
                    return;
                }
                Event::RawHtml(html) => {
                    if !self.current_line.is_empty() {
                        self.render_newline();
                    }
                    self.render_raw_html(html);
                    return;
                }
                _ => {}
            }
        }

        match event {
            Event::Text(text) => self.render_text(text),
            Event::SoftBreak => self.render_space(),
            Event::HardBreak => self.render_newline(),
            Event::Rule(marker) => self.render_rule(*marker),
            Event::Start(block) => self.render_block_start(block),
            Event::End(block) => self.render_block_end(block),
            Event::StrongStart => self.render_bold_start(),
            Event::StrongEnd => self.render_bold_end(),
            Event::EmphasisStart => self.render_italics_start(),
            Event::EmphasisEnd => self.render_italics_end(),
            Event::StrikethroughStart => self.render_strikethrough_start(),
            Event::StrikethroughEnd => self.render_strikethrough_end(),
            Event::InlineCode(code) => self.render_inline_code(code),
            Event::RawHtml(html) => self.render_raw_html(html),
            Event::Link { text, url, source } => self.render_link(text, url, source),
            Event::Image { alt, url, source } => self.render_image(alt, url, source),
            Event::TaskListMarker(checked) => self.render_task_marker(*checked),
            Event::FootnoteReference(label) => self.render_footnote_reference(label),
            Event::ListItemMarker(marker) => self.pending_unordered_list_marker = Some(*marker),
            Event::OrderedListItemMarker(number) => {
                self.pending_ordered_list_number = Some(*number)
            }
            Event::ListItemPrecededByBlankLine => self.pending_list_item_blank_before = true,
            Event::InterBlockBlankLines(count) => self.render_interblock_blank_lines(*count),
            Event::ReferenceDefinition(line) => self.render_reference_definition(line),
            Event::HeadingSetext(is_setext) => self.pending_heading_setext = *is_setext,
        }
    }

    fn render_interblock_blank_lines(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        if !self.current_line.is_empty() {
            self.lines.push(self.current_line.clone());
            self.current_line.clear();
            self.cursor_col = 0;
        }
        self.pending_interblock_blank_lines = count;
    }

    fn render_html_block_text(&mut self, text: &str) {
        let segments: Vec<&str> = text.split_terminator('\n').collect();
        for (idx, segment) in segments.iter().enumerate() {
            self.current_line.push_str(segment);
            self.cursor_col += Self::display_width(segment);
            if idx + 1 < segments.len() {
                self.render_newline();
            }
        }
        if text.ends_with('\n') {
            self.render_newline();
        }
    }

    fn render_reference_definition(&mut self, line: &str) {
        if !self.current_line.is_empty() {
            self.lines.push(self.current_line.clone());
            self.current_line.clear();
            self.cursor_col = 0;
        }
        if self.pending_interblock_blank_lines > 0 {
            let trailing_empty = self
                .lines
                .iter()
                .rev()
                .take_while(|line| line.is_empty())
                .count();
            if trailing_empty < self.pending_interblock_blank_lines {
                for _ in 0..(self.pending_interblock_blank_lines - trailing_empty) {
                    self.lines.push(String::new());
                }
            }
            self.pending_interblock_blank_lines = 0;
        }
        let rendered = Self::render_reference_definition_with_underlined_url(line)
            .unwrap_or_else(|| line.to_string());
        self.lines.push(rendered);
    }

    /// Render text content with word wrapping
    fn render_text(&mut self, text: &str) {
        if self.in_code_block {
            self.render_code_text(text);
            return;
        }

        let mut processed = Self::smart_punctuation(text);
        if self.strikethrough_fallback && self.in_strikethrough {
            processed = Self::apply_strikethrough_fallback(&processed);
        }

        if let Some(rendered_ref_def) =
            Self::render_reference_definition_with_underlined_url(&processed)
        {
            if self.in_table_cell {
                self.current_table_cell.push_str(&rendered_ref_def);
            } else {
                self.current_line.push_str(&rendered_ref_def);
                self.cursor_col += Self::display_width(&Self::strip_ansi(&rendered_ref_def));
            }
            return;
        }

        if self.in_table_cell {
            for word in processed.split_whitespace() {
                if self.table_cell_has_visible_content() && !self.table_cell_ends_with_visible_space()
                {
                    self.current_table_cell.push(' ');
                }
                self.current_table_cell.push_str(word);
            }
            return;
        }

        let fragment_has_leading_ws = processed
            .chars()
            .next()
            .map(|c| c.is_whitespace())
            .unwrap_or(false);
        let fragment_has_trailing_ws = processed
            .chars()
            .last()
            .map(|c| c.is_whitespace())
            .unwrap_or(false);

        for (idx, word) in processed.split_whitespace().enumerate() {
            let word_width = Self::display_width(word);
            let should_have_space_before = if idx == 0 {
                fragment_has_leading_ws
            } else {
                true
            };
            let prev_visible_char = Self::last_visible_char(&self.current_line);
            let token_needs_space = Self::needs_space_before_token(word, prev_visible_char);
            let needs_leading_space = self.cursor_col > 0
                && should_have_space_before
                && !Self::ends_with_visible_space(&self.current_line)
                && token_needs_space;

            // Check if we need to wrap to a new line
            if self.cursor_col > 0
                && self.cursor_col + word_width + usize::from(needs_leading_space) > self.width
                && needs_leading_space
            {
                self.render_newline();
            }

            // Add word to current line
            if self.cursor_col > 0
                && should_have_space_before
                && !Self::ends_with_visible_space(&self.current_line)
                && token_needs_space
                && !Self::ends_with_open_html_tag(&self.current_line)
            {
                self.current_line.push(' ');
                self.cursor_col += 1;
            }
            self.current_line.push_str(word);
            self.cursor_col += word_width;
        }

        if fragment_has_trailing_ws && !Self::ends_with_visible_space(&self.current_line) {
            if self.cursor_col >= self.width && self.width > 0 {
                self.render_newline();
            }
            if self.cursor_col == 0 || !Self::ends_with_visible_space(&self.current_line) {
                self.current_line.push(' ');
                self.cursor_col += 1;
            }
        }
    }

    fn render_code_text(&mut self, text: &str) {
        let segments: Vec<&str> = text.split_terminator('\n').collect();
        for (idx, segment) in segments.iter().enumerate() {
            self.current_line.push_str(segment);
            self.cursor_col += Self::display_width(segment);
            if idx + 1 < segments.len() {
                self.render_newline();
            }
        }
        if text.ends_with('\n') {
            self.render_newline();
        }
    }

    /// Render a space (soft break between words)
    fn render_space(&mut self) {
        if self.in_code_block {
            self.render_newline();
            return;
        }

        if self.in_table_cell {
            if self.table_cell_has_visible_content() && !self.table_cell_ends_with_visible_space() {
                self.current_table_cell.push(' ');
            }
            return;
        }

        // Preserve source soft-break line boundaries inside list items.
        if self.in_list && self.list_item_continuation_indent > 0 {
            self.render_newline();
            return;
        }

        // Within a block quote, soft breaks become new lines with prefix
        if self.in_block_quote {
            // Flush current line
            if !self.current_line.is_empty() {
                self.lines.push(self.current_line.clone());
                self.current_line.clear();
            }
            // Add block quote prefix for new line
            self.push_block_quote_prefix();
        } else if self.cursor_col > 0 && self.cursor_col < self.width {
            self.current_line.push(' ');
            self.cursor_col += 1;
        }
    }

    /// Render bold text start (ANSI bold on)
    fn render_bold_start(&mut self) {
        if self.bold_depth == 0 {
            if self.in_table_cell {
                self.current_table_cell.push_str("\x1b[1m");
            } else {
                self.current_line.push_str("\x1b[1m");
            }
        }
        self.bold_depth += 1;
    }

    /// Render bold text end (ANSI bold off)
    fn render_bold_end(&mut self) {
        if self.bold_depth > 0 {
            self.bold_depth -= 1;
        }
        if self.bold_depth == 0 {
            if self.in_table_cell {
                self.current_table_cell.push_str("\x1b[22m");
            } else {
                self.current_line.push_str("\x1b[22m");
            }
        }
    }

    /// Render italics text start (ANSI italics on)
    fn render_italics_start(&mut self) {
        if self.italics_depth == 0 {
            if self.in_table_cell {
                self.current_table_cell.push_str("\x1b[3m");
            } else {
                self.current_line.push_str("\x1b[3m");
            }
        }
        self.italics_depth += 1;
    }

    /// Render italics text end (ANSI italics off)
    fn render_italics_end(&mut self) {
        if self.italics_depth > 0 {
            self.italics_depth -= 1;
        }
        if self.italics_depth == 0 {
            if self.in_table_cell {
                self.current_table_cell.push_str("\x1b[23m");
            } else {
                self.current_line.push_str("\x1b[23m");
            }
        }
    }

    /// Render strikethrough text start (ANSI strikethrough on)
    fn render_strikethrough_start(&mut self) {
        self.strikethrough_depth += 1;
        if self.strikethrough_fallback {
            self.in_strikethrough = true;
        } else if self.strikethrough_depth == 1 {
            if self.in_table_cell {
                self.current_table_cell.push_str("\x1b[9m");
            } else {
                self.current_line.push_str("\x1b[9m");
            }
        }
    }

    /// Render strikethrough text end (ANSI reset)
    fn render_strikethrough_end(&mut self) {
        if self.strikethrough_depth > 0 {
            self.strikethrough_depth -= 1;
        }
        if self.strikethrough_fallback {
            self.in_strikethrough = self.strikethrough_depth > 0;
        } else if self.strikethrough_depth == 0 {
            if self.in_table_cell {
                self.current_table_cell.push_str("\x1b[29m");
            } else {
                self.current_line.push_str("\x1b[29m");
            }
        }
    }

    /// Render inline code with backticks and monospace indication
    fn render_inline_code(&mut self, code: &str) {
        let display_code = if self.strikethrough_fallback && self.in_strikethrough {
            Self::apply_strikethrough_fallback(code)
        } else {
            code.to_string()
        };
        let delim = Self::inline_code_delimiter(code);
        let needs_padding = display_code.starts_with('`')
            || display_code.ends_with('`')
            || display_code.starts_with(' ')
            || display_code.ends_with(' ');
        let serialized_code = if needs_padding {
            format!(" {display_code} ")
        } else {
            display_code.clone()
        };

        if self.in_table_cell {
            self.current_table_cell.push_str(&delim);
            self.current_table_cell.push_str(&serialized_code);
            self.current_table_cell.push_str(&delim);
            return;
        }

        self.maybe_insert_space_before_inline();

        // Add backticks and use faint (dim) for monospace indication
        self.current_line.push_str(&delim);
        self.current_line.push_str("\x1b[2m"); // Faint for monospace
        self.current_line.push_str(&serialized_code);
        self.current_line.push_str("\x1b[0m"); // Reset
        self.current_line.push_str(&delim);
        self.cursor_col += (2 * delim.len()) + Self::display_width(&serialized_code);
    }

    fn render_raw_html(&mut self, html: &str) {
        if self.in_table_cell {
            self.current_table_cell.push_str(html);
            return;
        }

        if !self.in_html_block && !html.starts_with('\n') && !html.starts_with("</") {
            self.maybe_insert_space_before_inline();
        }

        let is_comment = Self::is_html_comment(html);
        let mut segments = html.split_terminator('\n').peekable();
        while let Some(segment) = segments.next() {
            if is_comment {
                self.current_line.push_str("\x1b[2m");
            }
            self.current_line.push_str(segment);
            if is_comment {
                self.current_line.push_str("\x1b[22m");
            }
            self.cursor_col += Self::display_width(segment);
            if segments.peek().is_some() {
                self.render_newline();
            }
        }
    }

    /// Render a link as "text (url)"
    fn render_link(&mut self, text: &str, url: &str, source: &str) {
        if !source.is_empty() {
            let rendered = Self::render_link_source_with_underlined_url(source, url);
            if self.in_table_cell {
                self.current_table_cell.push_str(&rendered);
                return;
            }

            self.maybe_insert_space_before_inline();
            self.current_line.push_str(&rendered);
            self.cursor_col += Self::display_width(&Self::strip_ansi(&rendered));
            return;
        }

        let rendered_url = Self::underline_with_ansi(url);
        if self.in_table_cell {
            if !url.is_empty() && text == url {
                self.current_table_cell.push('<');
                self.current_table_cell.push_str(url);
                self.current_table_cell.push('>');
            } else {
                self.current_table_cell.push('[');
                self.current_table_cell.push_str(text);
                self.current_table_cell.push(']');
                if !url.is_empty() {
                    self.current_table_cell.push('(');
                    self.current_table_cell.push_str(url);
                    self.current_table_cell.push(')');
                }
            }
            return;
        }

        self.maybe_insert_space_before_inline();

        // Autolink form: don't duplicate "url (url)".
        if !url.is_empty() && text == url {
            self.current_line.push('<');
            self.current_line.push_str(&rendered_url);
            self.current_line.push('>');
            self.cursor_col += 2 + Self::display_width(url);
            return;
        }

        // Add markdown-like link text form.
        self.current_line.push('[');
        self.cursor_col += 1;
        self.render_text(text);
        self.current_line.push(']');
        self.cursor_col += 1;

        // Add URL in markdown parentheses.
        if !url.is_empty() {
            self.current_line.push('(');
            self.current_line.push_str(&rendered_url);
            self.current_line.push(')');
            self.cursor_col += 2 + Self::display_width(url); // "(" + url + ")"
        }
    }

    fn render_link_source_with_underlined_url(source: &str, url: &str) -> String {
        if source.starts_with("![") {
            return source.to_string();
        }

        if source.starts_with('<') && source.ends_with('>') && !url.is_empty() {
            return format!("<{}>", Self::underline_with_ansi(url));
        }

        if let Some(open) = source.find("](") {
            let close = source.rfind(')').unwrap_or(source.len());
            if close > open + 2 {
                let inside = &source[open + 2..close];
                let mut split_at = inside.len();
                let mut in_quote = false;
                for (i, ch) in inside.char_indices() {
                    if ch == '"' {
                        in_quote = !in_quote;
                    }
                    if !in_quote && ch.is_whitespace() {
                        split_at = i;
                        break;
                    }
                }
                let link_dest = &inside[..split_at];
                if !link_dest.is_empty() {
                    let mut out = String::new();
                    out.push_str(&source[..open + 2]);
                    out.push_str(&Self::underline_with_ansi(link_dest));
                    out.push_str(&inside[split_at..]);
                    out.push_str(&source[close..]);
                    return out;
                }
            }
        }

        source.to_string()
    }

    /// Render an image preserving source markdown form when available.
    fn render_image(&mut self, alt: &str, _url: &str, source: &str) {
        if !source.is_empty() {
            if self.in_table_cell {
                self.current_table_cell.push_str(source);
                return;
            }

            self.maybe_insert_space_before_inline();
            self.current_line.push_str(source);
            self.cursor_col += Self::display_width(source);
            return;
        }

        if self.in_table_cell {
            self.current_table_cell.push('[');
            self.current_table_cell.push_str(alt);
            self.current_table_cell.push(']');
            return;
        }

        self.maybe_insert_space_before_inline();

        // Show alt text in brackets to indicate it's an image, not actual image rendering
        self.current_line.push('[');
        self.current_line.push_str(alt);
        self.current_line.push(']');
        self.cursor_col += 2 + Self::display_width(alt); // "[" + alt + "]"
    }

    /// Render a task list checkbox marker
    fn render_task_marker(&mut self, checked: bool) {
        let marker = if checked { "[✔] " } else { "[ ] " };
        if self.in_table_cell {
            self.current_table_cell.push_str(marker);
            return;
        }
        self.current_line.push_str(marker);
        self.cursor_col += Self::display_width(marker);
    }

    /// Render a footnote reference inline.
    fn render_footnote_reference(&mut self, label: &str) {
        let text = format!("[^{}]", label);
        if self.in_table_cell {
            self.current_table_cell.push_str(&text);
            return;
        }
        self.maybe_insert_space_before_inline();
        self.current_line.push_str(&text);
        self.cursor_col += Self::display_width(&text);
    }

    /// Render a hard line break
    fn render_newline(&mut self) {
        // Flush current line
        if !self.current_line.is_empty() {
            if self.in_footnote_definition {
                self.current_line.push_str("\x1b[0m");
            }
            self.lines.push(self.current_line.clone());
            self.current_line.clear();
        } else {
            // Empty line
            self.lines.push(String::new());
        }
        self.cursor_col = 0;

        // If we're in a block quote, add prefix to new line
        if self.in_block_quote {
            self.push_block_quote_prefix();
        } else if self.in_list && self.list_item_continuation_indent > 0 {
            let indent = " ".repeat(self.list_item_continuation_indent);
            self.current_line.push_str(&indent);
            self.cursor_col = self.list_item_continuation_indent;
        }
        if self.in_code_block {
            self.current_line.push_str("\x1b[2m");
            if self.code_block_content_indent > 0 {
                let indent = " ".repeat(self.code_block_content_indent);
                self.current_line.push_str(&indent);
                self.cursor_col += self.code_block_content_indent;
            }
        }
        if self.in_footnote_definition {
            self.current_line.push_str("\x1b[2m");
        }
        self.reapply_active_inline_styles();
    }

    fn reapply_active_inline_styles(&mut self) {
        if self.in_code_block {
            return;
        }
        if self.bold_depth > 0 {
            self.current_line.push_str("\x1b[1m");
        }
        if self.italics_depth > 0 {
            self.current_line.push_str("\x1b[3m");
        }
        if !self.strikethrough_fallback && self.strikethrough_depth > 0 {
            self.current_line.push_str("\x1b[9m");
        }
    }

    /// Render a horizontal rule
    fn render_rule(&mut self, marker: char) {
        // Flush current line first
        if !self.current_line.is_empty() {
            self.lines.push(self.current_line.clone());
            self.current_line.clear();
            self.cursor_col = 0;
        }
        if !self.lines.is_empty() && !self.lines.last().is_some_and(String::is_empty) {
            self.lines.push(String::new());
        }

        let ch = match marker {
            '*' => '═',
            '_' => '┄',
            _ => '─',
        };
        let rule_width = (self.width / 2).max(3);
        let left_pad = self.width.saturating_sub(rule_width) / 2;
        let mut line = String::new();
        line.push_str(&" ".repeat(left_pad));
        line.push_str("\x1b[2m");
        line.push_str(&ch.to_string().repeat(rule_width));
        line.push_str("\x1b[0m");
        self.lines.push(line);
        self.lines.push(String::new());
    }

    /// Handle the start of a block element
    fn render_block_start(&mut self, block: &crate::parsing::Block) {
        if matches!(block, crate::parsing::Block::Paragraph) && self.in_block_quote {
            if self.current_line.is_empty() {
                if let Some(last_line) = self.lines.last() {
                    if Self::is_quote_line(last_line) && !Self::is_quote_prefix_only(last_line) {
                        self.lines
                            .push(self.block_quote_prefix_string().trim_end().to_string());
                    }
                }
                if self.in_list && self.list_item_continuation_indent > 0 {
                    let indent = " ".repeat(self.list_item_continuation_indent);
                    self.current_line.push_str(&indent);
                    self.cursor_col = self.list_item_continuation_indent;
                }
                self.push_block_quote_prefix();
            }
            return;
        }

        if matches!(block, crate::parsing::Block::Paragraph) && self.in_list {
            if self.list_item_paragraph_count > 0
                && !self.lines.is_empty()
                && !self.lines.last().is_some_and(String::is_empty)
            {
                self.lines.push(String::new());
            }
            if self.current_line.is_empty() && self.list_item_continuation_indent > 0 {
                let indent = " ".repeat(self.list_item_continuation_indent);
                self.current_line.push_str(&indent);
                self.cursor_col = self.list_item_continuation_indent;
            }
            self.list_item_paragraph_count += 1;
            return;
        }

        if matches!(block, crate::parsing::Block::Paragraph) && self.in_footnote_definition {
            // Paragraphs inside footnote definitions should continue on the
            // same line as the footnote label prefix.
            return;
        }

        if matches!(block, crate::parsing::Block::Paragraph)
            && self.in_list
            && !self.current_line.is_empty()
        {
            // Paragraphs inside list items should continue on the current item line.
            return;
        }

        let is_table_internal = matches!(
            block,
            crate::parsing::Block::TableHead
                | crate::parsing::Block::TableRow
                | crate::parsing::Block::TableCell
        );
        let is_nested_list_start =
            matches!(block, crate::parsing::Block::List { .. }) && self.in_list;

        if !is_table_internal && !is_nested_list_start {
            // Flush any pending content
            if !self.current_line.is_empty() {
                self.lines.push(self.current_line.clone());
                self.current_line.clear();
                self.cursor_col = 0;
            }

            let wants_blank = matches!(
                block,
                crate::parsing::Block::Heading { .. }
                    | crate::parsing::Block::Paragraph
                    | crate::parsing::Block::BlockQuote
                    | crate::parsing::Block::CodeBlock { .. }
                    | crate::parsing::Block::List { .. }
                    | crate::parsing::Block::Table { .. }
                    | crate::parsing::Block::DefinitionList
                    | crate::parsing::Block::HtmlBlock
            );

            // Add a blank line only for major block boundaries.
            if wants_blank && !self.lines.is_empty() {
                let desired_blank_lines = if self.pending_interblock_blank_lines > 0 {
                    self.pending_interblock_blank_lines
                } else {
                    0
                };
                self.pending_interblock_blank_lines = 0;

                if desired_blank_lines == 0 {
                    // Keep consecutive headings compact when source has no blank line.
                } else if self.in_block_quote {
                    let trailing_quote_blank = self
                        .lines
                        .iter()
                        .rev()
                        .take_while(|line| Self::is_quote_prefix_only(line))
                        .count();
                    if trailing_quote_blank < desired_blank_lines {
                        for _ in 0..(desired_blank_lines - trailing_quote_blank) {
                            self.lines
                                .push(self.block_quote_prefix_string().trim_end().to_string());
                        }
                    }
                } else {
                    let trailing_empty = self
                        .lines
                        .iter()
                        .rev()
                        .take_while(|line| line.is_empty())
                        .count();
                    if trailing_empty < desired_blank_lines {
                        for _ in 0..(desired_blank_lines - trailing_empty) {
                            self.lines.push(String::new());
                        }
                    }
                }
            }
        }

        match block {
            crate::parsing::Block::Table { alignments } => {
                self.in_table = true;
                self.table_alignments = alignments.clone();
                self.table_rows.clear();
                self.current_table_row.clear();
                self.current_table_cell.clear();
                self.in_table_cell = false;
            }
            crate::parsing::Block::TableHead => {
                self.current_table_row.clear();
            }
            crate::parsing::Block::TableRow => {
                self.current_table_row.clear();
            }
            crate::parsing::Block::TableCell => {
                self.current_table_cell.clear();
                self.in_table_cell = true;
            }
            crate::parsing::Block::FootnoteDefinition { label } => {
                if !self.footnote_section_started {
                    if !self.lines.is_empty() && !self.lines.last().is_some_and(String::is_empty) {
                        self.lines.push(String::new());
                    }
                    let heading = Self::footnote_separator_line(self.width);
                    self.lines.push(format!("\x1b[2m{heading}\x1b[0m"));
                    self.footnote_section_started = true;
                }
                self.in_footnote_definition = true;
                self.current_line.push_str("\x1b[2m");
                let prefix = format!("[^{}] ", label);
                self.current_line.push_str(&prefix);
                self.cursor_col += Self::display_width(&prefix);
            }
            crate::parsing::Block::DefinitionList => {
                self.in_definition_list = true;
            }
            crate::parsing::Block::DefinitionListTitle => {}
            crate::parsing::Block::DefinitionListDefinition => {
                self.in_definition_entry = true;
                self.current_line.push_str("  ");
                self.cursor_col += 2;
            }
            crate::parsing::Block::HtmlBlock => {
                self.in_html_block = true;
            }
            _ => {}
        }

        if is_table_internal
            || matches!(
                block,
                crate::parsing::Block::Table { .. }
                    | crate::parsing::Block::FootnoteDefinition { .. }
                    | crate::parsing::Block::DefinitionList
                    | crate::parsing::Block::DefinitionListTitle
                    | crate::parsing::Block::DefinitionListDefinition
            )
        {
            return;
        }

        match block {
            crate::parsing::Block::Heading { level, text: _ } => {
                if self.in_block_quote && self.current_line.is_empty() {
                    self.push_block_quote_prefix();
                }
                // Track heading level and start bold
                self.heading_level = *level;
                self.heading_text.clear();
                // Start heading style
                self.render_bold_start();
                self.current_heading_underlined = self.pending_heading_setext;
                self.pending_heading_setext = false;
                if self.current_heading_underlined {
                    self.current_line.push_str("\x1b[4m");
                }
                // Keep Markdown heading markers visible.
                let marker = "#".repeat(usize::from(*level));
                self.current_line.push_str(&marker);
                self.current_line.push(' ');
                self.cursor_col += marker.chars().count() + 1;
            }
            crate::parsing::Block::Paragraph => {
                // Start of paragraph - no special handling needed
            }
            crate::parsing::Block::BlockQuote => {
                // Track block quote state
                self.block_quote_depth += 1;
                self.in_block_quote = true;
            }
            crate::parsing::Block::CodeBlock { info, indented } => {
                if self.in_list
                    && self.current_line.is_empty()
                    && self.list_item_continuation_indent > 0
                {
                    let indent = " ".repeat(self.list_item_continuation_indent);
                    self.current_line.push_str(&indent);
                    self.cursor_col = self.list_item_continuation_indent;
                }
                if self.in_block_quote && self.current_line.is_empty() {
                    self.push_block_quote_prefix();
                }
                // Track code block state
                self.in_code_block = true;
                self.code_block_info = info.clone();
                self.code_block_content_indent = if *indented { 4 } else { 0 };
                if *indented {
                    // Indented code blocks should not be normalized to fenced form.
                    self.current_line.push_str("\x1b[2m");
                    if self.code_block_content_indent > 0 {
                        let indent = " ".repeat(self.code_block_content_indent);
                        self.current_line.push_str(&indent);
                        self.cursor_col += self.code_block_content_indent;
                    }
                } else {
                    // Emit opening fence and start a new line for code content.
                    if let Some(ref lang) = info {
                        self.current_line.push_str("```");
                        self.current_line.push_str(lang);
                    } else {
                        self.current_line.push_str("```");
                    }
                    self.cursor_col = Self::visible_display_width_without_ansi(&self.current_line);
                    self.render_newline();
                }
            }
            crate::parsing::Block::List { start } => {
                let is_ordered = start.is_some();
                let next_number = start.unwrap_or(1);
                self.list_stack.push((is_ordered, next_number));
                self.in_list = true;
                self.list_depth = self.list_stack.len().saturating_sub(1);
                self.list_is_ordered = is_ordered;
                self.list_item_number = next_number;
            }
            crate::parsing::Block::ListItem => {
                if self.pending_list_item_blank_before {
                    if !self.lines.is_empty() && !self.lines.last().is_some_and(String::is_empty) {
                        if self.in_block_quote {
                            self.lines
                                .push(self.block_quote_prefix_string().trim_end().to_string());
                        } else {
                            self.lines.push(String::new());
                        }
                    }
                    self.pending_list_item_blank_before = false;
                }
                self.list_item_paragraph_count = 0;
                if self.in_block_quote && self.current_line.is_empty() {
                    self.push_block_quote_prefix();
                }
                // Add list item prefix with proper indentation
                // Indentation is 2 spaces per depth level
                let indentation = "  ".repeat(self.list_depth);
                self.current_line.push_str(&indentation);

                // Add the marker
                let marker = if let Some((ordered, next_number)) = self.list_stack.last_mut() {
                    self.list_is_ordered = *ordered;
                    if *ordered {
                        let display_number = self
                            .pending_ordered_list_number
                            .take()
                            .unwrap_or(*next_number);
                        let marker = format!("{display_number}. ");
                        *next_number = display_number.saturating_add(1);
                        self.list_item_number = *next_number;
                        marker
                    } else {
                        match self.pending_unordered_list_marker.take().unwrap_or('*') {
                            '-' => "- ".to_string(),
                            '+' | '*' => "• ".to_string(),
                            _ => "• ".to_string(),
                        }
                    }
                } else if self.list_is_ordered {
                    let display_number = self
                        .pending_ordered_list_number
                        .take()
                        .unwrap_or(self.list_item_number);
                    let marker = format!("{display_number}. ");
                    self.list_item_number = display_number.saturating_add(1);
                    marker
                } else {
                    match self.pending_unordered_list_marker.take().unwrap_or('*') {
                        '-' => "- ".to_string(),
                        '+' | '*' => "• ".to_string(),
                        _ => "• ".to_string(),
                    }
                };
                self.current_line.push_str(&marker);

                // Wrapped lines should align with the list item content start.
                self.list_item_continuation_indent =
                    Self::display_width(&indentation) + Self::display_width(&marker);
                self.cursor_col = self.list_item_continuation_indent;
            }
            crate::parsing::Block::Table { .. }
            | crate::parsing::Block::TableHead
            | crate::parsing::Block::TableRow
            | crate::parsing::Block::TableCell
            | crate::parsing::Block::FootnoteDefinition { .. }
            | crate::parsing::Block::DefinitionList
            | crate::parsing::Block::DefinitionListTitle
            | crate::parsing::Block::DefinitionListDefinition
            | crate::parsing::Block::HtmlBlock
            | crate::parsing::Block::MetadataBlock
            | crate::parsing::Block::InlineIgnored => {}
        }
    }

    /// Handle the end of a block element
    fn render_block_end(&mut self, block: &crate::parsing::Block) {
        match block {
            crate::parsing::Block::Heading { level: _, text: _ } => {
                // End heading style.
                if self.current_heading_underlined {
                    self.current_line.push_str("\x1b[24m");
                }
                self.render_bold_end();

                if !self.current_line.is_empty() {
                    self.lines.push(self.current_line.clone());
                    self.current_line.clear();
                }
                self.cursor_col = 0;

                // Reset heading state
                self.heading_level = 0;
                self.current_heading_underlined = false;
            }
            crate::parsing::Block::CodeBlock {
                info: _,
                indented: _,
            } => {
                if self.code_block_content_indent > 0 {
                    // Indented code blocks: no opening/closing fences.
                    if !self.current_line.is_empty() {
                        let stripped = Self::strip_ansi(&self.current_line);
                        let meaningful = stripped.trim_end();
                        if !meaningful.is_empty() && !Self::is_quote_prefix_only(meaningful) {
                            let mut line = self.current_line.clone();
                            line.push_str("\x1b[0m");
                            self.lines.push(line);
                        } else if let Some(last) = self.lines.last_mut() {
                            // Trailing newline created a fresh dim-only line; close style on previous line.
                            last.push_str("\x1b[0m");
                        }
                        self.current_line.clear();
                    } else if let Some(last) = self.lines.last_mut() {
                        last.push_str("\x1b[0m");
                    }
                    self.cursor_col = 0;
                    self.in_code_block = false;
                    self.code_block_info = None;
                    self.code_block_content_indent = 0;
                    return;
                }

                // Flush any meaningful buffered code content line.
                if !self.current_line.is_empty() {
                    let stripped = Self::strip_ansi(&self.current_line);
                    let meaningful = stripped.trim_end();
                    if !meaningful.is_empty() && !Self::is_quote_prefix_only(meaningful) {
                        self.lines.push(self.current_line.clone());
                    }
                    self.current_line.clear();
                }
                self.cursor_col = 0;
                // Ensure styling is reset before rendering the closing fence line.
                self.current_line.push_str("\x1b[0m");
                if self.in_list && self.list_item_continuation_indent > 0 {
                    self.current_line
                        .push_str(&" ".repeat(self.list_item_continuation_indent));
                    self.cursor_col = self.list_item_continuation_indent;
                }
                if self.in_block_quote {
                    self.push_block_quote_prefix();
                }
                self.current_line.push_str("```");
                self.cursor_col = Self::visible_display_width_without_ansi(&self.current_line);
                self.lines.push(self.current_line.clone());
                self.current_line.clear();
                self.cursor_col = 0;
                // Reset code block state
                self.in_code_block = false;
                self.code_block_info = None;
                self.code_block_content_indent = 0;
            }
            crate::parsing::Block::BlockQuote => {
                if self.block_quote_depth > 0 {
                    self.block_quote_depth -= 1;
                }
                self.in_block_quote = self.block_quote_depth > 0;
                // Flush current line if not empty
                if !self.current_line.is_empty() {
                    self.lines.push(self.current_line.clone());
                    self.current_line.clear();
                    self.cursor_col = 0;
                }
            }
            crate::parsing::Block::List { start: _ } => {
                let _ = self.list_stack.pop();
                self.list_depth = self.list_stack.len().saturating_sub(1);
                if let Some((ordered, next)) = self.list_stack.last().copied() {
                    self.in_list = true;
                    self.list_is_ordered = ordered;
                    self.list_item_number = next;
                } else {
                    self.in_list = false;
                    self.list_is_ordered = false;
                    self.list_item_number = 0;
                    self.list_item_continuation_indent = 0;
                }
            }
            crate::parsing::Block::ListItem => {
                self.list_item_continuation_indent = 0;
                self.list_item_paragraph_count = 0;
                if !self.current_line.is_empty() {
                    self.lines.push(self.current_line.clone());
                    self.current_line.clear();
                    self.cursor_col = 0;
                }
            }
            crate::parsing::Block::TableCell => {
                self.in_table_cell = false;
                self.current_table_row
                    .push(self.current_table_cell.trim().to_string());
                self.current_table_cell.clear();
            }
            crate::parsing::Block::TableRow => {
                if !self.current_table_row.is_empty() {
                    self.table_rows.push(self.current_table_row.clone());
                    self.current_table_row.clear();
                }
            }
            crate::parsing::Block::TableHead => {
                if !self.current_table_row.is_empty() {
                    self.table_rows.push(self.current_table_row.clone());
                    self.current_table_row.clear();
                }
            }
            crate::parsing::Block::Table { .. } => {
                self.flush_table();
                self.in_table = false;
                self.table_alignments.clear();
                self.table_rows.clear();
                self.current_table_row.clear();
                self.current_table_cell.clear();
                self.in_table_cell = false;
            }
            crate::parsing::Block::DefinitionList => {
                self.in_definition_list = false;
                self.in_definition_entry = false;
            }
            crate::parsing::Block::DefinitionListDefinition => {
                self.in_definition_entry = false;
            }
            crate::parsing::Block::DefinitionListTitle => {}
            crate::parsing::Block::FootnoteDefinition { .. } => {
                if self.in_footnote_definition {
                    self.current_line.push_str("\x1b[0m");
                    self.in_footnote_definition = false;
                }
            }
            crate::parsing::Block::HtmlBlock => {
                self.in_html_block = false;
                if !self.current_line.is_empty() {
                    self.lines.push(self.current_line.clone());
                    self.current_line.clear();
                    self.cursor_col = 0;
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

    fn flush_table(&mut self) {
        if self.table_rows.is_empty() {
            return;
        }

        let col_count = self.table_rows.iter().map(Vec::len).max().unwrap_or(0);
        if col_count == 0 {
            return;
        }

        let mut widths = vec![0usize; col_count];
        for row in &self.table_rows {
            for (i, cell) in row.iter().enumerate() {
                widths[i] = widths[i].max(Self::visible_display_width_without_ansi(cell));
            }
        }
        for width in &mut widths {
            *width = (*width).max(3);
        }

        let separator = self.table_separator_line(&widths);
        let row_max_width = self
            .table_rows
            .iter()
            .map(|row| Self::visible_display_width_without_ansi(&self.table_row_line(row, &widths)))
            .max()
            .unwrap_or(0);
        let max_table_width = row_max_width.max(Self::display_width(&separator));
        if max_table_width > self.width {
            // Fallback to plain text if table is wider than render width.
            for row in &self.table_rows {
                self.lines.push(row.join("  "));
            }
            return;
        }

        let top = self.table_border_line('┌', '┬', '┐', &widths);
        let bottom = self.table_border_line('└', '┴', '┘', &widths);

        self.lines.push(top);
        for (idx, row) in self.table_rows.iter().enumerate() {
            self.lines.push(self.table_row_line(row, &widths));
            if idx == 0 {
                self.lines.push(separator.clone());
            }
        }
        self.lines.push(bottom);
    }

    fn table_row_line(&self, row: &[String], widths: &[usize]) -> String {
        let mut out = String::new();
        out.push('│');
        for (i, width) in widths.iter().enumerate() {
            let cell = row.get(i).map(String::as_str).unwrap_or("");
            let cell_w = Self::visible_display_width_without_ansi(cell);
            let pad = width.saturating_sub(cell_w);
            let alignment = self
                .table_alignments
                .get(i)
                .copied()
                .unwrap_or(crate::parsing::CellAlignment::Left);

            let (left_pad, right_pad) = match alignment {
                crate::parsing::CellAlignment::Right => (pad, 0),
                crate::parsing::CellAlignment::Center => (pad / 2, pad - (pad / 2)),
                crate::parsing::CellAlignment::None | crate::parsing::CellAlignment::Left => {
                    (0, pad)
                }
            };

            out.push(' ');
            out.push_str(&" ".repeat(left_pad));
            out.push_str(cell);
            out.push_str(&" ".repeat(right_pad));
            out.push(' ');
            out.push('│');
        }
        out
    }

    fn table_border_line(
        &self,
        left_corner: char,
        junction: char,
        right_corner: char,
        widths: &[usize],
    ) -> String {
        let mut out = String::new();
        out.push(left_corner);
        for (i, width) in widths.iter().enumerate() {
            out.push_str(&"─".repeat(*width + 2));
            if i + 1 < widths.len() {
                out.push(junction);
            }
        }
        out.push(right_corner);
        out
    }

    fn table_separator_line(&self, widths: &[usize]) -> String {
        let mut out = String::new();
        out.push('├');
        for (i, width) in widths.iter().enumerate() {
            out.push_str(&"─".repeat(*width + 2));
            if i + 1 < widths.len() {
                out.push('┼');
            }
        }
        out.push('┤');
        out
    }

    fn footnote_separator_line(width: usize) -> String {
        let title = " Footnotes ";
        let min_width = title.chars().count() + 4;
        let line_width = width.max(min_width);
        let side = (line_width - title.chars().count()) / 2;
        let left = "─".repeat(side);
        let right = "─".repeat(line_width - title.chars().count() - side);
        format!("{left}{title}{right}")
    }

    fn smart_punctuation(text: &str) -> String {
        // Preserve source text for CommonMark fidelity.
        text.to_string()
    }

    fn apply_strikethrough_fallback(text: &str) -> String {
        let mut out = String::new();
        for ch in text.chars() {
            out.push(ch);
            if ch != '\n' && ch != '\r' {
                out.push('\u{0336}');
            }
        }
        out
    }

    fn underline_with_ansi(text: &str) -> String {
        format!("\x1b[4m{text}\x1b[24m")
    }

    fn render_reference_definition_with_underlined_url(text: &str) -> Option<String> {
        let trimmed_start = text.trim_start();
        let leading = text.len().saturating_sub(trimmed_start.len());
        if !trimmed_start.starts_with('[') {
            return None;
        }
        let close = trimmed_start.find("]:")?;
        let after = &trimmed_start[close + 2..];
        let ws_len = after
            .char_indices()
            .find_map(|(i, ch)| (!ch.is_whitespace()).then_some(i))
            .unwrap_or(after.len());
        let ws = &after[..ws_len];
        let after_ws = &after[ws_len..];
        if after_ws.is_empty() {
            return None;
        }

        let (dest_token, rest) = if let Some(inner) = after_ws.strip_prefix('<') {
            let close_angle = inner.find('>')?;
            (&after_ws[..close_angle + 2], &after_ws[close_angle + 2..])
        } else {
            let end = after_ws
                .char_indices()
                .find_map(|(i, ch)| ch.is_whitespace().then_some(i))
                .unwrap_or(after_ws.len());
            (&after_ws[..end], &after_ws[end..])
        };

        if dest_token.is_empty() {
            return None;
        }

        let underlined_dest = if dest_token.starts_with('<') && dest_token.ends_with('>') {
            let inner = &dest_token[1..dest_token.len().saturating_sub(1)];
            format!("<{}>", Self::underline_with_ansi(inner))
        } else {
            Self::underline_with_ansi(dest_token)
        };

        let mut out = String::new();
        out.push_str(&text[..leading]);
        out.push_str(&trimmed_start[..close + 2]);
        out.push_str(ws);
        out.push_str(&underlined_dest);
        out.push_str(rest);
        Some(out)
    }

    fn is_html_comment(html: &str) -> bool {
        let trimmed = html.trim();
        trimmed.starts_with("<!--") && trimmed.ends_with("-->")
    }

    fn inline_code_delimiter(code: &str) -> String {
        let mut max_run = 0usize;
        let mut run = 0usize;
        for ch in code.chars() {
            if ch == '`' {
                run += 1;
                if run > max_run {
                    max_run = run;
                }
            } else {
                run = 0;
            }
        }
        "`".repeat(max_run + 1)
    }

    fn visible_display_width_without_ansi(text: &str) -> usize {
        let cleaned = Self::strip_ansi(text);
        Self::display_width(cleaned.trim_end())
    }

    fn last_visible_char(text: &str) -> Option<char> {
        Self::strip_ansi(text).chars().last()
    }

    fn strip_ansi(text: &str) -> String {
        let mut cleaned = String::new();
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' && chars.peek() == Some(&'[') {
                let _ = chars.next(); // consume '['
                for c in chars.by_ref() {
                    // End CSI sequence when reaching a final byte (e.g. 'm')
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
                continue;
            }
            cleaned.push(ch);
        }

        cleaned
    }

    fn is_quote_prefix_only(text: &str) -> bool {
        let mut s = text.trim();
        if s.is_empty() {
            return false;
        }
        let mut consumed = false;
        loop {
            let Some(rest) = s.strip_prefix('│') else {
                break;
            };
            consumed = true;
            s = rest.trim_start();
        }
        consumed && s.is_empty()
    }

    fn is_quote_line(text: &str) -> bool {
        let cleaned = Self::strip_ansi(text);
        cleaned.trim_start().starts_with('│')
    }

    fn ends_with_visible_space(text: &str) -> bool {
        Self::strip_ansi(text).ends_with(' ')
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
