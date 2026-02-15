# mdless - Markdown Pager Plan

## Context

Build a Rust-based markdown pager similar to `less` that interprets and renders markdown files according to the CommonMark spec v0.31.2. No colors - only text formatting like bold. The goal is a functional pager that can display markdown files with basic formatting.

## Architecture

- **Language**: Rust
- **Markdown Parser**: `pulldown-cmark` - a popular, CommonMark-compliant Rust markdown parser
- **Terminal UI**: `crossterm` - cross-platform terminal handling
- **Output**: Plain text with ANSI escape codes for bold (no colors)

## Key Files to Create

- `Cargo.toml` - Rust project configuration
- `src/main.rs` - Entry point and CLI argument handling
- `src/renderer.rs` - Terminal rendering logic
- `src/pager.rs` - Paging logic (like less)
- `tests/` - Unit and integration tests

## Task Breakdown

### T01 - Project Setup
- Initialize Rust project with `cargo new mdless`
- Add dependencies: `pulldown-cmark`, `crossterm`, `clap` (for CLI args)
- Verify project compiles with `cargo build`

**Test**: None - just verify compilation

**Status**: DONE

---

### T02 - Basic File Loading
- Accept file path as CLI argument
- Read markdown file content into memory
- Handle file not found errors gracefully
- Print usage if no file provided

**Tests**:
- `tests/file_loading.rs`:
  - `test_read_file_contents()` - read a markdown file and verify content
  - `test_file_not_found()` - verify error handling for missing file
  - `test_no_argument_prints_usage()` - verify usage message when no args

**Status**: DONE

---

### T03 - Markdown Parsing
- Use `pulldown-cmark` to parse markdown into events
- Create event stream for rendering
- Handle basic blocks: paragraphs, headings

**Tests**:
- `tests/parsing.rs`:
  - `test_parse_paragraph()` - parse simple paragraph, verify Event::Text
  - `test_parse_heading()` - parse `# Heading`, verify Event::Start(Heading(...))
  - `test_parse_multiple_blocks()` - parse multi-block markdown, verify all events

**Status**: DONE

---

### T04 - Terminal Setup
- Initialize crossterm terminal
- Set raw mode for input handling
- Restore terminal on exit

**Test**:
- `tests/terminal.rs`:
  - `test_terminal_cleanup()` - verify terminal restores on panic (can use RAII pattern test)

**Status**: DONE

---

### T05 - Basic Text Rendering
- Render plain text blocks
- Handle line wrapping to terminal width
- Track cursor position

**Tests**:
- `tests/rendering.rs`:
  - `test_render_plain_text()` - render "Hello world", verify output
  - `test_render_text_with_wrapping()` - render long line, verify wrapped at 80 chars
  - `test_render_multiple_lines()` - render 3 paragraphs, verify line breaks

**Status**: DONE

---

### T06 - Bold Text Rendering
- Detect strong emphasis (`**text**` or `__text__`)
- Render bold using ANSI escape codes: `\x1b[1m` for bold on, `\x1b[0m` for bold off
- No colors - only bold formatting

**Tests**:
- `tests/rendering.rs`:
  - `test_render_bold_asterisks()` - render `**bold**`, verify `\x1b[1m` and `\x1b[0m`
  - `test_render_bold_underscores()` - render `__bold__`, verify ANSI codes
  - `test_render_bold_mixed()` - render "normal **bold** normal", verify structure

**Status**: DONE

---

### T06b - Italics Text Rendering
- Detect emphasis (`*text*` or `_text_`)
- Render italics using ANSI escape codes: `\x1b[3m` for italics on, `\x1b[0m` for off

**Tests**:
- `tests/rendering.rs`:
  - `test_render_italics_asterisks()` - render `*italic*`, verify `\x1b[3m` codes
  - `test_render_italics_underscores()` - render `_italic_`, verify ANSI codes
  - `test_render_italics_mixed()` - render "normal *italic* normal"

**Status**: DONE

---

### T07 - Heading Rendering
- Render ATX headings (`#` to `######`)
- Display heading level visually (e.g., "===" underline for h1)
- Bold the heading text

