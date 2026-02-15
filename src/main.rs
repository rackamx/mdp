use clap::{Arg, Command};
use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::size as terminal_size;
use crossterm::terminal::{Clear, ClearType};
use crossterm::ExecutableCommand;
use mdp::pager::{Pager, PagerConfig};
use mdp::parsing::parse_markdown;
use mdp::rendering::Renderer;
use mdp::terminal::Terminal;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::Path;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static SIGINT_RECEIVED: AtomicBool = AtomicBool::new(false);

#[cfg(unix)]
const SIGINT: i32 = 2;

#[cfg(unix)]
type SigHandler = extern "C" fn(i32);

#[cfg(unix)]
unsafe extern "C" {
    fn signal(signum: i32, handler: SigHandler) -> SigHandler;
}

#[cfg(unix)]
extern "C" fn handle_sigint(_signum: i32) {
    SIGINT_RECEIVED.store(true, Ordering::SeqCst);
}

fn main() {
    process::exit(run());
}

fn run() -> i32 {
    let matches = Command::new("mdp")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("file")
                .help("Path to the markdown file to display ('-' for stdin)")
                .required(false),
        )
        .arg(
            Arg::new("width")
                .long("width")
                .value_name("COLUMNS")
                .value_parser(clap::value_parser!(usize))
                .help("Override render width"),
        )
        .arg(
            Arg::new("strikethrough-fallback")
                .long("strikethrough-fallback")
                .help("Render strikethrough using Unicode combining overlay instead of ANSI")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("ansi-strikethrough")
                .long("ansi-strikethrough")
                .help("Render strikethrough using ANSI escape codes instead of Unicode overlay")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();
    let width_override = matches.get_one::<usize>("width").copied();
    let strikethrough_fallback = !matches.get_flag("ansi-strikethrough");

    let (markdown, source_label, reload_path) = match matches.get_one::<String>("file") {
        Some(file_path) if file_path == "-" => match read_stdin() {
            Ok(contents) => (contents, source_label_for_arg(Some("-")), None),
            Err(e) => {
                eprintln!("Error reading stdin: {e}");
                return 2;
            }
        },
        Some(file_path) => match read_file(file_path) {
            Ok(contents) => (
                contents,
                source_label_for_arg(Some(file_path)),
                Some(file_path.to_string()),
            ),
            Err(e) => {
                eprintln!("Error reading file: {e}");
                return 1;
            }
        },
        None => {
            if io::stdin().is_terminal() {
                println!("Usage: mdp <file>");
                return 0;
            }
            match read_stdin() {
                Ok(contents) => (contents, source_label_for_arg(None), None),
                Err(e) => {
                    eprintln!("Error reading stdin: {e}");
                    return 2;
                }
            }
        }
    };

    if !io::stdout().is_terminal() || !io::stdin().is_terminal() {
        let rendered = render_markdown(
            &markdown,
            width_override.unwrap_or_else(detect_default_width),
            strikethrough_fallback,
        );
        println!("{rendered}");
        return 0;
    }

    match run_interactive_pager(
        markdown,
        width_override,
        strikethrough_fallback,
        &source_label,
        reload_path.as_deref(),
    ) {
        Ok(()) => 0,
        Err(e) if e.kind() == io::ErrorKind::Interrupted => interrupted_exit_code(),
        Err(e) => {
            eprintln!("Terminal error: {e}");
            2
        }
    }
}

fn render_markdown(markdown: &str, width: usize, strikethrough_fallback: bool) -> String {
    let events: Vec<_> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(width);
    renderer.set_strikethrough_fallback(strikethrough_fallback);
    renderer.render(&events)
}

