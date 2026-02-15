use pulldown_cmark::{Event as MdEvent, Options, Parser, Tag, TagEnd};

/// Events produced by the markdown parser for rendering
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Start of a block element
    Start(Block),
    /// End of a block element
    End(Block),
    /// Text content
    Text(String),
    /// Soft break (space between words)
    SoftBreak,
    /// Hard break (line break)
    HardBreak,
    /// Horizontal rule with detected source marker ('-', '*', or '_')
    Rule(char),
    /// Start of strong emphasis (bold)
    StrongStart,
    /// End of strong emphasis (bold)
    StrongEnd,
    /// Start of emphasis (italics)
    EmphasisStart,
    /// End of emphasis (italics)
    EmphasisEnd,
    /// Start of strikethrough
    StrikethroughStart,
    /// End of strikethrough
    StrikethroughEnd,
    /// Inline code (backtick-enclosed)
    InlineCode(String),
    /// Raw HTML (block or inline)
    RawHtml(String),
    /// Link with text and URL
    Link {
        text: String,
        url: String,
        source: String,
    },
    /// Image with alt text and URL, preserving original markdown source when available
    Image {
        alt: String,
        url: String,
        source: String,
    },
    /// Task list checkbox marker (`- [ ]` or `- [x]`)
    TaskListMarker(bool),
    /// Footnote reference (for example, `[^1]` in markdown source)
    FootnoteReference(String),
    /// Unordered list item marker from source ('-', '*', '+')
    ListItemMarker(char),
    /// Ordered list item marker number from source (e.g. `3.` -> 3)
    OrderedListItemMarker(u64),
    /// Source had a blank line immediately before this list item line
    ListItemPrecededByBlankLine,
    /// Extra blank lines in source between adjacent block-level elements
    /// beyond the single semantic paragraph separator.
    InterBlockBlankLines(usize),
    /// Source reference definition line (e.g. `[id]: https://...`)
    ReferenceDefinition(String),
    /// Indicates whether the next heading came from Setext syntax
    HeadingSetext(bool),
}

/// Block elements in markdown
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// Heading with level (1-6)
    Heading { level: u8, text: String },
    /// Paragraph
    Paragraph,
    /// Block quote
    BlockQuote,
    /// Code block
    CodeBlock {
        info: Option<String>,
        indented: bool,
    },
    /// Unordered list
    List { start: Option<u64> },
    /// List item
    ListItem,
    /// Table container
    Table { alignments: Vec<CellAlignment> },
    /// Table header section
    TableHead,
    /// Table row
    TableRow,
    /// Table cell
    TableCell,
    /// Footnote definition block
    FootnoteDefinition { label: String },
    /// Definition list container
    DefinitionList,
    /// Definition list term/title
    DefinitionListTitle,
    /// Definition list definition/content
    DefinitionListDefinition,
    /// HTML block container
    HtmlBlock,
    /// Metadata block
    MetadataBlock,
    /// Inline-only tags that are handled through dedicated inline events
    InlineIgnored,
}

/// Table cell alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellAlignment {
    None,
    Left,
    Center,
    Right,
}

