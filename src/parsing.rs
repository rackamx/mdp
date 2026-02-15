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
    Link { text: String, url: String },
    /// Image with alt text and URL (skip rendering, just show alt text)
    Image { alt: String, url: String },
    /// Task list checkbox marker (`- [ ]` or `- [x]`)
    TaskListMarker(bool),
    /// Footnote reference (e.g. [^1])
    FootnoteReference(String),
    /// Unordered list item marker from source ('-', '*', '+')
    ListItemMarker(char),
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
    CodeBlock { info: Option<String> },
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
    let mut link_state: Option<(bool, bool, String)> = None;
    let mut link_text = String::new();
    let rule_markers = detect_thematic_break_markers(markdown);
    let mut rule_index = 0usize;
    let unordered_item_markers = detect_unordered_item_markers(markdown);
    let mut unordered_item_index = 0usize;
    let mut list_kind_stack: Vec<bool> = Vec::new(); // true = ordered, false = unordered

    parser.flat_map(move |event| {
        let mut result = Vec::new();

        match event {
            MdEvent::Start(tag) => {
                match tag {
                    Tag::Link { dest_url, .. } => {
                        // Start of a link - store the URL and track state
                        link_state = Some((true, false, dest_url.to_string()));
                        link_text.clear();
                    }
                    Tag::Image { dest_url, .. } => {
                        // Start of an image - store the URL and track state
                        link_state = Some((false, true, dest_url.to_string()));
                        link_text.clear();
                    }
                    Tag::Strong => {
                        result.push(Event::StrongStart);
                    }
                    Tag::Emphasis => {
                        result.push(Event::EmphasisStart);
                    }
                    Tag::Strikethrough => {
                        result.push(Event::StrikethroughStart);
                    }
                    Tag::List(start) => {
                        list_kind_stack.push(start.is_some());
                        result.push(Event::Start(Block::List { start }));
                    }
                    Tag::Item => {
                        if list_kind_stack.last().copied() == Some(false) {
                            if let Some(marker) =
                                unordered_item_markers.get(unordered_item_index).copied()
                            {
                                unordered_item_index += 1;
                                result.push(Event::ListItemMarker(marker));
                            }
                        }
                        result.push(Event::Start(Block::ListItem));
                    }
                    _ => {
                        result.push(Event::Start(convert_tag(tag)));
                    }
                }
            }
            MdEvent::End(tag) => {
                match tag {
                    TagEnd::Link => {
                        // End of a link - emit the Link event with text and URL
                        if let Some((true, false, url)) = link_state.take() {
                            result.push(Event::Link {
                                text: link_text.clone(),
                                url,
                            });
                        } else {
                            // Fallback (shouldn't happen in well-formed markdown)
                            if !link_text.is_empty() {
                                result.push(Event::Text(link_text.clone()));
                            }
                        }
                        link_text.clear();
                    }
                    TagEnd::Image => {
                        // End of an image - emit the Image event with alt and URL
                        if let Some((false, true, url)) = link_state.take() {
                            result.push(Event::Image {
                                alt: link_text.clone(),
                                url,
                            });
                        } else {
                            // Fallback
                            if !link_text.is_empty() {
                                result.push(Event::Text(link_text.clone()));
                            }
                        }
                        link_text.clear();
                    }
                    TagEnd::Strong => {
                        result.push(Event::StrongEnd);
                    }
                    TagEnd::Emphasis => {
                        result.push(Event::EmphasisEnd);
                    }
                    TagEnd::Strikethrough => {
                        result.push(Event::StrikethroughEnd);
                    }
                    TagEnd::List(_) => {
                        let _ = list_kind_stack.pop();
                        result.push(Event::End(convert_tag_end(tag)));
                    }
                    _ => {
                        result.push(Event::End(convert_tag_end(tag)));
                    }
                }
            }
            MdEvent::Text(text) => {
                if link_state.is_some() {
                    // Accumulate text for link/image
                    link_text.push_str(&text);
                } else {
                    result.push(Event::Text(text.to_string()));
                }
            }
            MdEvent::SoftBreak => result.push(Event::SoftBreak),
            MdEvent::HardBreak => result.push(Event::HardBreak),
            MdEvent::Rule => {
                let marker = rule_markers.get(rule_index).copied().unwrap_or('-');
                rule_index += 1;
                result.push(Event::Rule(marker));
            }
            MdEvent::Code(code) => result.push(Event::InlineCode(code.to_string())),
            MdEvent::InlineMath(_) => result.push(Event::Text(String::new())),
            MdEvent::DisplayMath(_) => result.push(Event::Text(String::new())),
            MdEvent::Html(html) => result.push(Event::RawHtml(html.to_string())),
            MdEvent::InlineHtml(html) => result.push(Event::RawHtml(html.to_string())),
            MdEvent::FootnoteReference(label) => {
                result.push(Event::FootnoteReference(label.to_string()))
            }
            MdEvent::TaskListMarker(checked) => result.push(Event::TaskListMarker(checked)),
        }

        // Return iterator
        result.into_iter()
    })
}

fn detect_thematic_break_markers(markdown: &str) -> Vec<char> {
    markdown
        .lines()
        .filter_map(thematic_break_marker_for_line)
        .collect()
}

fn detect_unordered_item_markers(markdown: &str) -> Vec<char> {
    markdown
        .lines()
        .filter_map(unordered_list_marker_for_line)
        .collect()
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
            info: match kind {
                pulldown_cmark::CodeBlockKind::Fenced(info) => Some(info.to_string()),
                pulldown_cmark::CodeBlockKind::Indented => None,
            },
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
        Tag::Emphasis => Block::Paragraph, // Inline - simplified
        Tag::Strong => Block::Paragraph,   // Inline - simplified
        Tag::Strikethrough => Block::Paragraph, // Inline - simplified
        Tag::Link { .. } => Block::Paragraph, // Inline - simplified
        Tag::Image { .. } => Block::Paragraph, // Inline - simplified
        Tag::HtmlBlock => Block::Paragraph, // Simplified
        Tag::DefinitionList => Block::DefinitionList,
        Tag::DefinitionListTitle => Block::DefinitionListTitle,
        Tag::DefinitionListDefinition => Block::DefinitionListDefinition,
        Tag::MetadataBlock(_) => Block::Paragraph, // Simplified
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
        TagEnd::CodeBlock => Block::CodeBlock { info: None },
        TagEnd::List(_) => Block::List { start: None },
        TagEnd::Item => Block::ListItem,
        TagEnd::Table => Block::Table {
            alignments: Vec::new(),
        },
        TagEnd::TableRow => Block::TableRow,
        TagEnd::TableCell => Block::TableCell,
        TagEnd::TableHead => Block::TableHead,
        TagEnd::Emphasis => Block::Paragraph,
        TagEnd::Strong => Block::Paragraph,
        TagEnd::Strikethrough => Block::Paragraph,
        TagEnd::Link => Block::Paragraph,
        TagEnd::Image => Block::Paragraph,
        TagEnd::HtmlBlock => Block::Paragraph,
        TagEnd::FootnoteDefinition => Block::FootnoteDefinition {
            label: String::new(),
        },
        TagEnd::DefinitionList => Block::DefinitionList,
        TagEnd::DefinitionListTitle => Block::DefinitionListTitle,
        TagEnd::DefinitionListDefinition => Block::DefinitionListDefinition,
        TagEnd::MetadataBlock(_) => Block::Paragraph,
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