**Tests**:
- `tests/rendering.rs`:
  - `test_render_h1()` - render `# Title`, verify "===" underline
  - `test_render_h2()` - render `## Title`, verify "---" underline
  - `test_render_h3_to_h6()` - verify heading markers for h3-h6
  - `test_render_heading_bold()` - verify heading text is bolded

**Status**: DONE

---

### T08 - Code Block Rendering
- Render fenced code blocks
- Use monospace font indication (if possible)
- Preserve indentation

**Tests**:
- `tests/rendering.rs`:
  - `test_render_fenced_code_block()` - render ```rust\ncode\n```, verify fence markers
  - `test_render_indented_code_block()` - render indented code (4 spaces)
  - `test_render_inline_code()` - render `\`code\``, verify backticks preserved

**Status**: DONE

---

### T09 - Block Quote Rendering
- Render block quotes with `>` prefix
- Indent block quote content

**Tests**:
- `tests/rendering.rs`:
  - `test_render_block_quote()` - render `> quote`, verify `|` prefix
  - `test_render_block_quote_multiline()` - render multi-line block quote

**Status**: DONE

---

### T10 - List Rendering
- Render bullet lists (`-`, `*`, `+`)
- Render ordered lists (`1.`, `2.`, etc.)
- Handle nested lists with indentation

**Tests**:
- `tests/rendering.rs`:
  - `test_render_bullet_list()` - render `- item`, verify `* ` prefix
  - `test_render_ordered_list()` - render `1. item`, verify `1. ` prefix
  - `test_render_nested_list()` - render nested lists, verify indentation

**Status**: DONE

---

### T11 - Link Rendering
- Render links as: `text (url)`
- Skip image rendering (just show alt text)

**Tests**:
- `tests/rendering.rs`:
  - `test_render_link()` - render `[link](http://url)`, verify "link (http://url)"
  - `test_render_reference_link()` - render `[text][ref]`, verify fallback
  - `test_render_image_alt_text()` - render `![alt](url)`, verify "[alt]"

**Status**: DONE

---

### T11b - Auto-link Rendering
- Render auto-links: `<https://example.com>` or `<user@example.com>`
- Display as the URL in parentheses: `https://example.com (https://example.com)`
- Handle email auto-links

**Tests**:
- `tests/rendering.rs`:
  - `test_render_url_auto_link()` - render `<https://example.com>`, verify URL shown
  - `test_render_email_auto_link()` - render `<user@example.com>`, verify email shown

**Status**: DONE

---

### T11c - Escape Sequence Handling
- Handle markdown escape sequences: `\*`, `\[`, `\\`, etc.
- Display escaped characters as literal text