/// Parse markdown text and return an iterator of events
pub fn parse_markdown(markdown: &str) -> impl Iterator<Item = Event> + '_ {
    let parser = Parser::new_ext(
        markdown,
        Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_DEFINITION_LIST,
    );
    // Track link/image state: (is_link, is_image, url)
    let mut link_state: Option<(bool, bool, String, usize)> = None;
    let mut link_text = String::new();
    let mut list_kind_stack: Vec<bool> = Vec::new(); // true = ordered, false = unordered
    let mut out = Vec::new();
    let mut prev_end = 0usize;

    for (event, range) in parser.into_offset_iter() {
        if should_emit_inter_block_blank_lines_before_event(&event, markdown, range.start) {
            let blank_lines = blank_lines_before_offset(markdown, range.start);
            if blank_lines > 0 {
                out.push(Event::InterBlockBlankLines(blank_lines));
            }
        }

        if range.start > prev_end {
            emit_reference_definition_gap_events(markdown, prev_end, range.start, &mut out);
        }
        match event {
            MdEvent::Start(tag) => {
                match tag {
                    Tag::Link { dest_url, .. } => {
                        // Start of a link - store the URL and track state
                        link_state = Some((true, false, dest_url.to_string(), range.start));
                        link_text.clear();
                    }
                    Tag::Image { dest_url, .. } => {
                        // Start of an image - store the URL and track state
                        link_state = Some((false, true, dest_url.to_string(), range.start));
                        link_text.clear();
                    }
                    Tag::Heading { level, .. } => {
                        let is_setext = heading_is_setext_at_offset(
                            markdown,
                            range.start,
                            matches!(level, pulldown_cmark::HeadingLevel::H1),
                        );
                        out.push(Event::HeadingSetext(is_setext));
                        out.push(Event::Start(convert_tag(tag)));
                    }
                    Tag::Strong => {
                        out.push(Event::StrongStart);
                    }
                    Tag::Emphasis => {
                        out.push(Event::EmphasisStart);
                    }
                    Tag::Strikethrough => {
                        out.push(Event::StrikethroughStart);
                    }
                    Tag::List(start) => {
                        list_kind_stack.push(start.is_some());
                        out.push(Event::Start(Block::List { start }));
                    }
                    Tag::Item => {
                        if list_item_has_blank_line_before_at_offset(markdown, range.start) {
                            out.push(Event::ListItemPrecededByBlankLine);
                        }
                        match list_kind_stack.last().copied() {
                            Some(false) => {
                                if let Some(marker) =
                                    unordered_list_marker_at_offset(markdown, range.start)
                                {
                                    out.push(Event::ListItemMarker(marker));
                                }
                            }
                            Some(true) => {
                                if let Some(number) =
                                    ordered_list_number_at_offset(markdown, range.start)
                                {
                                    out.push(Event::OrderedListItemMarker(number));
                                }
                            }
                            _ => {}
                        }
                        out.push(Event::Start(Block::ListItem));
                    }
                    _ => {
                        out.push(Event::Start(convert_tag(tag)));
                    }
                }
            }
            MdEvent::End(tag) => {
                match tag {
                    TagEnd::Link => {
                        // End of a link - emit the Link event with text and URL
                        if let Some((true, false, url, start_offset)) = link_state.take() {
                            let mut source =
                                extract_source_from_range(markdown, start_offset, range.end);
                            if is_escaped_image_like_link(markdown, start_offset)
                                && source.starts_with('[')
                            {
                                source.insert(0, '!');
                            }
                            out.push(Event::Link {
                                text: link_text.clone(),
                                url,
                                source,
                            });
                        } else {
                            // Fallback (shouldn't happen in well-formed markdown)
                            if !link_text.is_empty() {
                                out.push(Event::Text(link_text.clone()));
                            }
                        }
                        link_text.clear();
                    }
                    TagEnd::Image => {
                        // End of an image - emit the Image event with alt and URL
                        if let Some((false, true, url, start_offset)) = link_state.take() {
                            let source =
                                extract_source_from_range(markdown, start_offset, range.end);
                            out.push(Event::Image {
                                alt: link_text.clone(),
                                url,
                                source,
                            });
                        } else {
                            // Fallback
                            if !link_text.is_empty() {
                                out.push(Event::Text(link_text.clone()));
                            }
                        }
                        link_text.clear();
                    }
                    TagEnd::Strong => {
                        out.push(Event::StrongEnd);
                    }
                    TagEnd::Emphasis => {
                        out.push(Event::EmphasisEnd);
                    }
                    TagEnd::Strikethrough => {
                        out.push(Event::StrikethroughEnd);
                    }
                    TagEnd::List(_) => {
                        let _ = list_kind_stack.pop();
                        out.push(Event::End(convert_tag_end(tag)));
                    }
                    _ => {
                        out.push(Event::End(convert_tag_end(tag)));
                    }
                }
            }
            MdEvent::Text(text) => {
                if link_state.is_some() {
                    // Accumulate text for link/image
                    link_text.push_str(&text);
                } else {
                    out.push(Event::Text(text.to_string()));
                }
            }
            MdEvent::SoftBreak => out.push(Event::SoftBreak),
            MdEvent::HardBreak => out.push(Event::HardBreak),
            MdEvent::Rule => {
                let marker = thematic_break_marker_at_offset(markdown, range.start).unwrap_or('-');
                out.push(Event::Rule(marker));
            }
            MdEvent::Code(code) => {
                if link_state.is_some() {
                    // Inline code inside link/image labels belongs to label text.
                    link_text.push_str(&code);
                } else {
                    out.push(Event::InlineCode(code.to_string()));
                }
            }
            MdEvent::InlineMath(_) => out.push(Event::Text(String::new())),
            MdEvent::DisplayMath(_) => out.push(Event::Text(String::new())),
            MdEvent::Html(html) => out.push(Event::RawHtml(html.to_string())),
            MdEvent::InlineHtml(html) => out.push(Event::RawHtml(html.to_string())),
            MdEvent::FootnoteReference(label) => {
                out.push(Event::FootnoteReference(label.to_string()))
            }
            MdEvent::TaskListMarker(checked) => out.push(Event::TaskListMarker(checked)),
        }
        prev_end = range.end;
    }

    if prev_end < markdown.len() {
        emit_reference_definition_gap_events(markdown, prev_end, markdown.len(), &mut out);
    }

    out.into_iter()
}

