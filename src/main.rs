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
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::Path;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static SIGINT_RECEIVED: AtomicBool = AtomicBool::new(false);
const HORIZONTAL_PADDING: u16 = 2;

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

struct LoadedInput {
    markdown: String,
    source_label: String,
    reload_path: Option<String>,
}

#[derive(Debug)]
enum InputLoadError {
    Usage,
    ReadFile(io::Error),
    ReadStdin(io::Error),
}

#[cfg(test)]
fn load_input<FReadStdin, FReadFile>(
    file_arg: Option<&str>,
    stdin_is_terminal: bool,
    read_stdin: &FReadStdin,
    read_file: &FReadFile,
) -> Result<LoadedInput, InputLoadError>
where
    FReadStdin: Fn() -> Result<String, io::Error>,
    FReadFile: Fn(&str) -> Result<String, io::Error>,
{
    match file_arg {
        Some("-") => {
            let markdown = read_stdin().map_err(InputLoadError::ReadStdin)?;
            Ok(LoadedInput {
                markdown,
                source_label: source_label_for_arg(Some("-")),
                reload_path: None,
            })
        }
        Some(file_path) => {
            let markdown = read_file(file_path).map_err(InputLoadError::ReadFile)?;
            Ok(LoadedInput {
                markdown,
                source_label: source_label_for_arg(Some(file_path)),
                reload_path: Some(file_path.to_string()),
            })
        }
        None => {
            if stdin_is_terminal {
                return Err(InputLoadError::Usage);
            }
            let markdown = read_stdin().map_err(InputLoadError::ReadStdin)?;
            Ok(LoadedInput {
                markdown,
                source_label: source_label_for_arg(None),
                reload_path: None,
            })
        }
    }
}

fn main() {
    process::exit(run());
}

fn run() -> i32 {
    let args: Vec<String> = env::args().collect();
    let mut io = RealAppIo;
    let mut interactive = RealInteractiveRunner;
    run_with_env(&args, &mut io, &mut interactive)
}

fn build_cli() -> Command {
    Command::new("mdp")
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
        .arg(
            Arg::new("flatten-wide-tables")
                .long("flatten-wide-tables")
                .help("Flatten wide tables to fallback layout instead of rendering as wrapped tables")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("benchmark")
                .long("benchmark")
                .help("Run a local performance benchmark (parse/render/search) and exit")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("bench-iters")
                .long("bench-iters")
                .value_name("N")
                .value_parser(clap::value_parser!(usize))
                .default_value("50")
                .help("Number of benchmark iterations"),
        )
}

trait AppIo {
    fn stdin_is_terminal(&self) -> bool;
    fn stdout_is_terminal(&self) -> bool;
    fn read_stdin(&mut self) -> Result<String, io::Error>;
    fn read_file(&mut self, path: &str) -> Result<String, io::Error>;
    fn write_stdout(&mut self, text: &str);
    fn write_stderr(&mut self, text: &str);
}

struct RealAppIo;

impl AppIo for RealAppIo {
    fn stdin_is_terminal(&self) -> bool {
        io::stdin().is_terminal()
    }

    fn stdout_is_terminal(&self) -> bool {
        io::stdout().is_terminal()
    }

    fn read_stdin(&mut self) -> Result<String, io::Error> {
        read_stdin()
    }

    fn read_file(&mut self, path: &str) -> Result<String, io::Error> {
        read_file(path)
    }

    fn write_stdout(&mut self, text: &str) {
        print!("{text}");
    }

    fn write_stderr(&mut self, text: &str) {
        eprint!("{text}");
    }
}

trait InteractiveRunner {
    fn run_interactive(
        &mut self,
        markdown: String,
        width_override: Option<usize>,
        strikethrough_fallback: bool,
        flatten_wide_tables: bool,
        source_label: &str,
        reload_path: Option<&str>,
    ) -> Result<(), io::Error>;
}

struct RealInteractiveRunner;

impl InteractiveRunner for RealInteractiveRunner {
    fn run_interactive(
        &mut self,
        markdown: String,
        width_override: Option<usize>,
        strikethrough_fallback: bool,
        flatten_wide_tables: bool,
        source_label: &str,
        reload_path: Option<&str>,
    ) -> Result<(), io::Error> {
        run_interactive_pager(
            markdown,
            width_override,
            strikethrough_fallback,
            flatten_wide_tables,
            source_label,
            reload_path,
        )
    }
}

fn load_input_with_io(
    file_arg: Option<&str>,
    stdin_is_terminal: bool,
    io: &mut impl AppIo,
) -> Result<LoadedInput, InputLoadError> {
    match file_arg {
        Some("-") => {
            let markdown = io.read_stdin().map_err(InputLoadError::ReadStdin)?;
            Ok(LoadedInput {
                markdown,
                source_label: source_label_for_arg(Some("-")),
                reload_path: None,
            })
        }
        Some(file_path) => {
            let markdown = io.read_file(file_path).map_err(InputLoadError::ReadFile)?;
            Ok(LoadedInput {
                markdown,
                source_label: source_label_for_arg(Some(file_path)),
                reload_path: Some(file_path.to_string()),
            })
        }
        None => {
            if stdin_is_terminal {
                return Err(InputLoadError::Usage);
            }
            let markdown = io.read_stdin().map_err(InputLoadError::ReadStdin)?;
            Ok(LoadedInput {
                markdown,
                source_label: source_label_for_arg(None),
                reload_path: None,
            })
        }
    }
}

fn run_with_env(
    args: &[String],
    app_io: &mut impl AppIo,
    interactive: &mut impl InteractiveRunner,
) -> i32 {
    let matches = match build_cli().try_get_matches_from(args.iter().map(String::as_str)) {
        Ok(m) => m,
        Err(e) => {
            let msg = e.to_string();
            let code = e.exit_code();
            if code == 0 {
                app_io.write_stdout(&msg);
            } else {
                app_io.write_stderr(&msg);
            }
            return code;
        }
    };
    let width_override = matches.get_one::<usize>("width").copied();
    let strikethrough_fallback = !matches.get_flag("ansi-strikethrough");
    let flatten_wide_tables = matches.get_flag("flatten-wide-tables");
    let benchmark_mode = matches.get_flag("benchmark");
    let bench_iters = *matches
        .get_one::<usize>("bench-iters")
        .expect("bench-iters has a default value");

    let loaded = match load_input_with_io(
        matches.get_one::<String>("file").map(String::as_str),
        app_io.stdin_is_terminal(),
        app_io,
    ) {
        Ok(loaded) => loaded,
        Err(InputLoadError::Usage) => {
            app_io.write_stdout("Usage: mdp <file>\n");
            return 0;
        }
        Err(InputLoadError::ReadFile(e)) => {
            app_io.write_stderr(&format!("Error reading file: {e}\n"));
            return 1;
        }
        Err(InputLoadError::ReadStdin(e)) => {
            app_io.write_stderr(&format!("Error reading stdin: {e}\n"));
            return 2;
        }
    };
    let LoadedInput {
        markdown,
        source_label,
        reload_path,
    } = loaded;

    if benchmark_mode {
        let width = width_override.unwrap_or_else(detect_default_width);
        let report = run_benchmark(
            &markdown,
            width,
            strikethrough_fallback,
            flatten_wide_tables,
            bench_iters,
        );
        app_io.write_stdout(&format!("{report}\n"));
        return 0;
    }

    if !app_io.stdout_is_terminal() || !app_io.stdin_is_terminal() {
        let rendered = render_markdown(
            &markdown,
            width_override.unwrap_or_else(detect_default_width),
            strikethrough_fallback,
            flatten_wide_tables,
        );
        app_io.write_stdout(&format!("{rendered}\n"));
        return 0;
    }

    match interactive.run_interactive(
        markdown,
        width_override,
        strikethrough_fallback,
        flatten_wide_tables,
        &source_label,
        reload_path.as_deref(),
    ) {
        Ok(()) => 0,
        Err(e) if e.kind() == io::ErrorKind::Interrupted => interrupted_exit_code(),
        Err(e) => {
            app_io.write_stderr(&format!("Terminal error: {e}\n"));
            2
        }
    }
}