**Tests**:
- `tests/rendering.rs`:
  - `test_render_escape_asterisk()` - render `\*`, verify `*`
  - `test_render_escape_bracket()` - render `\[`, verify `[`
  - `test_render_escape_backslash()` - render `\\`, verify `\`
  - `test_render_multiple_escapes()` - render `\*text\*`, verify `*text*`

**Status**: DONE

---

### T12 - Basic Paging
- Implement page-down (Space, PageDown)
- Implement page-up (PageUp, b)
- Show progress indicator (e.g., "--- More ---" or line count)

**Tests**:
- `tests/pager.rs`:
  - `test_page_down()` - verify content scrolls down one page
  - `test_page_up()` - verify content scrolls up one page
  - `test_progress_indicator()` - verify "more" indicator shows at bottom

**Status**: DONE

---

### T13 - Navigation
- Implement scroll by line (Enter, Down Arrow)
- Implement scroll by page (Space, PageDown, b, PageUp)
- Handle Up/Down arrows

**Tests**:
- `tests/pager.rs`:
  - `test_scroll_down_line()` - verify single line scroll
  - `test_scroll_up_line()` - verify single line scroll up
  - `test_scroll_to_end()` - verify behavior at EOF
  - `test_scroll_to_beginning()` - verify behavior at BOF

---

### T14 - Search (Optional Enhancement)
- Add basic search with `/` pattern
- Highlight matches

**Tests**:
- `tests/pager.rs`:
  - `test_search_forward()` - search for pattern, verify match found
  - `test_search_highlight()` - verify match is highlighted with bold
  - `test_search_no_match()` - verify "Pattern not found" message
  - `test_search_next()` - verify `n` key finds next match

---

### T15 - Help Screen
- Show available keybindings with `h` or `?`
- List navigation commands

**Tests**:
- `tests/pager.rs`:
  - `test_help_screen()` - verify help displays on `h` press
  - `test_help_keybindings()` - verify all keybindings listed

---

### T16 - Horizontal Rule Rendering
- Render horizontal rules (`---`, `***`, `___`)
- Use `---` as consistent output

**Tests**:
- `tests/rendering.rs`:
  - `test_render_horizontal_rule_dashes()` - render `---`, verify "---" output
  - `test_render_horizontal_rule_asterisks()` - render `***`, verify "---" output
  - `test_render_horizontal_rule_underscores()` - render `___`, verify "---" output

---

### T17 - Strikethrough Text Rendering
- Detect strikethrough (`~~text~~`)
- Render using strikethrough ANSI escape codes: `\x1b[9m` on, `\x1b[0m` off

**Tests**:
- `tests/rendering.rs`:
  - `test_render_strikethrough()` - render `~~text~~`, verify `\x1b[9m` codes
  - `test_render_strikethrough_mixed()` - render "normal ~~strikethrough~~ normal"

---

### T18 - Line Break Handling
- Distinguish between soft breaks (single newline) and hard breaks (two spaces + newline)
- Option to render soft breaks as spaces or line breaks

**Tests**:
- `tests/rendering.rs`:
  - `test_render_soft_break()` - single newline in paragraph renders as space
  - `test_render_hard_break()` - two spaces + newline renders as line break

---

### T19 - Goto Functionality
- Implement `g` to go to beginning of file
- Implement `G` to go to end of file
- Implement `<number>g` to go to specific line (optional)

**Tests**:
- `tests/pager.rs`:
  - `test_goto_beginning()` - verify `g` jumps to first line
  - `test_goto_end()` - verify `G` jumps to last line

---

### T20 - Empty File Handling
- Handle empty markdown files gracefully
- Display appropriate message

**Tests**:
- `tests/file_loading.rs`:
  - `test_empty_file()` - verify graceful handling of empty .md file

---

### T21 - Long Line Handling
- Handle lines significantly exceeding terminal width
- Proper wrapping for very long lines

**Tests**:
- `tests/rendering.rs`:
  - `test_render_very_long_line()` - verify wrapping for lines > 200 chars

---

### T22 - Unicode/UTF-8 Support
- Support non-ASCII characters (émoji, accented characters, CJK)
- Proper width calculation for double-width characters

**Tests**:
- `tests/rendering.rs`:
  - `test_render_unicode_text()` - verify UTF-8 characters render correctly
  - `test_render_emoji()` - verify emoji display
  - `test_render_cjk_characters()` - verify CJK characters display correctly

---

### T23 - Terminal Resize Handling
- Detect terminal resize events
- Recalculate display width on resize

**Tests**:
- `tests/terminal.rs`:
  - `test_terminal_resize()` - verify rendering adapts to new terminal size

---

### T24 - Malformed Markdown Handling
- Handle invalid/edge-case markdown syntax gracefully
- Don't panic on malformed input

**Tests**:
- `tests/parsing.rs`:
  - `test_parse_unclosed_fence()` - handle unclosed code fence
  - `test_parse_mismatched_brackets()` - handle `[text` without closing bracket
  - `test_parse_incomplete_emphasis()` - handle unclosed `**`

---

### T25 - Exit Command
- Implement `q` key to quit
- Verify exit code on normal quit

**Tests**:
- `tests/pager.rs`:
  - `test_exit_command()` - verify `q` exits the program
  - `test_exit_code()` - verify program returns 0 on normal exit

---

### T26 - File Name Display
- Show current file name in header/footer
- Display "(stdin)" when reading from stdin

**Tests**:
- `tests/pager.rs`:
  - `test_display_filename()` - verify file name shown in header
  - `test_stdin_label()` - verify "(stdin)" shown for stdin input

---

### T27 - CLI Options and Default Width
- Default width: detect terminal size and subtract 4 characters (2 margin on each side)
- Minimum width: 40 characters (fallback if terminal too small)
- Add `--width` option to override default width
- Add `--help` option (clap provides this)
- Add `--version` option

**Tests**:
- `tests/file_loading.rs`:
  - `test_default_width_detection()` - verify width = terminal_width - 4
  - `test_minimum_width()` - verify fallback to 40 when terminal < 44
  - `test_width_option()` - verify `--width` sets custom column width
  - `test_help_output()` - verify help message displays correctly

---

### T28 - Exit Codes
- Return 0 on normal exit
- Return 1 on file read error
- Return 2 on other errors

**Tests**:
- `tests/file_loading.rs`:
  - `test_exit_code_success()` - verify returns 0 on success
  - `test_exit_code_file_error()` - verify returns 1 on file error

---

### T29 - Integration Tests
- Create sample markdown files for testing
- Run binary against real files

**Tests**:
- `tests/integration.rs`:
  - `test_full_render_sample()` - render complete sample.md file
  - `test_navigation_sample()` - test navigation with sample file

---

### T30 - Stdin Support
- Read from stdin when no file is provided or with `-` argument
- Support piping: `cat file.md | mdless`
- Support stdin with other flags

**Tests**:
- `tests/file_loading.rs`:
  - `test_read_from_stdin()` - verify stdin reading works
  - `test_stdin_with_flag()` - verify `mdless -` reads from stdin

---

### T31 - Table Rendering
- Render tables using aligned columns
- Handle headers and borders
- Fallback to plain text if table is too wide

**Tests**:
- `tests/rendering.rs`:
  - `test_render_simple_table()` - render simple markdown table
  - `test_render_table_alignment()` - verify left/right/center alignment
  - `test_render_table_fallback()` - verify fallback for wide tables

---

### T32 - Task List Rendering
- Render task lists with `- [ ]` (unchecked) and `- [x]` / `- [X]` (checked)
- Show `[x]` or `[ ]` prefix for tasks

**Tests**:
- `tests/rendering.rs`:
  - `test_render_task_list_unchecked()` - render `- [ ] item`
  - `test_render_task_list_checked()` - render `- [x] item`
  - `test_render_task_list_mixed()` - render mixed checked/unchecked

---

### T33 - Signal Handling
- Handle SIGINT (Ctrl+C) gracefully
- Clean up terminal on signal
- Return appropriate exit code on interrupt

**Tests**:
- `tests/terminal.rs`:
  - `test_ctrl_c_cleanup()` - verify terminal restores on Ctrl+C
  - `test_sigint_exit_code()` - verify returns 130 on SIGINT

---

### T34 - Footnote Rendering
- Render reference footnotes `[^1]` with definitions
- Display footnote content at end or inline

**Tests**:
- `tests/rendering.rs`:
  - `test_render_footnote_reference()` - render `[^1]` in text
  - `test_render_footnote_definition()` - render `[^1]: definition`

---

### T35 - Definition List Rendering
- Render definition lists: term on one line, definition indented below

**Tests**:
- `tests/rendering.rs`:
  - `test_render_definition_list()` - render "Term : Definition" format

---

### T36 - Smart Punctuation
- Convert `...` to ellipsis `…`
- Convert `--` to en-dash `–`
- Convert `---` to em-dash `—`
- Convert straight quotes to curly quotes

**Tests**:
- `tests/rendering.rs`:
  - `test_render_ellipsis()` - verify `...` → `…`
  - `test_render_endash()` - verify `--` → `–`
  - `test_render_emdash()` - verify `---` → `—`

---

### T37 - Binary File Detection
- Detect binary files before rendering
- Display error message for binary files
- Check for null bytes or other binary indicators

**Tests**:
- `tests/file_loading.rs`:
  - `test_binary_file_rejected()` - verify binary file shows error
  - `test_text_file_accepted()` - verify text file works normally

---

## Implementation Scope

Full implementation: All 37 tasks (T01-T33, plus T06b, T11b, T11c, T34-T37)

## Verification

After implementing tasks:
1. Run `cargo build` to verify compilation
2. Run `cargo test` to run all tests
3. Test with sample markdown files: `cargo run -- README.md`
4. Verify bold and italic text renders correctly in terminal
5. Test navigation: scroll up/down, page up/down, goto beginning/end
6. Verify help screen displays with `h` key
7. Test all edge cases: empty files, long lines, unicode, malformed markdown
8. Verify exit codes
9. Test CLI options work correctly
10. Test stdin: `cat file.md | mdless`
11. Test table rendering
12. Test task lists
13. Test Ctrl+C handling