fn line_at_offset(markdown: &str, mut offset: usize) -> &str {
    if markdown.is_empty() {
        return "";
    }
    if offset >= markdown.len() {
        offset = markdown.len() - 1;
    }
    let start = markdown[..offset].rfind('\n').map_or(0, |i| i + 1);
    let end = markdown[offset..]
        .find('\n')
        .map_or(markdown.len(), |i| offset + i);
    &markdown[start..end]
}

fn thematic_break_marker_at_offset(markdown: &str, offset: usize) -> Option<char> {
    thematic_break_marker_for_line(line_at_offset(markdown, offset))
}

fn unordered_list_marker_at_offset(markdown: &str, offset: usize) -> Option<char> {
    unordered_list_marker_for_line(line_at_offset(markdown, offset))
}

fn ordered_list_number_at_offset(markdown: &str, offset: usize) -> Option<u64> {
    ordered_list_number_for_line(line_at_offset(markdown, offset))
}

fn list_item_has_blank_line_before_at_offset(markdown: &str, mut offset: usize) -> bool {
    if markdown.is_empty() {
        return false;
    }
    if offset >= markdown.len() {
        offset = markdown.len().saturating_sub(1);
    }

    let current_start = markdown[..offset].rfind('\n').map_or(0, |i| i + 1);
    if current_start == 0 {
        return false;
    }

    let prev_end = current_start.saturating_sub(1);
    let prev_start = markdown[..prev_end].rfind('\n').map_or(0, |i| i + 1);
    let prev_line = &markdown[prev_start..prev_end];
    prev_line.trim().is_empty()
}

fn extract_source_from_range(markdown: &str, start: usize, end: usize) -> String {
    if start >= markdown.len() || end <= start {
        return String::new();
    }
    markdown.get(start..end).unwrap_or_default().to_string()
}

fn is_escaped_image_like_link(markdown: &str, start: usize) -> bool {
    start >= 2
        && markdown.as_bytes().get(start - 1) == Some(&b'!')
        && markdown.as_bytes().get(start - 2) == Some(&b'\\')
}

fn emit_reference_definition_gap_events(
    markdown: &str,
    start: usize,
    end: usize,
    out: &mut Vec<Event>,
) {
    if start >= end || end > markdown.len() {
        return;
    }
    let gap = &markdown[start..end];
    let gap_lines: Vec<&str> = gap.lines().collect();
    let def_indexes: Vec<usize> = gap_lines
        .iter()
        .enumerate()
        .filter_map(|(idx, line)| reference_definition_line(line).then_some(idx))
        .collect();
    let defs: Vec<&str> = def_indexes
        .iter()
        .map(|idx| gap_lines[*idx].trim_end())
        .collect();
    if defs.is_empty() {
        return;
    }

    let first_def_idx = def_indexes[0];
    let leading_blank_lines = gap_lines[..first_def_idx]
        .iter()
        .rev()
        .take_while(|line| line.trim().is_empty())
        .count();
    if leading_blank_lines > 0 {
        out.push(Event::InterBlockBlankLines(leading_blank_lines));
    }

    for line in defs {
        out.push(Event::ReferenceDefinition(line.to_string()));
    }
}

fn should_emit_inter_block_blank_lines_before_event(
    event: &MdEvent<'_>,
    markdown: &str,
    offset: usize,
) -> bool {
    match event {
        MdEvent::Start(Tag::Paragraph) => paragraph_starts_at_line_indent_only(markdown, offset),
        MdEvent::Start(
            Tag::Heading { .. }
            | Tag::BlockQuote(_)
            | Tag::CodeBlock(_)
            | Tag::List(_)
            | Tag::Table(_)
            | Tag::FootnoteDefinition(_)
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition
            | Tag::HtmlBlock,
        ) => true,
        _ => false,
    }
}

fn paragraph_starts_at_line_indent_only(markdown: &str, mut offset: usize) -> bool {
    if markdown.is_empty() {
        return true;
    }
    if offset > markdown.len() {
        offset = markdown.len();
    }
    let line_start = markdown[..offset].rfind('\n').map_or(0, |i| i + 1);
    markdown[line_start..offset]
        .chars()
        .all(|ch| ch == ' ' || ch == '\t')
}