fn render_markdown(
    markdown: &str,
    width: usize,
    strikethrough_fallback: bool,
    flatten_wide_tables: bool,
) -> String {
    let events: Vec<_> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(width);
    renderer.set_strikethrough_fallback(strikethrough_fallback);
    renderer.set_flatten_wide_tables(flatten_wide_tables);
    renderer.render(&events)
}

fn render_markdown_lines(
    markdown: &str,
    width: usize,
    strikethrough_fallback: bool,
    flatten_wide_tables: bool,
) -> Vec<String> {
    let events: Vec<_> = parse_markdown(markdown).collect();
    let mut renderer = Renderer::new(width);
    renderer.set_strikethrough_fallback(strikethrough_fallback);
    renderer.set_flatten_wide_tables(flatten_wide_tables);
    renderer.render_lines(&events)
}

fn run_benchmark(
    markdown: &str,
    width: usize,
    strikethrough_fallback: bool,
    flatten_wide_tables: bool,
    iters: usize,
) -> String {
    let iterations = iters.max(1);
    let mut parse_render_total = Duration::ZERO;
    let mut search_total = Duration::ZERO;
    let mut rendered_lines = 0usize;
    let mut search_hits = 0usize;
    let mut search_attempts = 0usize;

    let queries = ["the", "list", "code", "heading", "footnote"];

    for _ in 0..iterations {
        let t0 = std::time::Instant::now();
        let events: Vec<_> = parse_markdown(markdown).collect();
        let mut renderer = Renderer::new(width);
        renderer.set_strikethrough_fallback(strikethrough_fallback);
        renderer.set_flatten_wide_tables(flatten_wide_tables);
        let lines = renderer.render_lines(&events);
        parse_render_total += t0.elapsed();
        rendered_lines = lines.len();

        let mut pager = Pager::new(
            PagerConfig {
                page_size: 24,
                cols: width,
            },
            if lines.is_empty() {
                vec![String::new()]
            } else {
                lines
            },
        );

        let t1 = std::time::Instant::now();
        for query in queries {
            search_attempts += 1;
            if pager.search(query).is_some() {
                search_hits += 1;
            }
        }
        search_total += t1.elapsed();
    }

    let parse_render_avg_ms = parse_render_total.as_secs_f64() * 1000.0 / iterations as f64;
    let search_avg_ms = search_total.as_secs_f64() * 1000.0 / iterations as f64;
    let total_avg_ms = parse_render_avg_ms + search_avg_ms;

    format!(
        "Benchmark (mdp)\nIterations: {iterations}\nInput bytes: {}\nRender width: {width}\nRendered lines: {rendered_lines}\nParse+Render avg: {:.3} ms\nSearch avg ({} queries): {:.3} ms\nTotal avg/iter: {:.3} ms\nSearch hits: {search_hits}/{search_attempts}",
        markdown.len(),
        parse_render_avg_ms,
        queries.len(),
        search_avg_ms,
        total_avg_ms
    )
}

#[derive(Default)]
struct FrameCache {
    body_lines: Vec<String>,
    footer: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PagerKeyAction {
    Quit,
    ScrollDown,
    ScrollUp,
    PageDown,
    PageUp,
    GoToBeginning,
    GoToEnd,
    SearchPrompt,
    SearchNext,
    SearchPrevious,
    ShowHelp,
    Reload,
    Noop,
}

trait PagerNavigation {
    fn scroll_down(&mut self);
    fn scroll_up(&mut self);
    fn page_down(&mut self);
    fn page_up(&mut self);
    fn go_to_beginning(&mut self);
    fn go_to_end(&mut self);
    fn search_next(&mut self);
    fn search_previous(&mut self);
}

impl PagerNavigation for Pager {
    fn scroll_down(&mut self) {
        Pager::scroll_down(self);
    }
    fn scroll_up(&mut self) {
        Pager::scroll_up(self);
    }
    fn page_down(&mut self) {
        Pager::page_down(self);
    }
    fn page_up(&mut self) {
        Pager::page_up(self);
    }
    fn go_to_beginning(&mut self) {
        Pager::go_to_beginning(self);
    }
    fn go_to_end(&mut self) {
        Pager::go_to_end(self);
    }
    fn search_next(&mut self) {
        Pager::search_next(self);
    }
    fn search_previous(&mut self) {
        Pager::search_previous(self);
    }
}

trait EventSource {
    fn poll(&mut self, timeout: Duration) -> Result<bool, io::Error>;
    fn read(&mut self) -> Result<CEvent, io::Error>;
}

struct CrosstermEventSource;

impl EventSource for CrosstermEventSource {
    fn poll(&mut self, timeout: Duration) -> Result<bool, io::Error> {
        event::poll(timeout)
    }

    fn read(&mut self) -> Result<CEvent, io::Error> {
        event::read()
    }
}

trait TerminalSizeProvider {
    fn size(&self) -> Result<mdp::terminal::Size, io::Error>;
}

impl TerminalSizeProvider for Terminal {
    fn size(&self) -> Result<mdp::terminal::Size, io::Error> {
        Terminal::size(self)
    }
}

trait PagerScreen {
    fn draw_page(
        &mut self,
        pager: &Pager,
        source_label: &str,
        status_message: Option<&str>,
        cache: &mut FrameCache,
    ) -> Result<(), io::Error>;
    fn prompt_search(
        &mut self,
        pager: &Pager,
        source_label: &str,
    ) -> Result<Option<String>, io::Error>;
    fn draw_help(&mut self, help_lines: &[String]) -> Result<(), io::Error>;
    fn clear(&mut self) -> Result<(), io::Error>;
}

struct CrosstermScreen;

impl PagerScreen for CrosstermScreen {
    fn draw_page(
        &mut self,
        pager: &Pager,
        source_label: &str,
        status_message: Option<&str>,
        cache: &mut FrameCache,
    ) -> Result<(), io::Error> {
        draw_page(pager, source_label, status_message, cache)
    }

    fn prompt_search(
        &mut self,
        pager: &Pager,
        source_label: &str,
    ) -> Result<Option<String>, io::Error> {
        prompt_search(pager, source_label)
    }

    fn draw_help(&mut self, help_lines: &[String]) -> Result<(), io::Error> {
        draw_help(help_lines)
    }

