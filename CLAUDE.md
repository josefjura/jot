# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Jot is a terminal-first note-taking application designed for quick, frictionless note capture. The core philosophy is that opening traditional note apps (Obsidian, OneNote) involves too much overhead (deciding where to save, how to name) when you just want to jot something down quickly. Since terminals are always open for developers, Jot provides instant note capture from the command line.

The application consists of two Rust components:
- **CLI** (`cli/`): Command-line interface for creating, searching, and managing notes (v0.2.0)
- **Server** (`server/`): REST API backend using Axum framework with SQLite database, designed to run as a Docker container for self-hosted, centralized note storage

### Key Features & Design Goals

- **Instant capture**: `jot note add "my quick thought"` or `jot down "quick thought"` - no naming, no navigation required
- **Profile system**: Multiple isolated note databases (`jot profile use work` switches contexts)
- **Date assignment**: `-d today` (or other date expressions) automatically organizes notes chronologically
- **Tag support**: `-t work,important` for organizing and filtering notes
- **Editor integration**: `-e` flag opens `$EDITOR` (or `$VISUAL`) for longer-form note editing with TOML frontmatter template
- **Templating**: Fully implemented - notes opened in editor have TOML frontmatter (tags, date) separated from content by `+++` delimiter
- **Search & filtering**: Flexible search with `jot note search` (or `jot ls`) supporting term matching, tag filters, date filters, and multiple output formats (pretty/plain/json/id)
- **Shell completions**: Built-in completion generation for bash, zsh, fish, powershell, elvish
- **Scripting support**: Quiet mode (`-q`) outputs only note IDs for pipeline integration
- **Device-based authentication**: OAuth-like device flow - CLI generates code, opens browser for login, polls for token
- **Self-hosted**: Server designed for deployment (Docker support not yet implemented)
- **Quality focus**: Emphasis on comprehensive testing and clean code despite being a learning project (third Rust project)

### CLI Usage Examples

```bash
# Quick note capture
jot down "meeting notes"
jot note add -t work,urgent "bug in production"

# Editor mode for longer notes
jot note add -e

# Search and filter
jot ls                          # List all notes (alias for 'note search')
jot ls -t work -n 5             # Last 5 work notes
jot note search "meeting" -d today
jot note search --output id     # Get IDs for scripting

# Get latest note
jot note last                   # or 'jot note latest'
NOTE_ID=$(jot note add -q "scripting note")  # Quiet mode

# Profile management
jot profile                     # Show current profile
jot profile use work            # Switch to work profile
jot profile list                # List all profiles

# Shell completions
jot completion bash > /etc/bash_completion.d/jot
```

### Profile System

Profiles provide isolated note databases for different contexts (personal, work, projects):

**Directory Structure:**
```
~/.config/jot/
├── current                      # Active profile name
└── profiles/
    ├── default.toml            # Profile configs
    ├── work.toml
    └── personal.toml

~/.local/share/jot/
└── profiles/
    ├── default/notes.db        # Separate DBs per profile
    ├── work/notes.db
    └── personal/notes.db
```

**Profile Features:**
- `default_tags = ["work"]` in profile config auto-applies tags
- Profiles created on-demand when switching
- XDG-compliant directory structure
- Profile specified via `-p/--profile` flag or `JOT_PROFILE` env var

## Development Commands

### Server

```bash
# Run server (requires .env file with JOT_HOST, JOT_PORT, JOT_JWT_SECRET, DATABASE_PATH)
cd server
cargo run

# Run server tests
cd server
cargo test

# Build server
cd server
cargo build --release
```

### CLI

```bash
# Run CLI (workspace root)
cargo run --bin jot-cli -- <command>

# Run CLI tests
cargo test --package jot-cli

# Build CLI release (outputs to target/release/jot-cli)
cargo build --release --package jot-cli

# Binary size analysis
cargo bloat --release --package jot-cli --crates
```

**Note:** This is a Cargo workspace. Build artifacts are in the workspace root `target/` directory, not `cli/target/`.

## Architecture

### Server (`server/`)

The server is built with Axum and follows a layered architecture:

- **`main.rs`**: Entry point, sets up tracing, loads environment variables, creates database pool, and starts server
- **`router/`**: Route definitions organized by domain (auth, note, health, openapi)
  - Uses `aide` crate for OpenAPI documentation generation
  - Auth middleware applied via `with_auth_middleware()` for protected routes
- **`db/`**: Database access layer using SQLx with SQLite
  - Migrations in `migrations/` directory (run automatically via `sqlx::migrate!()` on startup)
  - Database schema: users, notes (with tags stored as text), device_auth
- **`model/`**: Data models and DTOs (auth, user, note)
- **`jwt.rs`**: JWT token generation and validation
- **`middleware.rs`**: Authentication middleware
- **`state.rs`**: Application state (database pool and JWT secret)
- **`errors/`**: Error types and HTTP error responses

Database migrations are automatically applied on server startup in `db/create_db_pool()`.

### CLI (`cli/`)

The CLI uses Clap for argument parsing and follows a command-based architecture:

