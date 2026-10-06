# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-10-06

### Added

- Recursive Markdown page discovery across explicitly configured roots.
- Deterministic root, direct-page, and index-page precedence.
- Exact topic lookup with nested topics and typo suggestions.
- Raw Markdown output and automatic non-interactive output.
- Source-path lookup with `--where`.
- Alphabetical topic listing with `-l` and `--list`.
- Collection search with `-k` and `--search`, including source-line previews.
- Interactive topic and duplicate-source pickers.
- Interactive Markdown viewer with Unicode-aware layout and scrolling.
- Rendering for headings, paragraphs, emphasis, lists, quotes, links, thematic
  breaks, inline code, and fenced code blocks.
- Rosé Pine terminal styling and fenced-code syntax highlighting.
- Case-insensitive in-page search with match navigation and highlighting.
- Terminal lifecycle restoration on normal exit, controlled errors, `Ctrl-C`,
  and panic unwinding.
- macOS and Linux automated test coverage.

### Security

- Reject absolute, hidden, current-directory, parent-directory, malformed, and
  control-character topic paths.
- Ignore hidden pages and symlinked directories.
- Sanitize terminal control characters and treat Markdown as inert data.

[Unreleased]: https://github.com/koutaroyumiba/mman/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/koutaroyumiba/mman/releases/tag/v0.1.0
