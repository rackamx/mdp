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
    Parser::new(markdown).map(|event| convert_event(event))
}

fn convert_event(event: MdEvent) -> Event {
    match event {
        MdEvent::Start(tag) => Event::Start(convert_tag(tag)),
        MdEvent::End(tag) => Event::End(convert_tag_end(tag)),
        MdEvent::Text(text) => Event::Text(text.to_string()),
        MdEvent::SoftBreak => Event::SoftBreak,
        MdEvent::HardBreak => Event::HardBreak,
        MdEvent::Rule => Event::Rule,
        MdEvent::Code(_) => Event::Text(String::new()), // Simplified: ignore code
        MdEvent::InlineMath(_) => Event::Text(String::new()), // Simplified
        MdEvent::DisplayMath(_) => Event::Text(String::new()), // Simplified
        MdEvent::Html(_) => Event::Text(String::new()), // Simplified: ignore HTML
        MdEvent::InlineHtml(_) => Event::Text(String::new()),
        MdEvent::FootnoteReference(_) => Event::Text(String::new()), // Simplified
        MdEvent::TaskListMarker(_) => Event::Text(String::new()), // Simplified
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