fn run_interactive_pager(
    mut markdown: String,
    width_override: Option<usize>,
    strikethrough_fallback: bool,
    source_label: &str,
    reload_path: Option<&str>,
) -> Result<(), io::Error> {
    install_sigint_handler();
    let terminal = Terminal::new()?;
    let size = terminal.size()?;
    let mut pager = build_pager(
        &markdown,
        usize::from(size.rows),
        usize::from(size.cols),
        0,
        width_override,
        strikethrough_fallback,
    );
    let mut status_message: Option<String> = None;
    draw_page(&pager, source_label, status_message.as_deref())?;

    loop {
        if SIGINT_RECEIVED.load(Ordering::SeqCst) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "SIGINT"));
        }

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        let event = event::read()?;
        let key_event = match event {
            CEvent::Resize(new_cols, new_rows) => {
                let old_scroll = pager.scroll_position();
                pager = build_pager(
                    &markdown,
                    usize::from(new_rows),
                    usize::from(new_cols),
                    old_scroll,
                    width_override,
                    strikethrough_fallback,
                );
                draw_page(&pager, source_label, status_message.as_deref())?;
                continue;
            }
            CEvent::Key(key_event) => key_event,
            _ => continue,
        };

        if !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            continue;
        }

        if key_event.code == KeyCode::Char('c')
            && key_event.modifiers.contains(KeyModifiers::CONTROL)
        {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "SIGINT"));
        }

        status_message = None;
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => break,
            KeyCode::Char('j') | KeyCode::Down | KeyCode::Enter => pager.scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => pager.scroll_up(),
            KeyCode::Char(' ') | KeyCode::Char('f') | KeyCode::PageDown => pager.page_down(),
            KeyCode::Char('b') | KeyCode::PageUp => pager.page_up(),
            KeyCode::Char('g') | KeyCode::Home => pager.go_to_beginning(),
            KeyCode::Char('G') | KeyCode::End => pager.go_to_end(),
            KeyCode::Char('n') => pager.search_next(),
            KeyCode::Char('N') => pager.search_previous(),
            KeyCode::Char('h') | KeyCode::Char('?') => draw_help(&pager.help_text())?,
            KeyCode::Char('r') | KeyCode::Char('R') => match reload_markdown(reload_path) {
                Ok(Some(reloaded)) => {
                    markdown = reloaded;
                    let old_scroll = pager.scroll_position();
                    let size = terminal.size()?;
                    pager = build_pager(
                        &markdown,
                        usize::from(size.rows),
                        usize::from(size.cols),
                        old_scroll,
                        width_override,
                        strikethrough_fallback,
                    );
                    status_message = Some("Reloaded".to_string());
                }
                Ok(None) => {
                    status_message = Some("Reload unavailable for stdin".to_string());
                }
                Err(e) => {
                    status_message = Some(format!("Reload failed: {e}"));
                }
            },
            _ => {}
        }
        draw_page(&pager, source_label, status_message.as_deref())?;
    }

    let mut stdout = io::stdout();
    stdout.execute(MoveTo(0, 0))?;
    stdout.execute(Clear(ClearType::All))?;
    stdout.flush()?;
    Ok(())
}

fn build_pager(
    markdown: &str,
    rows: usize,
    cols: usize,
    scroll_position: usize,
    width_override: Option<usize>,
    strikethrough_fallback: bool,
) -> Pager {
    let page_size = rows.saturating_sub(1).max(1);
    let render_width = width_override.unwrap_or_else(|| default_width_for_cols(cols));
    let rendered = render_markdown(markdown, render_width, strikethrough_fallback);

    let lines: Vec<String> = if rendered.is_empty() {
        vec![String::new()]
    } else {
        rendered.lines().map(|line| line.to_string()).collect()
    };

    let mut pager = Pager::new(PagerConfig { page_size, cols }, lines);
    pager.go_to_line(scroll_position);
    pager
}

fn default_width_for_cols(cols: usize) -> usize {
    cols.saturating_sub(4).max(40)
}

fn detect_default_width() -> usize {
    match terminal_size() {
        Ok((cols, _rows)) => default_width_for_cols(usize::from(cols)),
        Err(_) => 80,
    }
}

fn interrupted_exit_code() -> i32 {
    130
}

fn install_sigint_handler() {
    SIGINT_RECEIVED.store(false, Ordering::SeqCst);
    #[cfg(unix)]
    // SAFETY: installing a process signal handler for SIGINT with a C-compatible function.
    unsafe {
        let _ = signal(SIGINT, handle_sigint);
    }
}

