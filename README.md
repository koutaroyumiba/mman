<div align="center">

# mman

**Browse custom Markdown manual pages from the command line.**

<a href="https://github.com/koutaroyumiba/mman/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/koutaroyumiba/mman?style=flat-square&color=ca9ee6" /></a>
<a href="https://github.com/koutaroyumiba/mman/actions/workflows/ci.yml"><img alt="CI status" src="https://img.shields.io/github/actions/workflow/status/koutaroyumiba/mman/ci.yml?branch=main&style=flat-square&label=CI" /></a>
<img alt="Platforms: macOS and Linux" src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux-8caaee?style=flat-square&logo=apple&logoColor=white" />
<img alt="Minimum Rust version: 1.88" src="https://img.shields.io/badge/Rust-1.88%2B-ef9f76?style=flat-square&logo=rust&logoColor=white" />
<a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-a6d189?style=flat-square" /></a>

</div>

## Preview

<p align="center">
  <img src="assets/mman-demo.gif" alt="mman topic picker, Markdown viewer, and in-page search">
</p>

## Current features

- Recursive discovery of `.md` files in explicitly configured directories.
- Multiple documentation roots with deterministic left-to-right precedence.
- Exact, case-sensitive topic lookup.
- Nested topics such as `concepts/ownership`.
- Optional `.md` suffix when entering a topic.
- Direct-page precedence over `index.md` within the same root.
- Interactive duplicate-source selection with `-s` or `--select`.
- Original, byte-for-byte Markdown output with `--raw`.
- Automatic raw output for a plain topic when the session is non-interactive.
- Interactive Markdown viewer with Rosé Pine colors, keyboard scrolling, and
  case-insensitive in-page search with highlighted matches.
- Built-in topic picker with case-insensitive literal filtering.
- Stable alphabetical topic listing with `-l` or `--list`.
- Collection search across topic names and highest-precedence page content with
  `-k` or `--search`.
- Rendering for headings, paragraphs, emphasis, links, quotes, lists, code
  blocks, thematic breaks, and Unicode-aware wrapping.
- Man-page-style top headers with the page topic on both sides and an
  `[mman manual]` label centered between them.
- Rosé Pine syntax highlighting for recognized fenced-code languages, with a
  readable plain-code fallback for unknown languages.
- Bordered code blocks that remain visually distinct from surrounding prose.
- Selected source-path output with `--where`.
- Hidden-file and hidden-directory exclusion.
- Symlinked Markdown files, without traversing symlinked directories.
- Validation against absolute topics, hidden components, `.` and `..` traversal,
  repeated separators, and control characters.
- Invalid-root warnings while valid roots continue to work.
- Deterministic, case-insensitive typo suggestions for missing topics.

## Installation

A packaged release is not available yet. Building from source requires Rust
1.88 or newer:

```sh
git clone https://github.com/koutaroyumiba/mman.git
cd mman
cargo build --release
```

The executable is created at:

```text
target/release/mman
```

You can also install the checkout into Cargo's binary directory:

```sh
cargo install --path .
```

## Configure documentation roots

`mman` has no implicit documentation directory. Configure one or more roots
with `MMANPATH`:

```sh
export MMANPATH="$HOME/workspace/mman-pages:$HOME/work/docs"
```

On macOS and Linux, roots are separated by `:`. They are searched from left to
right, and the first matching page wins.

Use `-M` to replace `MMANPATH` for one invocation:

```sh
mman -M "$HOME/manuals:$HOME/project/docs" --raw concepts/ownership
```

Path behavior:

- Empty path entries are ignored; they do not mean the current directory.
- A leading `~` is expanded when it is a distinct first path component.
- Duplicate roots are removed without changing precedence.
- Missing, unreadable, and non-directory roots produce warnings.
- The command fails when no root is configured or no configured root is usable.

The repository's `examples/` directory can be used as a documentation root
while developing:

```sh
mman -M examples --raw concepts/ownership
```

## Usage

```text
mman [OPTIONS] [TOPIC]

Arguments:
  [TOPIC]  Topic to open

Options:
  -M <PATHS>           Override MMANPATH for this invocation
      --raw            Print the original Markdown
      --where          Print the selected page's source path
  -s, --select         Select among duplicate page sources
  -l, --list           List available topics
  -k, --search <TERM>  Search topic names and page content
  -h, --help           Print help
  -V, --version        Print version
```