    fn clear(&mut self) -> Result<(), io::Error> {
        let mut stdout = io::stdout();
        stdout.execute(MoveTo(0, 0))?;
        stdout.execute(Clear(ClearType::All))?;
        stdout.flush()?;
        Ok(())
    }
}

fn key_event_to_action(code: KeyCode) -> PagerKeyAction {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') => PagerKeyAction::Quit,
        KeyCode::Char('j') | KeyCode::Down | KeyCode::Enter => PagerKeyAction::ScrollDown,
        KeyCode::Char('k') | KeyCode::Up => PagerKeyAction::ScrollUp,
        KeyCode::Char(' ') | KeyCode::Char('f') | KeyCode::PageDown => PagerKeyAction::PageDown,
        KeyCode::Char('b') | KeyCode::PageUp => PagerKeyAction::PageUp,
        KeyCode::Char('g') | KeyCode::Home => PagerKeyAction::GoToBeginning,
        KeyCode::Char('G') | KeyCode::End => PagerKeyAction::GoToEnd,
        KeyCode::Char('/') => PagerKeyAction::SearchPrompt,
        KeyCode::Char('n') => PagerKeyAction::SearchNext,
        KeyCode::Char('N') => PagerKeyAction::SearchPrevious,
        KeyCode::Char('h') | KeyCode::Char('?') => PagerKeyAction::ShowHelp,
        KeyCode::Char('r') | KeyCode::Char('R') => PagerKeyAction::Reload,
        _ => PagerKeyAction::Noop,
    }
}

fn apply_navigation_action(pager: &mut impl PagerNavigation, action: PagerKeyAction) -> bool {
    match action {
        PagerKeyAction::ScrollDown => pager.scroll_down(),
        PagerKeyAction::ScrollUp => pager.scroll_up(),
        PagerKeyAction::PageDown => pager.page_down(),
        PagerKeyAction::PageUp => pager.page_up(),
        PagerKeyAction::GoToBeginning => pager.go_to_beginning(),
        PagerKeyAction::GoToEnd => pager.go_to_end(),
        PagerKeyAction::SearchNext => pager.search_next(),
        PagerKeyAction::SearchPrevious => pager.search_previous(),
        _ => return false,
    }
    true
}

