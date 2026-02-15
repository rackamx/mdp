use mdp::parsing::parse_markdown;
use mdp::rendering::Renderer;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct SpecExample {
    markdown: String,
    html: String,
    example: u32,
    section: String,
}

#[derive(Debug)]
struct Failure {
    example: u32,
    section: String,
    reason: String,
}

#[test]
fn spec_json_is_readable() {
    let examples = load_examples();
    assert!(
        !examples.is_empty(),
        "Expected CommonMark spec examples in spec.json"
    );
}

#[test]
fn commonmark_spec_smoke_subset() {
    let examples = load_examples();
    for ex in examples.iter().take(200) {
        let events: Vec<_> = parse_markdown(&ex.markdown).collect();
        let mut renderer = Renderer::new(80);
        let rendered = renderer.render(&events);
        if ex.markdown.trim().is_empty() {
            assert!(
                strip_ansi(&rendered).trim().is_empty(),
                "Example {} [{}] expected empty output for empty input",
                ex.example,
                ex.section
            );
        } else {
            assert!(
                !rendered.is_empty(),
                "Example {} [{}] rendered to empty output unexpectedly",
                ex.example,
                ex.section
            );
        }
    }
}

#[test]
#[ignore = "Runs the full CommonMark corpus and reports all failing examples"]
fn commonmark_spec_full_audit() {
    let examples = load_examples();
    let mut failures = Vec::new();

    for ex in &examples {
        run_example_checks(ex, &mut failures);
    }

    if !failures.is_empty() {
        panic!("{}", format_failure_report(&failures, 200));
    }
}

fn load_examples() -> Vec<SpecExample> {
    let raw = fs::read_to_string("spec.json").expect("Failed to read spec.json");
    serde_json::from_str(&raw).expect("Failed to parse spec.json")
}

fn run_example_checks(ex: &SpecExample, failures: &mut Vec<Failure>) {
    let events: Vec<_> = parse_markdown(&ex.markdown).collect();
    let mut renderer = Renderer::new(80);
    let rendered = renderer.render(&events);
    let plain = strip_ansi(&rendered);

    let actual_norm = normalize_for_compare(&plain);
    let expected_text = normalize_for_compare(&html_to_text(&ex.html));

    if ex.markdown.trim().is_empty() && !actual_norm.is_empty() {
        failures.push(Failure {
            example: ex.example,
            section: ex.section.clone(),
            reason: "Expected empty output for empty markdown".to_string(),
        });
        return;
    }

    let skip_text_compare = matches!(
        ex.section.as_str(),
        "HTML blocks" | "Raw HTML" | "Links" | "Link reference definitions"
    );
    if !skip_text_compare
        && !expected_text.is_empty()
        && !is_token_subsequence(&expected_text, &actual_norm)
    {
        failures.push(Failure {
            example: ex.example,
            section: ex.section.clone(),
            reason: format!(
                "Rendered text does not preserve expected textual content. expected={:?} actual={:?}",
                expected_text, actual_norm
            ),
        });
    }

    if ex.html.contains("<blockquote>") && !plain.contains('│') {
        failures.push(Failure {
            example: ex.example,
            section: ex.section.clone(),
            reason: "Expected block quote indicator for <blockquote>".to_string(),
        });
    }

    if ex.html.contains("<li>") && !contains_list_marker(&plain) {
        failures.push(Failure {
            example: ex.example,
            section: ex.section.clone(),
            reason: "Expected list marker for <li>".to_string(),
        });
    }

    if ex.html.contains("<pre><code>") && !plain.contains("```") {
        failures.push(Failure {
            example: ex.example,
            section: ex.section.clone(),
            reason: "Expected fenced code indicator for <pre><code>".to_string(),
        });
    }
}

fn contains_list_marker(plain: &str) -> bool {
    plain.lines().map(strip_quote_prefixes).any(|line| {
        line.starts_with("* ")
            || line.starts_with("- ")
            || line.starts_with("• ")
            || ordered_marker(line)
    })
}

fn strip_quote_prefixes(line: &str) -> &str {
    let mut s = line.trim_start();
    loop {
        if let Some(rest) = s.strip_prefix('│') {
            s = rest.trim_start();
            continue;
        }
        break;
    }
    s
}