- **`main.rs`**: Entry point, parses args, loads profile, dispatches to commands
- **`args/`**: Command-line argument definitions using Clap derive API
- **`commands/`**: Command implementations
  - `config`: Display current configuration
  - `profile`: Profile management (use, list, current)
  - `note`: Note operations (add, search, last/latest, edit, delete)
- **`profile.rs`**: Profile management (TOML configuration files, XDG directories)
- **`app_config.rs`**: Application configuration merging CLI args, profile, and defaults
- **`editor.rs`**: External editor integration with TOML frontmatter
- **`formatters.rs`**: Output formatting (pretty, plain, json, id)
- **`utils/`**: Date parsing utilities (`DateSource`, `DateTarget`)
- **`db/`**: Local SQLite database operations using `jot-core`

Configuration precedence: CLI flags > Profile file > Current profile > Defaults

### Testing

**Server tests** (`server/src/test/`):
- Uses `axum-test` crate to create test servers
- `setup_server()` helper creates TestServer with in-memory database
- `login()` helper performs authentication and returns JWT token
- Tests organized by domain (auth, health, note)

**CLI tests** (`cli/src/test/`):
- Uses `assert_cmd` for integration testing of CLI binary
- `TestDb` helper creates isolated profile environments with unique profile names and temp XDG directories
- Tests use `XDG_CONFIG_HOME` and `XDG_DATA_HOME` env vars to isolate test data
- Profile names are generated using UUID to ensure test isolation
- All 41 tests pass ✅

## Code Style & Standards

Both projects enforce strict clippy lints:
```rust
#![deny(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
```

**Critical Rules:**
- Avoid `.unwrap()`, `.expect()`, and `panic!()` - use proper error handling with `Result` types
- This applies to both production AND test code - no exceptions
- Use `map_err()`, `context()`, or `?` operator for error propagation

**Security Considerations:**
- **Server**: Always verify note ownership in endpoints - authenticated user must own the resource
- **Server**: Filter database queries by `user_id` - never return cross-user data
- **Server**: Use SQLite syntax in migrations (not PostgreSQL `SERIAL`, `TIMESTAMP`)
- **CLI**: Token storage must have restrictive permissions (0600 on Unix)
- **CLI**: Validate and sanitize all user-provided paths
- **Both**: Be cautious with string delimiters - ensure they can be escaped or are unlikely to appear in user content

**Template Parsing:**
- The `+++` delimiter is line-based - only recognized when appearing on its own line (with optional whitespace)
- This allows `+++` to safely appear within note content (e.g., "Learning C+++" works fine)
- When modifying template parsing, maintain this line-based approach to avoid reintroducing delimiter collision bugs

**Profile System Implementation:**
- Profile names are simple strings, not file paths
- XDG directories are respected: check `XDG_CONFIG_HOME` and `XDG_DATA_HOME` env vars first, fall back to `directories` crate
- When XDG env vars are set, append `/jot` subdirectory to match `ProjectDirs` behavior
- Profile config files: `$XDG_CONFIG_HOME/jot/profiles/{name}.toml`
- Profile databases: `$XDG_DATA_HOME/jot/profiles/{name}/notes.db`
- Current profile stored in: `$XDG_CONFIG_HOME/jot/current`

## CLI Design Principles

**Unix Conventions:**
- Short flags for common operations: `-t` for tags, `-e` for editor, `-n` for limit, `-L` for lines, `-q` for quiet
- Sensible defaults that minimize typing for common operations
- Pipeline-friendly with `--output id` and `-q` modes
- Shell completion support for better UX

**Command Aliases:**
- `jot down` = `jot note add` (quick capture metaphor)
- `jot ls` = `jot note search` (familiar listing command)
- `jot note latest` = `jot note last` (natural language variation)
- `jot profile` = `jot profile current` (show current by default)

**Output Formats:**
- `pretty` (default): Colored, human-readable output
- `plain`: No colors, for piping/logging
- `json`: Machine-readable structured data
- `id`: ID-only output for scripting (one per line)

## Dependency Management

The project is optimized for binary size:
- Removed heavyweight dependencies: `config`, `tokio`, `async-trait`, `rand`, `cliclack`
- Uses `toml` crate directly instead of multi-format `config` crate
- CLI is fully synchronous - no async runtime overhead
- Final binary: ~4.2 MB (release build, unstripped)

**Key Dependencies:**
- `clap` + `clap_complete`: CLI parsing and shell completions
- `jot-core`: Shared note database logic (SQLite)
- `rusqlite`: SQLite with bundled driver (no system deps)
- `chrono`: Date/time parsing and formatting
- `toml`: Profile config and editor frontmatter parsing
- `reqwest`: HTTP client for server communication (blocking mode)
- `termcolor`: Colored terminal output
- `directories`: XDG directory paths

## Note Search Feature

The `jot note search` (or `jot ls`) command supports:
- Multiple output formats: `--output pretty|plain|json|id`
- Tag filtering: `-t work,urgent`
- Date filtering: `--date today|yesterday|2024-01-15`
- Content preview: `-L 3` (show first 3 lines)
- Result limiting: `-n 10` (max 10 results)
- Term matching: `jot ls "meeting notes"`

See `cli/docs/note-search.md` for detailed usage examples.
