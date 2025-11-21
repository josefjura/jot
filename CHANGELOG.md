# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2025-11-21

### Added
- Interactive note cleanup with `jot note prune` command
  - Git-rebase-style editor interface for batch note deletion
  - Supports filtering by tags, dates, and search terms
  - Default limit of 20 notes, configurable with `-n` or `--all`
  - Always requires confirmation before deletion
  - Shows first line preview of each note (truncated at 80 chars)

## [0.2.0] - Previous Release

### Added
- Profile system for isolated note databases
- Search and filtering with multiple output formats (pretty/plain/json/id)
- Editor integration with TOML frontmatter templating
- Shell completion generation (bash, zsh, fish, powershell, elvish)
- Device-based authentication flow
- CI/CD workflows for automated testing and releases
- Version bump checking for pull requests
- Command aliases (down, ls, latest)
- Tag support for organizing notes
- Date assignment with natural language parsing

### Changed
- Complete rewrite in Rust for v0.2.0
- XDG-compliant directory structure

### Fixed

## [0.1.0] - Initial Release

Initial prototype version.

[Unreleased]: https://github.com/josefjura/jot/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/josefjura/jot/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/josefjura/jot/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/josefjura/jot/releases/tag/v0.1.0
