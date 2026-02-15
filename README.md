# mdp

`mdp` is a terminal Markdown pager for people who want readable docs without opening a browser, summoning Electron, or pretending `cat` is enough.

It renders Markdown to terminal-friendly text and gives you `less`-style navigation with search and reload.

![Rust](https://img.shields.io/badge/language-Rust-000000?logo=rust)
![License](https://img.shields.io/badge/license-Unlicense-blue)
![Terminal](https://img.shields.io/badge/interface-terminal-success)
[![CI](https://github.com/rackamx/mdp/actions/workflows/ci.yml/badge.svg)](https://github.com/rackamx/mdp/actions/workflows/ci.yml)
[![Coverage](https://github.com/rackamx/mdp/actions/workflows/coverage.yml/badge.svg)](https://github.com/rackamx/mdp/actions/workflows/coverage.yml)

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
mdp --width 100 SPEC.md
```

Strikethrough mode:

```bash
mdp --ansi-strikethrough SPEC.md
```

Benchmark:

```bash
mdp --benchmark --bench-iters 100 SPEC.md
```

## Testing

Run all tests:

```bash
cargo test
```

Get line coverage summary:

```bash
make coverage
```

Open HTML coverage report:

```bash
make coverage-open
```

Enforce coverage gate (regions/functions/lines >= 90%):

```bash
make coverage-gate
```

Run regression tests under AddressSanitizer (nightly toolchain required):

```bash
make asan-regression
```

Run CommonMark audit tests (ignored by default):

```bash
cargo test --test commonmark_spec -- --ignored --nocapture
```

## GitHub Workflows

- [CI](.github/workflows/ci.yml): formatting, clippy, and test checks on push/PR.
- [Coverage](.github/workflows/coverage.yml): enforces coverage gate and uploads coverage artifacts.
- [Sanitizers](.github/workflows/sanitizers.yml): runs ASAN regression tests.
- [Security](.github/workflows/security.yml): runs `cargo audit` and `cargo deny`.
- [Release Check](.github/workflows/release-check.yml): release builds and smoke tests on Linux/macOS.

## Project Files

- `SPEC.md`: Product specification and implementation plan, including task breakdown and current TODO checklist.
- `AGENTS.md`: Repository working rules for coding workflow (for example TDD-first and commit expectations).
- `spec.json`: CommonMark spec fixture data used by tests/audit tooling.
- `STRESS.md`: Large/complex markdown fixture used for stress/performance and rendering behavior checks.

## Scope

`mdp` targets practical terminal readability and broad CommonMark coverage. It is not trying to be a pixel-perfect HTML browser inside your shell.

If it renders your docs clearly and lets you navigate quickly, it is doing its job.

## Why not just `less` + `glow`/`bat`?

Good question. Those tools are great.

- `less` is incredible at paging, but it does not natively render Markdown structure.
- `glow`/`bat` render nicely, but are not focused on this exact pager loop.
- `mdp` is opinionated about one workflow: open markdown, navigate quickly, search everywhere, reload often, stay in one tiny binary.

Also, writing one more terminal tool is a long-standing developer tradition, and we are nothing if not respectful of tradition.
