# Changelog

All notable changes to Easy will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-10-10

### Added
- Initial release of Easy
- TUI with RGB colors and animated spinner
- Fuzzy search across command name, description, and tags
- AI-powered search using uncloseai (Arabic + English queries)
- Auto-discovery scanner for installed tools (pi, hermes, git, cargo, npm)
- Add / Edit / Delete custom commands
- Command execution with confirmation dialog
- Placeholder input dialog for parameterized commands
- Output dialog with stdout/stderr separation
- Help screen (`h` key)
- CLI flags: `--help`, `--version`
- 181 built-in commands across 10 categories:
  - Package Management, Languages, Tools, Libraries
  - Files, System, Network, TUI, Development, Databases
- Bilingual interface (Arabic + English)
- `assets/builtin_commands.json` for extending the built-in catalog
- MIT License
- Release binary for aarch64 Android (Termux)

### Technical
- Written in Rust (100%)
- Uses ratatui + crossterm for TUI
- Uses reqwest for HTTP (AI search)
- Uses serde + serde_json for config persistence
- ~6000 lines of code

[Unreleased]: https://github.com/iuhgftynhgfu/easy/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/iuhgftynhgfu/easy/releases/tag/v0.1.0
