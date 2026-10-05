<div align="center">

# mman

**Browse custom Markdown manual pages from the command line.**

</div>

## Status

`mman` is under active development. The current version supports deterministic
page discovery, exact topic lookup, raw Markdown output, and source-path lookup.
The interactive viewer and topic picker shown in the preview are not implemented
yet.

## Preview

<p align="center">
  <img src="assets/mman-demo.gif" alt="Concept preview of mman opening, navigating, and searching a Markdown manual page">
</p>

> [!NOTE]
> This is a concept preview. The final interface may change during implementation.

## Current features

- Recursive discovery of `.md` files in explicitly configured directories.
- Multiple documentation roots with deterministic left-to-right precedence.
- Exact, case-sensitive topic lookup.
- Nested topics such as `concepts/ownership`.
- Optional `.md` suffix when entering a topic.
- Direct-page precedence over `index.md` within the same root.
- Retention of duplicate topic sources for future source selection.
- Original, byte-for-byte Markdown output with `--raw`.
- Automatic raw output for a plain topic when the session is non-interactive.
- Selected source-path output with `--where`.
- Hidden-file and hidden-directory exclusion.
- Symlinked Markdown files, without traversing symlinked directories.
- Validation against absolute topics, hidden components, `.` and `..` traversal,
  repeated separators, and control characters.
- Invalid-root warnings while valid roots continue to work.
- Deterministic, case-insensitive typo suggestions in the library layer. These
  are not yet displayed by the CLI.

## Installation

A packaged release is not available yet. To build the current development
version from a local checkout:

```sh
git clone <repository-url>
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
  -M <PATHS>     Override MMANPATH for this invocation
      --raw      Print the original Markdown
      --where    Print the selected page's source path
  -h, --help     Print help
  -V, --version  Print version
```

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

A plain topic in a fully interactive terminal is reserved for the upcoming
terminal viewer. Until that viewer is implemented, use `--raw` explicitly.

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

- Page content and `--where` results are written to standard output.
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
duplicate retention, raw loading, typo suggestions, and command-line output.

See the project documents for the broader design and planned work:

- [`PLAN.md`](PLAN.md) — architecture, behavior, and implementation phases
- [`TEST.md`](TEST.md) — testing conventions
- [`REVIEW.md`](REVIEW.md) — review and safety checklist
- [`PAGES.md`](PAGES.md) — manual-page authoring and publishing conventions

## Not implemented yet

The following documented goals are still planned:

- Interactive Markdown terminal viewer
- Interactive topic picker
- In-page search and match highlighting
- Topic listing with `-l`
- Collection search with `-k`
- Interactive duplicate selection with `--select`
- CLI display of missing-topic suggestions
- Markdown terminal rendering
- Internal links and navigation history
- Terminal lifecycle and restoration handling

See [`PLAN.md`](PLAN.md) for the intended behavior and implementation order.
