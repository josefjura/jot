# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Jot is a terminal-first note-taking application designed for quick, frictionless note capture. The core philosophy is that opening traditional note apps (Obsidian, OneNote) involves too much overhead (deciding where to save, how to name) when you just want to jot something down quickly. Since terminals are always open for developers, Jot provides instant note capture from the command line.

The application consists of two Rust components:
- **CLI** (`cli/`): Command-line interface for creating, searching, and managing notes
- **Server** (`server/`): REST API backend using Axum framework with SQLite database, designed to run as a Docker container for self-hosted, centralized note storage

### Key Features & Design Goals

- **Instant capture**: `jot note add "my quick thought"` or `jot down "quick thought"` - no naming, no navigation required
- **Date assignment**: `--date today` (or other date expressions) automatically organizes notes chronologically
- **Tag support**: `--tag work,important` for organizing and filtering notes
- **Editor integration**: `--edit` flag opens `$EDITOR` (or `$VISUAL`) for longer-form note editing with TOML frontmatter template
- **Templating**: Fully implemented - notes opened in editor have TOML frontmatter (tags, date) separated from content by `+++` delimiter
- **Search & filtering**: Flexible search with `jot note search` supporting term matching, tag filters, date filters, and multiple output formats (pretty/plain/JSON)
- **Device-based authentication**: OAuth-like device flow - CLI generates code, opens browser for login, polls for token
- **Self-hosted**: Server designed for deployment (Docker support not yet implemented)
- **Quality focus**: Emphasis on comprehensive testing and clean code despite being a learning project (third Rust project)

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
# Run CLI
cd cli
cargo run -- <command>

# Run CLI tests
cd cli
cargo test

# Build CLI
cd cli
cargo build --release
```

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

- **`main.rs`**: Entry point, parses args, loads profile, creates web client, dispatches to commands
- **`args/`**: Command-line argument definitions using Clap derive API
- **`commands/`**: Command implementations
  - `init`: Initialize new profile
  - `login`: Authenticate with server (device flow)
  - `note`: Note operations (add, search, last)
  - `config`: Display current configuration
- **`web_client/`**: HTTP client abstraction with mock implementation for testing
- **`profile.rs`**: Profile management (TOML configuration files)
- **`app_config.rs`**: Application configuration merging CLI args, profile, and defaults
- **`auth.rs`**: Authentication flow handling
- **`editor.rs`**: External editor integration
- **`formatters.rs`**: Output formatting (pretty, plain, JSON)
- **`utils/`**: Date parsing utilities (`DateSource`, `DateTarget`)

Configuration precedence: CLI flags > Profile file > Defaults

### Testing

**Server tests** (`server/src/test/`):
- Uses `axum-test` crate to create test servers
- `setup_server()` helper creates TestServer with in-memory database
- `login()` helper performs authentication and returns JWT token
- Tests organized by domain (auth, health, note)

**CLI tests** (`cli/src/test/`):
- Uses `assert_cmd` for integration testing of CLI binary
- `test_context.rs`: Test utilities including mock server setup
- Tests verify command-line argument parsing, profile loading, and end-to-end flows

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

## Note Search Feature

The `jot note search` command supports multiple output formats (pretty, plain, JSON), filtering by tags and dates, and controlling content display with `--lines`. See `cli/docs/note-search.md` for detailed usage examples.