fn run_interactive_pager(
    mut markdown: String,
    width_override: Option<usize>,
    strikethrough_fallback: bool,
    flatten_wide_tables: bool,
    source_label: &str,
    reload_path: Option<&str>,
) -> Result<(), io::Error> {
    install_sigint_handler();
    let terminal = Terminal::new()?;
    let mut events = CrosstermEventSource;
    let mut screen = CrosstermScreen;
    run_interactive_pager_with(
        &terminal,
        &mut events,
        &mut screen,
        &mut markdown,
        width_override,
        strikethrough_fallback,
        flatten_wide_tables,
        source_label,
        reload_path,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_interactive_pager_with<T, E, S>(
    terminal: &T,
    events: &mut E,
    screen: &mut S,
    markdown: &mut String,
    width_override: Option<usize>,
    strikethrough_fallback: bool,
    flatten_wide_tables: bool,
    source_label: &str,
    reload_path: Option<&str>,
) -> Result<(), io::Error>
where
    T: TerminalSizeProvider,
    E: EventSource,
    S: PagerScreen,
{
    let size = terminal.size()?;
    let mut pager = build_pager(
        markdown,
        usize::from(size.rows),
        usize::from(size.cols),
        0,
        width_override,
        strikethrough_fallback,
        flatten_wide_tables,
    );
    let mut status_message: Option<String> = None;
    let mut frame_cache = FrameCache::default();
    screen.draw_page(
        &pager,
        source_label,
        status_message.as_deref(),
        &mut frame_cache,
    )?;

    loop {
        if SIGINT_RECEIVED.load(Ordering::SeqCst) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "SIGINT"));
        }

        if !events.poll(Duration::from_millis(100))? {
            continue;
        }
        let event = events.read()?;
        let key_event = match event {
            CEvent::Resize(new_cols, new_rows) => {
                let old_scroll = pager.scroll_position();
                pager = build_pager(
                    markdown,
                    usize::from(new_rows),
                    usize::from(new_cols),
                    old_scroll,
                    width_override,
                    strikethrough_fallback,
                    flatten_wide_tables,
                );
                frame_cache = FrameCache::default();
                screen.draw_page(
                    &pager,
                    source_label,
                    status_message.as_deref(),
                    &mut frame_cache,
                )?;
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
        let action = key_event_to_action(key_event.code);
        match action {
            PagerKeyAction::Quit => break,
            PagerKeyAction::SearchPrompt => match screen.prompt_search(&pager, source_label)? {
                Some(pattern) if pattern.is_empty() => {
                    pager.clear_search();
                    status_message = Some("Search cleared".to_string());
                }
                Some(pattern) => {
                    let _ = pager.search(&pattern);
                }
                None => {
                    status_message = Some("Search canceled".to_string());
                }
            },
            PagerKeyAction::ShowHelp => screen.draw_help(&pager.help_text())?,
            PagerKeyAction::Reload => match reload_markdown(reload_path) {
                Ok(Some(reloaded)) => {
                    *markdown = reloaded;
                    let old_scroll = pager.scroll_position();
                    let size = terminal.size()?;
                    pager = build_pager(
                        markdown,
                        usize::from(size.rows),
                        usize::from(size.cols),
                        old_scroll,
                        width_override,
                        strikethrough_fallback,
                        flatten_wide_tables,
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
            _ => {
                let _ = apply_navigation_action(&mut pager, action);
            }
        }
        screen.draw_page(
            &pager,
            source_label,
            status_message.as_deref(),
            &mut frame_cache,
        )?;
    }

    screen.clear()?;
    Ok(())
}

fn build_pager(
    markdown: &str,
    rows: usize,
    cols: usize,
    scroll_position: usize,
    width_override: Option<usize>,
    strikethrough_fallback: bool,
    flatten_wide_tables: bool,
) -> Pager {
    let page_size = rows.saturating_sub(1).max(1);
    let render_width = width_override.unwrap_or_else(|| default_width_for_cols(cols));
    let rendered_lines = render_markdown_lines(
        markdown,
        render_width,
        strikethrough_fallback,
        flatten_wide_tables,
    );
    let lines: Vec<String> = if rendered_lines.is_empty() {
        vec![String::new()]
    } else {
        rendered_lines
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
        apply_navigation_action, apply_prompt_key, build_footer_line, build_help_screen_lines,
        build_pager, default_width_for_cols, draw_page_with_writer, interrupted_exit_code,
        key_event_to_action, load_input, looks_binary, reload_markdown, render_markdown_lines,
        run_benchmark, run_with_env, source_label_for_arg, FrameCache, InputLoadError,
        PagerKeyAction, PagerNavigation, PromptAction, SIGINT_RECEIVED,
    };
    use mdp::pager::{Pager, PagerConfig};
    use std::collections::VecDeque;
    use std::fs;
    use std::io;
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

    #[test]
    fn test_footer_includes_search_status() {
        let mut pager = Pager::new(
            PagerConfig {
                page_size: 10,
                cols: 80,
            },
            vec!["alpha beta".to_string()],
        );
        let _ = pager.search("beta");

        let footer = build_footer_line(&pager, "sample.md", None);
        assert!(
            footer.contains("Search: 'beta' (1/1)"),
            "Expected search status in footer, got: {footer:?}"
        );
    }

    #[test]
    fn test_footer_includes_search_and_status_message() {
        let mut pager = Pager::new(
            PagerConfig {
                page_size: 10,
                cols: 80,
            },
            vec!["alpha beta".to_string()],
        );
        let _ = pager.search("beta");

        let footer = build_footer_line(&pager, "sample.md", Some("Reloaded"));
        assert!(
            footer.contains("Search: 'beta' (1/1) | Reloaded"),
            "Expected search and status message in footer, got: {footer:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_sigint_handler_sets_flag() {
        SIGINT_RECEIVED.store(false, Ordering::SeqCst);
        super::handle_sigint(super::SIGINT);
        assert!(SIGINT_RECEIVED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_load_input_returns_usage_when_no_file_and_tty() {
        let result = load_input(None, true, &|| Ok("stdin".to_string()), &|_| {
            Ok("".to_string())
        });
        assert!(matches!(result, Err(InputLoadError::Usage)));
    }

    #[test]
    fn test_load_input_propagates_file_read_error() {
        let result = load_input(Some("missing.md"), true, &|| Ok(String::new()), &|_| {
            Err(io::Error::new(io::ErrorKind::NotFound, "missing"))
        });
        assert!(matches!(result, Err(InputLoadError::ReadFile(_))));
    }

    #[test]
    fn test_load_input_propagates_stdin_read_error_for_dash() {
        let result = load_input(
            Some("-"),
            false,
            &|| Err(io::Error::other("stdin fail")),
            &|_| Ok(String::new()),
        );
        assert!(matches!(result, Err(InputLoadError::ReadStdin(_))));
    }

    #[test]
    fn test_load_input_propagates_stdin_read_error_without_file_arg() {
        let result = load_input(
            None,
            false,
            &|| Err(io::Error::other("stdin fail")),
            &|_| Ok(String::new()),
        );
        assert!(matches!(result, Err(InputLoadError::ReadStdin(_))));
    }

    #[test]
    fn test_benchmark_report_has_expected_fields_and_clamps_iterations() {
        let report = run_benchmark("# heading\n\n- one\n- two\n", 80, true, true, 0);
        assert!(
            report.contains("Benchmark (mdp)")
                && report.contains("Iterations: 1")
                && report.contains("Input bytes:")
                && report.contains("Parse+Render avg:")
                && report.contains("Search hits:"),
            "Unexpected benchmark report format: {report}"
        );
    }

    #[test]
    fn test_benchmark_handles_empty_rendered_output_path() {
        let report = run_benchmark("", 80, true, true, 1);
        assert!(
            report.contains("Rendered lines: 0"),
            "Expected empty markdown benchmark to report 0 rendered lines: {report}"
        );
    }

    #[test]
    fn test_key_mapping_for_navigation_actions() {
        assert_eq!(
            key_event_to_action(crossterm::event::KeyCode::Down),
            PagerKeyAction::ScrollDown
        );
        assert_eq!(
            key_event_to_action(crossterm::event::KeyCode::PageUp),
            PagerKeyAction::PageUp
        );
        assert_eq!(
            key_event_to_action(crossterm::event::KeyCode::Char('/')),
            PagerKeyAction::SearchPrompt
        );
        assert_eq!(
            key_event_to_action(crossterm::event::KeyCode::Char('r')),
            PagerKeyAction::Reload
        );
    }

    struct MockPagerNav {
        calls: Vec<&'static str>,
    }

    impl MockPagerNav {
        fn new() -> Self {
            Self { calls: Vec::new() }
        }
    }

    impl PagerNavigation for MockPagerNav {
        fn scroll_down(&mut self) {
            self.calls.push("scroll_down");
        }
        fn scroll_up(&mut self) {
            self.calls.push("scroll_up");
        }
        fn page_down(&mut self) {
            self.calls.push("page_down");
        }
        fn page_up(&mut self) {
            self.calls.push("page_up");
        }
        fn go_to_beginning(&mut self) {
            self.calls.push("go_to_beginning");
        }
        fn go_to_end(&mut self) {
            self.calls.push("go_to_end");
        }
        fn search_next(&mut self) {
            self.calls.push("search_next");
        }
        fn search_previous(&mut self) {
            self.calls.push("search_previous");
        }
    }

    #[test]
    fn test_apply_navigation_action_dispatches_to_target() {
        let mut nav = MockPagerNav::new();
        assert!(apply_navigation_action(
            &mut nav,
            PagerKeyAction::ScrollDown
        ));
        assert!(apply_navigation_action(&mut nav, PagerKeyAction::GoToEnd));
        assert!(apply_navigation_action(
            &mut nav,
            PagerKeyAction::SearchPrevious
        ));
        assert!(!apply_navigation_action(&mut nav, PagerKeyAction::ShowHelp));
        assert_eq!(
            nav.calls,
            vec!["scroll_down", "go_to_end", "search_previous"]
        );
    }

    #[test]
    fn test_load_input_file_success_sets_reload_path_and_label() {
        let result = load_input(
            Some("/tmp/path/example.md"),
            true,
            &|| Ok(String::new()),
            &|_| Ok("# doc\n".to_string()),
        )
        .expect("file load should succeed");
        assert_eq!(result.markdown, "# doc\n");
        assert_eq!(result.source_label, "example.md");
        assert_eq!(result.reload_path.as_deref(), Some("/tmp/path/example.md"));
    }

    #[test]
    fn test_load_input_stdin_success_without_file_arg() {
        let result = load_input(None, false, &|| Ok("from-stdin".to_string()), &|_| {
            Ok(String::new())
        })
        .expect("stdin load should succeed");
        assert_eq!(result.markdown, "from-stdin");
        assert_eq!(result.source_label, "(stdin)");
        assert!(result.reload_path.is_none());
    }

    #[test]
    fn test_key_mapping_for_non_navigation_keys() {
        assert_eq!(
            key_event_to_action(crossterm::event::KeyCode::Char('x')),
            PagerKeyAction::Noop
        );
        assert_eq!(
            key_event_to_action(crossterm::event::KeyCode::Char('Q')),
            PagerKeyAction::Quit
        );
        assert_eq!(
            key_event_to_action(crossterm::event::KeyCode::Char('h')),
            PagerKeyAction::ShowHelp
        );
        assert_eq!(
            key_event_to_action(crossterm::event::KeyCode::Char('k')),
            PagerKeyAction::ScrollUp
        );
        assert_eq!(
            key_event_to_action(crossterm::event::KeyCode::Char(' ')),
            PagerKeyAction::PageDown
        );
        assert_eq!(
            key_event_to_action(crossterm::event::KeyCode::Char('g')),
            PagerKeyAction::GoToBeginning
        );
        assert_eq!(
            key_event_to_action(crossterm::event::KeyCode::Char('G')),
            PagerKeyAction::GoToEnd
        );
        assert_eq!(
            key_event_to_action(crossterm::event::KeyCode::Char('n')),
            PagerKeyAction::SearchNext
        );
        assert_eq!(
            key_event_to_action(crossterm::event::KeyCode::Char('N')),
            PagerKeyAction::SearchPrevious
        );
    }

    #[test]
    fn test_apply_navigation_action_covers_all_navigation_calls() {
        let mut nav = MockPagerNav::new();
        let actions = [
            PagerKeyAction::ScrollDown,
            PagerKeyAction::ScrollUp,
            PagerKeyAction::PageDown,
            PagerKeyAction::PageUp,
            PagerKeyAction::GoToBeginning,
            PagerKeyAction::GoToEnd,
            PagerKeyAction::SearchNext,
            PagerKeyAction::SearchPrevious,
        ];
        for action in actions {
            assert!(apply_navigation_action(&mut nav, action));
        }
        assert_eq!(
            nav.calls,
            vec![
                "scroll_down",
                "scroll_up",
                "page_down",
                "page_up",
                "go_to_beginning",
                "go_to_end",
                "search_next",
                "search_previous"
            ]
        );
    }

    #[test]
    fn test_apply_navigation_action_with_real_pager_executes_impl_methods() {
        let mut pager = Pager::new(
            PagerConfig {
                page_size: 2,
                cols: 80,
            },
            vec![
                "alpha beta".to_string(),
                "beta gamma".to_string(),
                "gamma delta".to_string(),
            ],
        );
        let _ = pager.search("beta");

        assert!(apply_navigation_action(
            &mut pager,
            PagerKeyAction::ScrollDown
        ));
        assert!(apply_navigation_action(
            &mut pager,
            PagerKeyAction::ScrollUp
        ));
        assert!(apply_navigation_action(
            &mut pager,
            PagerKeyAction::PageDown
        ));
        assert!(apply_navigation_action(&mut pager, PagerKeyAction::PageUp));
        assert!(apply_navigation_action(
            &mut pager,
            PagerKeyAction::GoToBeginning
        ));
        assert!(apply_navigation_action(&mut pager, PagerKeyAction::GoToEnd));
        assert!(apply_navigation_action(
            &mut pager,
            PagerKeyAction::SearchNext
        ));
        assert!(apply_navigation_action(
            &mut pager,
            PagerKeyAction::SearchPrevious
        ));
    }

    #[test]
    fn test_reload_markdown_missing_file_propagates_error() {
        let err = reload_markdown(Some("/definitely/missing/file.md"))
            .expect_err("missing file should produce error");
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn test_looks_binary_threshold_and_edge_cases() {
        assert!(!looks_binary(&[]), "empty content should not be binary");
        assert!(
            !looks_binary(&[0x09, 0x0A, 0x0D, b'a', b'b']),
            "whitespace controls should be treated as text"
        );
        let exactly_ten_percent = [b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', 0x01];
        assert!(
            !looks_binary(&exactly_ten_percent),
            "10% suspicious bytes should not cross the binary threshold"
        );
        let above_ten_percent = [b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', 0x01, 0x02];
        assert!(
            looks_binary(&above_ten_percent),
            ">10% suspicious bytes should be binary"
        );
    }

    #[test]
    fn test_read_file_accepts_non_utf8_text_and_rejects_binary() {
        let temp_dir = std::env::temp_dir();
        let text_path = temp_dir.join("mdp_main_read_file_text.bin");
        fs::write(&text_path, b"hello \xFF world\n").expect("write text-like bytes");
        let text = super::read_file(&text_path.to_string_lossy()).expect("text-like bytes valid");
        assert!(
            text.contains("hello"),
            "expected lossy text conversion for non-utf8"
        );

        let bin_path = temp_dir.join("mdp_main_read_file_binary.bin");
        fs::write(&bin_path, [0_u8, 1, 2, 3]).expect("write binary bytes");
        let err = super::read_file(&bin_path.to_string_lossy())
            .expect_err("binary bytes should be rejected");
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);

        fs::remove_file(text_path).ok();
        fs::remove_file(bin_path).ok();
    }

    #[test]
    fn test_render_markdown_lines_returns_empty_for_empty_input() {
        let lines = render_markdown_lines("", 80, true, true);
        assert!(lines.is_empty(), "Expected no lines for empty markdown");
    }

    #[test]
    fn test_build_pager_ensures_at_least_one_line_for_empty_input() {
        let pager = build_pager("", 10, 80, 0, None, true, true);
        assert_eq!(
            pager.total_lines(),
            1,
            "pager should contain one empty line"
        );
        assert_eq!(pager.visible_lines(), vec![String::new()]);
    }

    struct MockTerminal {
        size: mdp::terminal::Size,
    }

    impl super::TerminalSizeProvider for MockTerminal {
        fn size(&self) -> Result<mdp::terminal::Size, io::Error> {
            Ok(self.size)
        }
    }

    struct MockEventSource {
        poll_responses: VecDeque<bool>,
        events: VecDeque<crossterm::event::Event>,
        poll_error: Option<io::ErrorKind>,
        read_error: Option<io::ErrorKind>,
    }

    impl MockEventSource {
        fn with_events(events: Vec<crossterm::event::Event>) -> Self {
            Self {
                poll_responses: VecDeque::new(),
                events: events.into(),
                poll_error: None,
                read_error: None,
            }
        }
    }

    impl super::EventSource for MockEventSource {
        fn poll(&mut self, _timeout: std::time::Duration) -> Result<bool, io::Error> {
            if let Some(kind) = self.poll_error.take() {
                return Err(io::Error::new(kind, "poll error"));
            }
            if let Some(v) = self.poll_responses.pop_front() {
                return Ok(v);
            }
            Ok(!self.events.is_empty())
        }

        fn read(&mut self) -> Result<crossterm::event::Event, io::Error> {
            if let Some(kind) = self.read_error.take() {
                return Err(io::Error::new(kind, "read error"));
            }
            self.events
                .pop_front()
                .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "no events"))
        }
    }

    #[derive(Default)]
    struct MockScreen {
        draw_count: usize,
        help_calls: usize,
        cleared: bool,
        prompt_queue: VecDeque<Option<String>>,
        statuses: Vec<Option<String>>,
        draw_error: Option<io::ErrorKind>,
        help_error: Option<io::ErrorKind>,
        prompt_error: Option<io::ErrorKind>,
        clear_error: Option<io::ErrorKind>,
    }

    impl super::PagerScreen for MockScreen {
        fn draw_page(
            &mut self,
            _pager: &Pager,
            _source_label: &str,
            status_message: Option<&str>,
            _cache: &mut super::FrameCache,
        ) -> Result<(), io::Error> {
            if let Some(kind) = self.draw_error.take() {
                return Err(io::Error::new(kind, "draw error"));
            }
            self.draw_count += 1;
            self.statuses
                .push(status_message.map(std::string::ToString::to_string));
            Ok(())
        }

        fn prompt_search(
            &mut self,
            _pager: &Pager,
            _source_label: &str,
        ) -> Result<Option<String>, io::Error> {
            if let Some(kind) = self.prompt_error.take() {
                return Err(io::Error::new(kind, "prompt error"));
            }
            Ok(self.prompt_queue.pop_front().unwrap_or(None))
        }

        fn draw_help(&mut self, _help_lines: &[String]) -> Result<(), io::Error> {
            if let Some(kind) = self.help_error.take() {
                return Err(io::Error::new(kind, "help error"));
            }
            self.help_calls += 1;
            Ok(())
        }

        fn clear(&mut self) -> Result<(), io::Error> {
            if let Some(kind) = self.clear_error.take() {
                return Err(io::Error::new(kind, "clear error"));
            }
            self.cleared = true;
            Ok(())
        }
    }

    fn key_press(code: crossterm::event::KeyCode) -> crossterm::event::Event {
        crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
            code,
            crossterm::event::KeyModifiers::NONE,
        ))
    }

    struct MockAppIo {
        stdin_tty: bool,
        stdout_tty: bool,
        stdin_data: Result<String, io::ErrorKind>,
        file_data: Result<String, io::ErrorKind>,
        out: String,
        err: String,
    }

    impl Default for MockAppIo {
        fn default() -> Self {
            Self {
                stdin_tty: false,
                stdout_tty: false,
                stdin_data: Ok(String::new()),
                file_data: Ok(String::new()),
                out: String::new(),
                err: String::new(),
            }
        }
    }

    impl super::AppIo for MockAppIo {
        fn stdin_is_terminal(&self) -> bool {
            self.stdin_tty
        }
        fn stdout_is_terminal(&self) -> bool {
            self.stdout_tty
        }
        fn read_stdin(&mut self) -> Result<String, io::Error> {
            self.stdin_data
                .clone()
                .map_err(|k| io::Error::new(k, "stdin read error"))
        }
        fn read_file(&mut self, _path: &str) -> Result<String, io::Error> {
            self.file_data
                .clone()
                .map_err(|k| io::Error::new(k, "file read error"))
        }
        fn write_stdout(&mut self, text: &str) {
            self.out.push_str(text);
        }
        fn write_stderr(&mut self, text: &str) {
            self.err.push_str(text);
        }
    }

    #[derive(Default)]
    struct MockInteractiveRunner {
        result: Option<Result<(), io::ErrorKind>>,
        called: usize,
    }

    impl super::InteractiveRunner for MockInteractiveRunner {
        fn run_interactive(
            &mut self,
            _markdown: String,
            _width_override: Option<usize>,
            _strikethrough_fallback: bool,
            _flatten_wide_tables: bool,
            _source_label: &str,
            _reload_path: Option<&str>,
        ) -> Result<(), io::Error> {
            self.called += 1;
            match self.result.take().unwrap_or(Ok(())) {
                Ok(()) => Ok(()),
                Err(kind) => Err(io::Error::new(kind, "interactive error")),
            }
        }
    }

    #[test]
    fn test_run_with_env_usage_path() {
        let args = vec!["mdp".to_string()];
        let mut io = MockAppIo {
            stdin_tty: true,
            stdout_tty: true,
            stdin_data: Ok(String::new()),
            file_data: Ok(String::new()),
            ..Default::default()
        };
        let mut interactive = MockInteractiveRunner::default();
        let code = run_with_env(&args, &mut io, &mut interactive);
        assert_eq!(code, 0);
        assert!(io.out.contains("Usage: mdp <file>"));
        assert_eq!(interactive.called, 0);
    }

    #[test]
    fn test_run_with_env_stdin_read_error_path() {
        let args = vec!["mdp".to_string()];
        let mut io = MockAppIo {
            stdin_tty: false,
            stdout_tty: false,
            stdin_data: Err(io::ErrorKind::BrokenPipe),
            file_data: Ok(String::new()),
            ..Default::default()
        };
        let mut interactive = MockInteractiveRunner::default();
        let code = run_with_env(&args, &mut io, &mut interactive);
        assert_eq!(code, 2);
        assert!(io.err.contains("Error reading stdin"));
    }

    #[test]
    fn test_run_with_env_clap_parse_error_writes_stderr() {
        let args = vec!["mdp".to_string(), "--definitely-invalid-flag".to_string()];
        let mut io = MockAppIo {
            stdin_tty: true,
            stdout_tty: true,
            ..Default::default()
        };
        let mut interactive = MockInteractiveRunner::default();
        let code = run_with_env(&args, &mut io, &mut interactive);
        assert_ne!(code, 0, "invalid flag should return non-zero exit");
        assert!(
            !io.err.is_empty(),
            "expected clap parse error message on stderr output sink"
        );
    }

    #[test]
    fn test_run_with_env_interactive_error_mappings() {
        let args = vec!["mdp".to_string(), "-".to_string()];
        let mut io = MockAppIo {
            stdin_tty: true,
            stdout_tty: true,
            stdin_data: Ok("doc".to_string()),
            file_data: Ok(String::new()),
            ..Default::default()
        };
        let mut interactive = MockInteractiveRunner {
            result: Some(Err(io::ErrorKind::Interrupted)),
            called: 0,
        };
        let code = run_with_env(&args, &mut io, &mut interactive);
        assert_eq!(code, 130);

        let mut io2 = MockAppIo {
            stdin_tty: true,
            stdout_tty: true,
            stdin_data: Ok("doc".to_string()),
            file_data: Ok(String::new()),
            ..Default::default()
        };
        let mut interactive2 = MockInteractiveRunner {
            result: Some(Err(io::ErrorKind::Other)),
            called: 0,
        };
        let code2 = run_with_env(&args, &mut io2, &mut interactive2);
        assert_eq!(code2, 2);
        assert!(io2.err.contains("Terminal error"));
    }

    #[derive(Default)]
    struct MockFrameWriter {
        ops: Vec<String>,
        buf: String,
    }

    impl super::FrameWriter for MockFrameWriter {
        fn move_to(&mut self, x: u16, y: u16) -> Result<(), io::Error> {
            self.ops.push(format!("move:{x}:{y}"));
            Ok(())
        }
        fn clear_current_line(&mut self) -> Result<(), io::Error> {
            self.ops.push("clear_line".to_string());
            Ok(())
        }
        fn clear_all(&mut self) -> Result<(), io::Error> {
            self.ops.push("clear_all".to_string());
            Ok(())
        }
        fn write_str(&mut self, s: &str) -> Result<(), io::Error> {
            self.buf.push_str(s);
            Ok(())
        }
        fn flush(&mut self) -> Result<(), io::Error> {
            self.ops.push("flush".to_string());
            Ok(())
        }
    }

    #[test]
    fn test_draw_page_with_writer_uses_cache_diffing() {
        let mut pager = Pager::new(
            PagerConfig {
                page_size: 2,
                cols: 80,
            },
            vec!["l1".to_string(), "l2".to_string(), "l3".to_string()],
        );
        let mut cache = FrameCache::default();
        let mut writer = MockFrameWriter::default();

        draw_page_with_writer(&pager, "doc.md", None, &mut cache, &mut writer).expect("draw ok");
        let first_ops = writer.ops.len();
        assert!(first_ops > 0);

        writer.ops.clear();
        draw_page_with_writer(&pager, "doc.md", None, &mut cache, &mut writer).expect("draw ok");
        assert_eq!(
            writer.ops,
            vec!["flush".to_string()],
            "second draw with identical frame should flush only"
        );

        pager.scroll_down();
        writer.ops.clear();
        draw_page_with_writer(&pager, "doc.md", None, &mut cache, &mut writer).expect("draw ok");
        assert!(
            writer.ops.iter().any(|op| op.starts_with("move:")),
            "scroll changed frame so move ops are expected"
        );
    }

    #[test]
    fn test_prompt_key_state_machine() {
        let mut pattern = String::new();
        assert_eq!(
            apply_prompt_key(
                &mut pattern,
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char('a'),
                    crossterm::event::KeyModifiers::NONE
                )
            ),
            PromptAction::Continue
        );
        assert_eq!(pattern, "a");
        assert_eq!(
            apply_prompt_key(
                &mut pattern,
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Backspace,
                    crossterm::event::KeyModifiers::NONE
                )
            ),
            PromptAction::Continue
        );
        assert_eq!(pattern, "");
        assert_eq!(
            apply_prompt_key(
                &mut pattern,
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Enter,
                    crossterm::event::KeyModifiers::NONE
                )
            ),
            PromptAction::Return(Some(String::new()))
        );
    }

    #[test]
    fn test_build_help_screen_lines_appends_footer() {
        let lines = build_help_screen_lines(&["A".to_string(), "B".to_string()]);
        assert_eq!(lines[0], "A");
        assert_eq!(lines[1], "B");
        assert_eq!(lines[2], "");
        assert_eq!(lines[3], "Press any key to return");
    }

    #[test]
    fn test_interactive_core_handles_resize_and_quit() {
        SIGINT_RECEIVED.store(false, Ordering::SeqCst);
        let terminal = MockTerminal {
            size: mdp::terminal::Size { rows: 8, cols: 40 },
        };
        let mut events = MockEventSource::with_events(vec![
            crossterm::event::Event::Resize(100, 20),
            key_press(crossterm::event::KeyCode::Char('q')),
        ]);
        let mut screen = MockScreen::default();
        let mut markdown = "# title\n\nbody".to_string();

        let result = super::run_interactive_pager_with(
            &terminal,
            &mut events,
            &mut screen,
            &mut markdown,
            None,
            true,
            true,
            "spec.md",
            Some("/tmp/spec.md"),
        );

        assert!(result.is_ok(), "interactive loop should exit cleanly");
        assert!(
            screen.draw_count >= 2,
            "expected at least initial + post-resize draw"
        );
        assert!(screen.cleared, "expected screen clear on normal exit");
    }

    #[test]
    fn test_interactive_core_ctrl_c_returns_interrupted() {
        SIGINT_RECEIVED.store(false, Ordering::SeqCst);
        let terminal = MockTerminal {
            size: mdp::terminal::Size { rows: 8, cols: 40 },
        };
        let mut events = MockEventSource::with_events(vec![crossterm::event::Event::Key(
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('c'),
                crossterm::event::KeyModifiers::CONTROL,
            ),
        )]);
        let mut screen = MockScreen::default();
        let mut markdown = "body".to_string();

        let err = super::run_interactive_pager_with(
            &terminal,
            &mut events,
            &mut screen,
            &mut markdown,
            None,
            true,
            true,
            "stdin",
            None,
        )
        .expect_err("ctrl-c should interrupt loop");

        assert_eq!(err.kind(), io::ErrorKind::Interrupted);
        assert!(
            !screen.cleared,
            "should not clear screen on interrupted error"
        );
    }

    #[test]
    fn test_interactive_core_show_help_and_quit() {
        SIGINT_RECEIVED.store(false, Ordering::SeqCst);
        let terminal = MockTerminal {
            size: mdp::terminal::Size { rows: 8, cols: 40 },
        };
        let mut events = MockEventSource::with_events(vec![
            key_press(crossterm::event::KeyCode::Char('h')),
            key_press(crossterm::event::KeyCode::Char('q')),
        ]);
        let mut screen = MockScreen::default();
        let mut markdown = "body".to_string();

        let result = super::run_interactive_pager_with(
            &terminal,
            &mut events,
            &mut screen,
            &mut markdown,
            None,
            true,
            true,
            "stdin",
            None,
        );

        assert!(result.is_ok());
        assert_eq!(screen.help_calls, 1);
    }

    #[test]
    fn test_interactive_core_search_prompt_statuses() {
        SIGINT_RECEIVED.store(false, Ordering::SeqCst);
        let terminal = MockTerminal {
            size: mdp::terminal::Size { rows: 8, cols: 40 },
        };
        let mut events = MockEventSource::with_events(vec![
            key_press(crossterm::event::KeyCode::Char('/')),
            key_press(crossterm::event::KeyCode::Char('/')),
            key_press(crossterm::event::KeyCode::Char('q')),
        ]);
        let mut screen = MockScreen::default();
        screen.prompt_queue.push_back(Some(String::new()));
        screen.prompt_queue.push_back(None);
        let mut markdown = "alpha beta".to_string();

        let result = super::run_interactive_pager_with(
            &terminal,
            &mut events,
            &mut screen,
            &mut markdown,
            None,
            true,
            true,
            "stdin",
            None,
        );
        assert!(result.is_ok());
        assert!(
            screen
                .statuses
                .iter()
                .flatten()
                .any(|s| s == "Search cleared"),
            "expected 'Search cleared' status to be drawn"
        );
        assert!(
            screen
                .statuses
                .iter()
                .flatten()
                .any(|s| s == "Search canceled"),
            "expected 'Search canceled' status to be drawn"
        );
    }

    #[test]
    fn test_interactive_core_reload_without_path_sets_status() {
        SIGINT_RECEIVED.store(false, Ordering::SeqCst);
        let terminal = MockTerminal {
            size: mdp::terminal::Size { rows: 8, cols: 40 },
        };
        let mut events = MockEventSource::with_events(vec![
            key_press(crossterm::event::KeyCode::Char('r')),
            key_press(crossterm::event::KeyCode::Char('q')),
        ]);
        let mut screen = MockScreen::default();
        let mut markdown = "alpha beta".to_string();

        let result = super::run_interactive_pager_with(
            &terminal,
            &mut events,
            &mut screen,
            &mut markdown,
            None,
            true,
            true,
            "stdin",
            None,
        );
        assert!(result.is_ok());
        assert!(
            screen
                .statuses
                .iter()
                .flatten()
                .any(|s| s == "Reload unavailable for stdin"),
            "expected reload-unavailable status to be drawn"
        );
    }

    #[test]
    fn test_interactive_core_propagates_event_poll_error() {
        SIGINT_RECEIVED.store(false, Ordering::SeqCst);
        let terminal = MockTerminal {
            size: mdp::terminal::Size { rows: 8, cols: 40 },
        };
        let mut events = MockEventSource::with_events(vec![]);
        events.poll_error = Some(io::ErrorKind::TimedOut);
        let mut screen = MockScreen::default();
        let mut markdown = "body".to_string();
        let err = super::run_interactive_pager_with(
            &terminal,
            &mut events,
            &mut screen,
            &mut markdown,
            None,
            true,
            true,
            "stdin",
            None,
        )
        .expect_err("poll error should propagate");
        assert_eq!(err.kind(), io::ErrorKind::TimedOut);
    }

    #[test]
    fn test_interactive_core_propagates_event_read_error() {
        SIGINT_RECEIVED.store(false, Ordering::SeqCst);
        let terminal = MockTerminal {
            size: mdp::terminal::Size { rows: 8, cols: 40 },
        };
        let mut events = MockEventSource::with_events(vec![]);
        events.poll_responses.push_back(true);
        events.read_error = Some(io::ErrorKind::UnexpectedEof);
        let mut screen = MockScreen::default();
        let mut markdown = "body".to_string();
        let err = super::run_interactive_pager_with(
            &terminal,
            &mut events,
            &mut screen,
            &mut markdown,
            None,
            true,
            true,
            "stdin",
            None,
        )
        .expect_err("read error should propagate");
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_interactive_core_propagates_draw_error() {
        SIGINT_RECEIVED.store(false, Ordering::SeqCst);
        let terminal = MockTerminal {
            size: mdp::terminal::Size { rows: 8, cols: 40 },
        };
        let mut events = MockEventSource::with_events(vec![]);
        let mut screen = MockScreen {
            draw_error: Some(io::ErrorKind::WriteZero),
            ..Default::default()
        };
        let mut markdown = "body".to_string();
        let err = super::run_interactive_pager_with(
            &terminal,
            &mut events,
            &mut screen,
            &mut markdown,
            None,
            true,
            true,
            "stdin",
            None,
        )
        .expect_err("draw error should propagate");
        assert_eq!(err.kind(), io::ErrorKind::WriteZero);
    }

    #[test]
    fn test_interactive_core_propagates_help_error() {
        SIGINT_RECEIVED.store(false, Ordering::SeqCst);
        let terminal = MockTerminal {
            size: mdp::terminal::Size { rows: 8, cols: 40 },
        };
        let mut events =
            MockEventSource::with_events(vec![key_press(crossterm::event::KeyCode::Char('h'))]);
        let mut screen = MockScreen {
            help_error: Some(io::ErrorKind::Other),
            ..Default::default()
        };
        let mut markdown = "body".to_string();
        let err = super::run_interactive_pager_with(
            &terminal,
            &mut events,
            &mut screen,
            &mut markdown,
            None,
            true,
            true,
            "stdin",
            None,
        )
        .expect_err("help error should propagate");
        assert_eq!(err.kind(), io::ErrorKind::Other);
    }

    #[test]
    fn test_interactive_core_propagates_prompt_error() {
        SIGINT_RECEIVED.store(false, Ordering::SeqCst);
        let terminal = MockTerminal {
            size: mdp::terminal::Size { rows: 8, cols: 40 },
        };
        let mut events =
            MockEventSource::with_events(vec![key_press(crossterm::event::KeyCode::Char('/'))]);
        let mut screen = MockScreen {
            prompt_error: Some(io::ErrorKind::ConnectionReset),
            ..Default::default()
        };
        let mut markdown = "body".to_string();
        let err = super::run_interactive_pager_with(
            &terminal,
            &mut events,
            &mut screen,
            &mut markdown,
            None,
            true,
            true,
            "stdin",
            None,
        )
        .expect_err("prompt error should propagate");
        assert_eq!(err.kind(), io::ErrorKind::ConnectionReset);
    }

    #[test]
    fn test_interactive_core_propagates_clear_error() {
        SIGINT_RECEIVED.store(false, Ordering::SeqCst);
        let terminal = MockTerminal {
            size: mdp::terminal::Size { rows: 8, cols: 40 },
        };
        let mut events =
            MockEventSource::with_events(vec![key_press(crossterm::event::KeyCode::Char('q'))]);
        let mut screen = MockScreen {
            clear_error: Some(io::ErrorKind::BrokenPipe),
            ..Default::default()
        };
        let mut markdown = "body".to_string();
        let err = super::run_interactive_pager_with(
            &terminal,
            &mut events,
            &mut screen,
            &mut markdown,
            None,
            true,
            true,
            "stdin",
            None,
        )
        .expect_err("clear error should propagate");
        assert_eq!(err.kind(), io::ErrorKind::BrokenPipe);
    }
}

trait FrameWriter {
    fn move_to(&mut self, x: u16, y: u16) -> Result<(), io::Error>;
    fn clear_current_line(&mut self) -> Result<(), io::Error>;
    fn clear_all(&mut self) -> Result<(), io::Error>;
    fn write_str(&mut self, s: &str) -> Result<(), io::Error>;
    fn flush(&mut self) -> Result<(), io::Error>;
}

struct CrosstermFrameWriter {
    stdout: io::Stdout,
}

impl CrosstermFrameWriter {
    fn new() -> Self {
        Self {
            stdout: io::stdout(),
        }
    }
}

impl FrameWriter for CrosstermFrameWriter {
    fn move_to(&mut self, x: u16, y: u16) -> Result<(), io::Error> {
        self.stdout.execute(MoveTo(x, y))?;
        Ok(())
    }

    fn clear_current_line(&mut self) -> Result<(), io::Error> {
        self.stdout.execute(Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    fn clear_all(&mut self) -> Result<(), io::Error> {
        self.stdout.execute(Clear(ClearType::All))?;
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result<(), io::Error> {
        write!(self.stdout, "{s}")?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.stdout.flush()
    }
}

fn draw_page_with_writer(
    pager: &Pager,
    source_label: &str,
    status_message: Option<&str>,
    cache: &mut FrameCache,
    writer: &mut impl FrameWriter,
) -> Result<(), io::Error> {
    let page_size = pager.page_size();
    let (start, end) = pager.visible_range();
    let mut body_lines = Vec::with_capacity(page_size);
    for line_num in start..end {
        body_lines.push(pager.display_line(line_num).into_owned());
    }
    while body_lines.len() < page_size {
        body_lines.push(String::new());
    }

    for row in 0..page_size {
        if cache.body_lines.get(row) != body_lines.get(row) {
            writer.move_to(HORIZONTAL_PADDING, row as u16)?;
            writer.clear_current_line()?;
            if let Some(line) = body_lines.get(row) {
                writer.write_str(&format!("\x1b[0m{line}"))?;
            }
        }
    }

    let footer = build_footer_line(pager, source_label, status_message);
    if cache.footer != footer {
        writer.move_to(HORIZONTAL_PADDING, page_size as u16)?;
        writer.clear_current_line()?;
        writer.write_str(&format!("\x1b[0m{footer}"))?;
    }

    cache.body_lines = body_lines;
    cache.footer = footer;
    writer.flush()?;
    Ok(())
}

fn draw_page(
    pager: &Pager,
    source_label: &str,
    status_message: Option<&str>,
    cache: &mut FrameCache,
) -> Result<(), io::Error> {
    let mut writer = CrosstermFrameWriter::new();
    draw_page_with_writer(pager, source_label, status_message, cache, &mut writer)
}

fn build_footer_line(pager: &Pager, source_label: &str, status_message: Option<&str>) -> String {
    let mut footer = if let Some(indicator) = pager.progress_indicator() {
        format!("[{source_label}] {indicator}")
    } else {
        format!("[{source_label}]")
    };

    let search_status = pager.search_status_message();
    if !search_status.is_empty() {
        footer.push_str(" | ");
        footer.push_str(&search_status);
    }

    if let Some(message) = status_message {
        if !message.is_empty() {
            footer.push_str(" | ");
            footer.push_str(message);
        }
    }

    footer
}

fn prompt_search(pager: &Pager, source_label: &str) -> Result<Option<String>, io::Error> {
    let mut pattern = String::new();
    let mut cache = FrameCache::default();

    loop {
        let prompt = format!("/{}", pattern);
        draw_page(pager, source_label, Some(&prompt), &mut cache)?;

        let ev = event::read()?;
        let key = match ev {
            CEvent::Resize(_, _) => continue,
            CEvent::Key(k) => k,
            _ => continue,
        };

        match apply_prompt_key(&mut pattern, key) {
            PromptAction::Continue => continue,
            PromptAction::Return(v) => return Ok(v),
            PromptAction::Interrupt => {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "SIGINT"));
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PromptAction {
    Continue,
    Return(Option<String>),
    Interrupt,
}

fn apply_prompt_key(pattern: &mut String, key: crossterm::event::KeyEvent) -> PromptAction {
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return PromptAction::Continue;
    }
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return PromptAction::Interrupt;
    }
    match key.code {
        KeyCode::Enter => PromptAction::Return(Some(pattern.clone())),
        KeyCode::Esc => PromptAction::Return(None),
        KeyCode::Backspace => {
            pattern.pop();
            PromptAction::Continue
        }
        KeyCode::Char(ch) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !key.modifiers.contains(KeyModifiers::ALT)
            {
                pattern.push(ch);
            }
            PromptAction::Continue
        }
        _ => PromptAction::Continue,
    }
}

fn build_help_screen_lines(help_lines: &[String]) -> Vec<String> {
    let mut lines = help_lines.to_vec();
    lines.push(String::new());
    lines.push("Press any key to return".to_string());
    lines
}

fn draw_help(help_lines: &[String]) -> Result<(), io::Error> {
    let mut writer = CrosstermFrameWriter::new();
    writer.move_to(0, 0)?;
    writer.clear_all()?;
    for line in build_help_screen_lines(help_lines) {
        writer.write_str(&format!("{line}\r\n"))?;
    }
    writer.flush()?;
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