fn blank_lines_before_offset(markdown: &str, mut offset: usize) -> usize {
    if markdown.is_empty() || offset == 0 {
        return 0;
    }
    if offset > markdown.len() {
        offset = markdown.len();
    }

    let line_start = markdown[..offset].rfind('\n').map_or(0, |i| i + 1);
    if line_start == 0 {
        return 0;
    }

    let mut line_end = line_start.saturating_sub(1);
    let mut count = 0usize;
    loop {
        let prev_start = markdown[..line_end].rfind('\n').map_or(0, |i| i + 1);
        let prev_line = &markdown[prev_start..line_end];
        if prev_line.trim().is_empty() {
            count += 1;
        } else {
            break;
        }

        if prev_start == 0 {
            break;
        }
        line_end = prev_start.saturating_sub(1);
    }
    count
}

fn unordered_list_marker_for_line(line: &str) -> Option<char> {
    let mut s = line;

    loop {
        let trimmed = s.trim_start();
        if let Some(rest) = trimmed.strip_prefix('>') {
            s = rest.trim_start();
            continue;
        }
        s = trimmed;
        break;
    }

    let trimmed = s.trim_start();
    let mut chars = trimmed.chars();
    let marker = chars.next()?;
    if marker != '-' && marker != '*' && marker != '+' {
        return None;
    }

    match chars.next() {
        Some(c) if c == ' ' || c == '\t' => Some(marker),
        None => Some(marker),
        _ => None,
    }
}

fn thematic_break_marker_for_line(line: &str) -> Option<char> {
    let trimmed = line.trim_end_matches([' ', '\t', '\r']);
    let without_indent = trimmed
        .strip_prefix("   ")
        .or_else(|| trimmed.strip_prefix("  "))
        .or_else(|| trimmed.strip_prefix(' '))
        .unwrap_or(trimmed);
    let mut marker: Option<char> = None;
    let mut count = 0usize;

    for ch in without_indent.chars() {
        if ch == ' ' || ch == '\t' {
            continue;
        }
        if ch != '-' && ch != '*' && ch != '_' {
            return None;
        }
        if let Some(m) = marker {
            if m != ch {
                return None;
            }
        } else {
            marker = Some(ch);
        }
        count += 1;
    }

    match (marker, count) {
        (Some(m), c) if c >= 3 => Some(m),
        _ => None,
    }
}

fn ordered_list_number_for_line(line: &str) -> Option<u64> {
    let mut s = line;

    loop {
        let trimmed = s.trim_start();
        if let Some(rest) = trimmed.strip_prefix('>') {
            s = rest.trim_start();
            continue;
        }
        s = trimmed;
        break;
    }

    let trimmed = s.trim_start();
    let mut chars = trimmed.chars().peekable();
    let mut digits = String::new();
    while let Some(ch) = chars.peek().copied() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            let _ = chars.next();
        } else {
            break;
        }
    }
    if digits.is_empty() || digits.len() > 9 {
        return None;
    }

    let delimiter = chars.next()?;
    if delimiter != '.' && delimiter != ')' {
        return None;
    }

    match chars.next() {
        Some(c) if c == ' ' || c == '\t' => digits.parse::<u64>().ok(),
        None => digits.parse::<u64>().ok(),
        _ => None,
    }
}

fn reference_definition_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return false;
    }
    if !trimmed.starts_with('[') {
        return false;
    }
    let Some(close) = trimmed.find(']') else {
        return false;
    };
    let rest = &trimmed[close + 1..];
    if !rest.starts_with(':') {
        return false;
    }
    let after_colon = rest[1..].trim_start();
    !after_colon.is_empty()
}

fn heading_is_setext_at_offset(markdown: &str, mut offset: usize, is_h1: bool) -> bool {
    if markdown.is_empty() {
        return false;
    }
    if offset >= markdown.len() {
        offset = markdown.len().saturating_sub(1);
    }

    let current_start = markdown[..offset].rfind('\n').map_or(0, |i| i + 1);
    let current_end = markdown[offset..]
        .find('\n')
        .map_or(markdown.len(), |i| offset + i);
    if current_end >= markdown.len() {
        return false;
    }
    let next_start = current_end + 1;
    let next_end = markdown[next_start..]
        .find('\n')
        .map_or(markdown.len(), |i| next_start + i);
    let _current_line = &markdown[current_start..current_end];
    let marker_line = markdown[next_start..next_end].trim();
    if marker_line.is_empty() {
        return false;
    }

    if is_h1 {
        marker_line.chars().all(|ch| ch == '=')
    } else {
        marker_line.chars().all(|ch| ch == '-')
    }
}