fn source_label_for_arg(file_arg: Option<&str>) -> String {
    match file_arg {
        None | Some("-") => "(stdin)".to_string(),
        Some(path) => Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
            .unwrap_or_else(|| path.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        default_width_for_cols, interrupted_exit_code, looks_binary, reload_markdown,
        source_label_for_arg, SIGINT_RECEIVED,
    };
    use std::fs;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_default_width_detection() {
        assert_eq!(default_width_for_cols(100), 96);
    }

    #[test]
    fn test_minimum_width() {
        assert_eq!(default_width_for_cols(43), 40);
        assert_eq!(default_width_for_cols(10), 40);
    }

    #[test]
    fn test_display_filename() {
        assert_eq!(source_label_for_arg(Some("/tmp/example.md")), "example.md");
    }

    #[test]
    fn test_stdin_label() {
        assert_eq!(source_label_for_arg(None), "(stdin)");
        assert_eq!(source_label_for_arg(Some("-")), "(stdin)");
    }

    #[test]
    fn test_sigint_exit_code() {
        assert_eq!(interrupted_exit_code(), 130);
    }

    #[test]
    fn test_binary_heuristic() {
        assert!(looks_binary(&[0, 1, 2, 3]));
        assert!(!looks_binary(b"plain text\nwith lines"));
    }

    #[test]
    fn test_reload_markdown_from_file() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("mdp_reload_test.md");
        fs::write(&path, "# reloaded\n").expect("write test file");
        let path_str = path.to_string_lossy().into_owned();

        let reloaded = reload_markdown(Some(&path_str))
            .expect("reload should succeed")
            .expect("file reload should return Some");
        assert_eq!(reloaded, "# reloaded\n");

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_reload_markdown_unavailable_for_stdin() {
        let reloaded = reload_markdown(None).expect("reload should not fail");
        assert!(reloaded.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn test_sigint_handler_sets_flag() {
        SIGINT_RECEIVED.store(false, Ordering::SeqCst);
        super::handle_sigint(super::SIGINT);
        assert!(SIGINT_RECEIVED.load(Ordering::SeqCst));
    }
}

fn draw_page(
    pager: &Pager,
    source_label: &str,
    status_message: Option<&str>,
) -> Result<(), io::Error> {
    let mut stdout = io::stdout();
    stdout.execute(MoveTo(0, 0))?;
    stdout.execute(Clear(ClearType::All))?;

    for line in pager.visible_lines_with_highlight() {
        write!(stdout, "{line}\r\n")?;
    }

    if let Some(indicator) = pager.progress_indicator() {
        write!(stdout, "[{source_label}] {indicator}")?;
    } else {
        write!(stdout, "[{source_label}]")?;
    }
    if let Some(message) = status_message {
        write!(stdout, " | {message}")?;
    }
    stdout.flush()?;
    Ok(())
}

fn draw_help(help_lines: &[String]) -> Result<(), io::Error> {
    let mut stdout = io::stdout();
    stdout.execute(MoveTo(0, 0))?;
    stdout.execute(Clear(ClearType::All))?;
    for line in help_lines {
        write!(stdout, "{line}\r\n")?;
    }
    write!(stdout, "\r\n")?;
    write!(stdout, "Press any key to return\r\n")?;
    stdout.flush()?;
    let _ = event::read()?;
    Ok(())
}

fn read_file(path: &str) -> Result<String, io::Error> {
    let bytes = fs::read(path)?;
    if looks_binary(&bytes) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Binary file detected",
        ));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn read_stdin() -> Result<String, io::Error> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}

fn reload_markdown(reload_path: Option<&str>) -> Result<Option<String>, io::Error> {
    match reload_path {
        Some(path) => read_file(path).map(Some),
        None => Ok(None),
    }
}

fn looks_binary(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    if bytes.contains(&0) {
        return true;
    }

    let suspicious = bytes
        .iter()
        .filter(|&&b| (b < 0x09) || (b > 0x0D && b < 0x20))
        .count();
    suspicious * 100 / bytes.len() > 10
}
