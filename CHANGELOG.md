# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `jot show` command to display a note with full details
  - Shows complete note ID, timestamps, tags, date, and full content
  - Supports all output formats: pretty (default), plain, json, id
  - Can show specific note by ID or latest note (default)
  - Available as `jot show [ID]` or `jot note show [ID]`
  - Displays human-readable timestamps (e.g., "2025-11-21 16:58:19")

### Changed
- **BREAKING**: Renamed internal `date` field to `subject_date` for clarity
  - The date field now semantically represents "what date this note is about" rather than when it was created
  - Database automatically migrates from v1 to v2 schema on first run
  - Search ordering now uses subject_date with created_at as fallback: notes are ordered by their subject date (or creation date if no subject date is set)
  - Added database versioning system using SQLite's `PRAGMA user_version`
  - API change: `Note.date` → `Note.subject_date` in all code

### Fixed
- Running `jot` with no arguments now displays help message instead of doing nothing

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