fn ordered_marker(line: &str) -> bool {
    let mut chars = line.chars().peekable();
    let mut seen_digit = false;
    while let Some(ch) = chars.peek().copied() {
        if ch.is_ascii_digit() {
            seen_digit = true;
            let _ = chars.next();
        } else {
            break;
        }
    }
    if !seen_digit {
        return false;
    }
    chars.next() == Some('.') && chars.next() == Some(' ')
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for c in chars.by_ref() {
                if ('@'..='~').contains(&c) {
                    break;
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn normalize_ws(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_for_compare(input: &str) -> String {
    let no_backticks = input.replace('`', "");
    let no_link_urls = strip_appended_link_urls(&no_backticks);
    let with_br_separation = separate_html_empty_tags(&no_link_urls);
    let normalized_punct = normalize_punctuation_spacing(&with_br_separation);
    normalize_ws(&normalized_punct)
}

fn separate_html_empty_tags(input: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        out.push(chars[i]);
        if chars[i] == '>'
            && i > 0
            && chars[i - 1] == '/'
            && i + 1 < chars.len()
            && !chars[i + 1].is_whitespace()
        {
            out.push(' ');
        }
        i += 1;
    }
    out
}

fn strip_appended_link_urls(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    let mut out = String::new();

    while i < chars.len() {
        if chars[i] == ' ' && i + 2 < chars.len() && chars[i + 1] == '(' {
            let mut j = i + 2;
            let mut content = String::new();
            while j < chars.len() && chars[j] != ')' {
                content.push(chars[j]);
                j += 1;
            }
            if j < chars.len() && is_likely_url(&content) {
                i = j + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }

    out
}

fn is_likely_url(s: &str) -> bool {
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("mailto:")
        || s.starts_with('/')
        || s.starts_with('#')
        || s.starts_with('<')
}

fn normalize_punctuation_spacing(input: &str) -> String {
    let mut out = input.to_string();
    for p in [".", ",", ";", ":", "!", "?", ")"] {
        out = out.replace(&format!(" {}", p), p);
    }
    for p in ["(", "[", "{"] {
        out = out.replace(&format!("{} ", p), p);
    }
    out
}

fn html_to_text(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let mut chars = html.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_tag {
            if ch == '>' {
                in_tag = false;
                out.push(' ');
            }
            continue;
        }

        if ch == '<' {
            in_tag = true;
            continue;
        }

        if ch == '&' {
            let mut entity = String::new();
            while let Some(&c) = chars.peek() {
                entity.push(c);
                let _ = chars.next();
                if c == ';' || entity.len() > 12 {
                    break;
                }
            }
            out.push_str(&decode_entity(&entity));
            continue;
        }

        out.push(ch);
    }

    out
}

fn decode_entity(entity_with_semicolon: &str) -> String {
    match entity_with_semicolon {
        "amp;" => "&".to_string(),
        "lt;" => "<".to_string(),
        "gt;" => ">".to_string(),
        "quot;" => "\"".to_string(),
        "apos;" => "'".to_string(),
        "nbsp;" => " ".to_string(),
        _ => {
            if let Some(num) = entity_with_semicolon
                .strip_prefix("#x")
                .and_then(|s| s.strip_suffix(';'))
                .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                .and_then(char::from_u32)
            {
                return num.to_string();
            }
            if let Some(num) = entity_with_semicolon
                .strip_prefix('#')
                .and_then(|s| s.strip_suffix(';'))
                .and_then(|dec| dec.parse::<u32>().ok())
                .and_then(char::from_u32)
            {
                return num.to_string();
            }
            format!("&{}", entity_with_semicolon)
        }
    }
}

fn is_token_subsequence(expected: &str, actual: &str) -> bool {
    let expected_tokens: Vec<&str> = expected.split_whitespace().collect();
    if expected_tokens.is_empty() {
        return true;
    }
    let mut idx = 0usize;
    for token in actual.split_whitespace() {
        if token == expected_tokens[idx] {
            idx += 1;
            if idx == expected_tokens.len() {
                return true;
            }
        }
    }
    false
}

fn format_failure_report(failures: &[Failure], max_lines: usize) -> String {
    let mut msg = String::new();
    msg.push_str(&format!(
        "CommonMark spec audit found {} failing checks.\n",
        failures.len()
    ));
    msg.push_str("First failures:\n");

    for failure in failures.iter().take(max_lines) {
        msg.push_str(&format!(
            "- Example {} [{}]: {}\n",
            failure.example, failure.section, failure.reason
        ));
    }
    msg
}