### Pick a topic interactively

Run `mman` without a topic in an interactive terminal:

```sh
mman
```

Type to filter topic names, use the arrow keys or `j`/`k` to move, press `Enter`
to open the selected topic, and press `Esc`, `q`, or `Ctrl-C` to quit.

### Select a duplicate source

```sh
mman --select concepts/ownership
```

When a topic exists in multiple roots, the source picker displays each full
path. A topic with only one source opens directly. Press `Esc`, `q`, or `Ctrl-C`
to cancel. Source selection requires an interactive terminal.

### List available topics

```sh
mman -l
```

Listing prints each unique topic once in alphabetical order, including nested
topics such as `concepts/ownership`.

### Search the page collection

```sh
mman -k branch
mman -k "working tree"
```

Search uses case-insensitive literal matching over topic names and raw Markdown
content. Results are ordered by topic and include up to three matching source
lines with one-based line numbers. Only the highest-precedence copy of each
topic is searched.

### Print original Markdown

```sh
mman --raw concepts/ownership
```

Raw output preserves the source bytes exactly, including line endings and the
presence or absence of a final newline.

A plain topic also emits raw Markdown when stdin or stdout is not an interactive
terminal:

```sh
mman concepts/ownership | grep borrow
mman concepts/ownership > ownership.md
```

### Open the interactive viewer

In a fully interactive terminal, open a topic without an output option:

```sh
mman concepts/ownership
```

Current viewer keybindings:

```text
j / Down          Scroll down
k / Up            Scroll up
Ctrl-d / PageDown Move down by one page
Ctrl-u / PageUp   Move up by one page
h / Left          Pan horizontally left
l / Right         Pan horizontally right
g / Home          Go to the beginning
G / End           Go to the end
/                 Enter in-page search
n                 Go to the next match
N                 Go to the previous match
Esc               Clear the active search; quit when no search is active
?                 Toggle keybinding help
q                 Quit
Ctrl-C            Quit
```

The viewer reflows content when the terminal is resized. An RAII terminal guard
and panic hook restore raw mode, cursor visibility, and the alternate screen on
normal exits, controlled errors, interruption, and unwinding.

### Print the selected source path

```sh
mman --where concepts/ownership
```

This applies normal root and page-form precedence and prints the selected
Markdown path.

## Page discovery

Only files ending in `.md` are recognized. Topic names are derived as follows:

```text
<root>/git.md                       -> git
<root>/languages/rust.md            -> languages/rust
<root>/concepts/ownership/index.md  -> concepts/ownership
```

Topics can be entered with or without the extension:

```sh
mman --raw concepts/ownership
mman --raw concepts/ownership.md
```

### Precedence

For duplicate topics, precedence is:

1. Earlier configured roots before later roots.
2. A direct page such as `ownership.md` before `ownership/index.md` within the
   same root.
3. Stable path order as a deterministic tie-breaker.

Root precedence is stronger than page form. Therefore, an index page in the
first root wins over a direct page in the second root.

### Safety rules

The following topic forms are rejected:

```text
/absolute/topic
./topic
../topic
concepts/../topic
concepts/.private/topic
concepts//topic
```

Markdown is treated as data. The current implementation does not execute code,
interpret embedded HTML, fetch remote resources, or invoke external renderers.

## Exit behavior

- Page content, topic listings, collection-search results, and `--where`
  results are written to standard output.
- Warnings and errors are written to standard error.
- Successful commands return status `0`.
- Invalid usage, configuration failures, and missing topics return a non-zero
  status.

## Development

Run the standard checks with:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

The implementation currently includes focused tests for CLI parsing, execution
mode selection, path handling, topic validation, page discovery, precedence,
duplicate retention, raw loading, typo suggestions, viewer state transitions,
terminal-buffer rendering, and command-line output.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history.

## License

Licensed under the [MIT License](LICENSE).

## Not implemented yet

The following documented goals are still planned:

- Internal links and navigation history
- Link focus and external URL actions
