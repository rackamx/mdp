# mdp

`mdp` is a terminal Markdown pager for people who want readable docs without opening a browser, summoning Electron, or pretending `cat` is enough.

It renders Markdown to terminal-friendly text and gives you `less`-style navigation with search and reload.

![Rust](https://img.shields.io/badge/language-Rust-000000?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![Terminal](https://img.shields.io/badge/interface-terminal-success)

## Philosophy

`mdp` believes:

- Documentation should be fast.
- Your terminal is already a perfectly good UI.
- Markdown should be readable, not treated like a negotiation with CSS.
- If your markdown viewer needs a GPU, we have made several mistakes.

In short: this is a markdown pager that tries to stay out of your way, except when it can be useful.

## Features

- Interactive pager mode with:
- `j`/`k`, arrows, `PgUp`/`PgDn`, `g`/`G`, `Enter`, `space`
- `/` search, `n` next, `N` previous
- `r` reload file from disk
- `h`/`?` help
- `q`/`Q`/`ZZ` quit
- Search across the whole document (forward from current position, with wrap)
- Highlighted current search match
- Stdin support (`cat file.md | mdp`)
- Non-interactive render mode (prints rendered output when stdout is not a TTY)
- Unicode strikethrough fallback by default (with `--ansi-strikethrough` override)
- Built-in benchmark mode (`--benchmark`)

## Install / Build

```bash
cargo build --release
```

Binary:

```bash
./target/release/mdp
```

## Usage

Basic:

```bash
mdp README.md
```

From stdin:

```bash
cat README.md | mdp
```

Explicit stdin:

```bash
cat README.md | mdp -
```

Width override:

```bash
mdp --width 100 PLAN.md
```

Strikethrough mode:

```bash
mdp --ansi-strikethrough PLAN.md
```

Benchmark:

```bash
mdp --benchmark --bench-iters 100 PLAN.md
```

## Testing

Run all tests:

```bash
cargo test
```

Run CommonMark audit tests (ignored by default):

```bash
cargo test --test commonmark_spec -- --ignored --nocapture
```

## Scope

`mdp` targets practical terminal readability and broad CommonMark coverage. It is not trying to be a pixel-perfect HTML browser inside your shell.

If it renders your docs clearly and lets you navigate quickly, it is doing its job.

## Why not just `less` + `glow`/`bat`?

Good question. Those tools are great.

- `less` is incredible at paging, but it does not natively render Markdown structure.
- `glow`/`bat` render nicely, but are not focused on this exact pager loop.
- `mdp` is opinionated about one workflow: open markdown, navigate quickly, search everywhere, reload often, stay in one tiny binary.

Also, writing one more terminal tool is a long-standing developer tradition, and we are nothing if not respectful of tradition.
