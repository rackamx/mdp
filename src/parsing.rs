use pulldown_cmark::{Event as MdEvent, Parser, Tag, TagEnd};

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
    /// Horizontal rule
    Rule,
    /// Start of strong emphasis (bold)
    StrongStart,
    /// End of strong emphasis (bold)
    StrongEnd,
    /// Start of emphasis (italics)
    EmphasisStart,
    /// End of emphasis (italics)
    EmphasisEnd,
    /// Inline code (backtick-enclosed)
    InlineCode(String),
    /// Link with text and URL
    Link { text: String, url: String },
    /// Image with alt text and URL (skip rendering, just show alt text)
    Image { alt: String, url: String },
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
}

/// Parse markdown text and return an iterator of events
pub fn parse_markdown(markdown: &str) -> impl Iterator<Item = Event> + '_ {
    let parser = Parser::new(markdown);
    // Track link/image state: (is_link, is_image, url)
    let mut link_state: Option<(bool, bool, String)> = None;
    let mut link_text = String::new();

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
            MdEvent::Rule => result.push(Event::Rule),
            MdEvent::Code(code) => result.push(Event::InlineCode(code.to_string())),
            MdEvent::InlineMath(_) => result.push(Event::Text(String::new())),
            MdEvent::DisplayMath(_) => result.push(Event::Text(String::new())),
            MdEvent::Html(_) => result.push(Event::Text(String::new())),
            MdEvent::InlineHtml(_) => result.push(Event::Text(String::new())),
            MdEvent::FootnoteReference(_) => result.push(Event::Text(String::new())),
            MdEvent::TaskListMarker(_) => result.push(Event::Text(String::new())),
        }

        // Return iterator
        result.into_iter()
    })
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
        Tag::FootnoteDefinition(_) => Block::Paragraph, // Simplified
        Tag::Table(_) => Block::Paragraph, // Simplified
        Tag::TableRow => Block::Paragraph, // Simplified
        Tag::TableCell => Block::Paragraph, // Simplified
        Tag::TableHead => Block::Paragraph, // Simplified
        Tag::Emphasis => Block::Paragraph, // Inline - simplified
        Tag::Strong => Block::Paragraph, // Inline - simplified
        Tag::Strikethrough => Block::Paragraph, // Inline - simplified
        Tag::Link { .. } => Block::Paragraph, // Inline - simplified
        Tag::Image { .. } => Block::Paragraph, // Inline - simplified
        Tag::HtmlBlock => Block::Paragraph, // Simplified
        Tag::DefinitionList => Block::Paragraph, // Simplified
        Tag::DefinitionListTitle => Block::Paragraph, // Simplified
        Tag::DefinitionListDefinition => Block::Paragraph, // Simplified
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
        TagEnd::Table => Block::Paragraph,
        TagEnd::TableRow => Block::Paragraph,
        TagEnd::TableCell => Block::Paragraph,
        TagEnd::TableHead => Block::Paragraph,
        TagEnd::Emphasis => Block::Paragraph,
        TagEnd::Strong => Block::Paragraph,
        TagEnd::Strikethrough => Block::Paragraph,
        TagEnd::Link => Block::Paragraph,
        TagEnd::Image => Block::Paragraph,
        TagEnd::HtmlBlock => Block::Paragraph,
        TagEnd::FootnoteDefinition => Block::Paragraph,
        TagEnd::DefinitionList => Block::Paragraph,
        TagEnd::DefinitionListTitle => Block::Paragraph,
        TagEnd::DefinitionListDefinition => Block::Paragraph,
        TagEnd::MetadataBlock(_) => Block::Paragraph,
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