fn convert_tag(tag: Tag) -> Block {
    match tag {
        Tag::Heading { level, .. } => Block::Heading {
            level: match level {
                pulldown_cmark::HeadingLevel::H1 => 1,
                pulldown_cmark::HeadingLevel::H2 => 2,
                pulldown_cmark::HeadingLevel::H3 => 3,
                pulldown_cmark::HeadingLevel::H4 => 4,
                pulldown_cmark::HeadingLevel::H5 => 5,
                pulldown_cmark::HeadingLevel::H6 => 6,
            },
            text: String::new(),
        },
        Tag::Paragraph => Block::Paragraph,
        Tag::BlockQuote(_) => Block::BlockQuote,
        Tag::CodeBlock(kind) => Block::CodeBlock {
            info: match &kind {
                pulldown_cmark::CodeBlockKind::Fenced(info) => Some(info.to_string()),
                pulldown_cmark::CodeBlockKind::Indented => None,
            },
            indented: matches!(kind, pulldown_cmark::CodeBlockKind::Indented),
        },
        Tag::List(start) => Block::List { start },
        Tag::Item => Block::ListItem,
        Tag::FootnoteDefinition(label) => Block::FootnoteDefinition {
            label: label.to_string(),
        },
        Tag::Table(alignments) => Block::Table {
            alignments: alignments.into_iter().map(convert_alignment).collect(),
        },
        Tag::TableRow => Block::TableRow,
        Tag::TableCell => Block::TableCell,
        Tag::TableHead => Block::TableHead,
        Tag::Emphasis => Block::InlineIgnored,
        Tag::Strong => Block::InlineIgnored,
        Tag::Strikethrough => Block::InlineIgnored,
        Tag::Link { .. } => Block::InlineIgnored,
        Tag::Image { .. } => Block::InlineIgnored,
        Tag::HtmlBlock => Block::HtmlBlock,
        Tag::DefinitionList => Block::DefinitionList,
        Tag::DefinitionListTitle => Block::DefinitionListTitle,
        Tag::DefinitionListDefinition => Block::DefinitionListDefinition,
        Tag::MetadataBlock(_) => Block::MetadataBlock,
    }
}

fn convert_tag_end(tag: TagEnd) -> Block {
    match tag {
        TagEnd::Heading(_) => Block::Heading {
            level: 1,
            text: String::new(),
        }, // Level doesn't matter for End
        TagEnd::Paragraph => Block::Paragraph,
        TagEnd::BlockQuote(_) => Block::BlockQuote,
        TagEnd::CodeBlock => Block::CodeBlock {
            info: None,
            indented: false,
        },
        TagEnd::List(_) => Block::List { start: None },
        TagEnd::Item => Block::ListItem,
        TagEnd::Table => Block::Table {
            alignments: Vec::new(),
        },
        TagEnd::TableRow => Block::TableRow,
        TagEnd::TableCell => Block::TableCell,
        TagEnd::TableHead => Block::TableHead,
        TagEnd::Emphasis => Block::InlineIgnored,
        TagEnd::Strong => Block::InlineIgnored,
        TagEnd::Strikethrough => Block::InlineIgnored,
        TagEnd::Link => Block::InlineIgnored,
        TagEnd::Image => Block::InlineIgnored,
        TagEnd::HtmlBlock => Block::HtmlBlock,
        TagEnd::FootnoteDefinition => Block::FootnoteDefinition {
            label: String::new(),
        },
        TagEnd::DefinitionList => Block::DefinitionList,
        TagEnd::DefinitionListTitle => Block::DefinitionListTitle,
        TagEnd::DefinitionListDefinition => Block::DefinitionListDefinition,
        TagEnd::MetadataBlock(_) => Block::MetadataBlock,
    }
}

fn convert_alignment(alignment: pulldown_cmark::Alignment) -> CellAlignment {
    match alignment {
        pulldown_cmark::Alignment::None => CellAlignment::None,
        pulldown_cmark::Alignment::Left => CellAlignment::Left,
        pulldown_cmark::Alignment::Center => CellAlignment::Center,
        pulldown_cmark::Alignment::Right => CellAlignment::Right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text() {
        let markdown = "Hello";
        let events: Vec<Event> = parse_markdown(markdown).collect();
        assert!(!events.is_empty());
    }
}
