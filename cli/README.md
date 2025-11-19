# Jot CLI

**Quick, frictionless note-taking from your terminal.**

Jot is a terminal-first note-taking application designed for developers who want to capture thoughts instantly without the overhead of traditional note apps. No deciding where to save, no naming files—just jot it down.

## Why Jot?

Traditional note apps (Obsidian, OneNote, etc.) require too much thinking when you just want to capture a quick thought:
- Where should I save this?
- What should I name it?
- Which folder does it belong in?

Since your terminal is always open, Jot provides instant note capture from the command line:

```bash
jot down "bug in auth flow - check token expiration"
```

That's it. Your note is saved, timestamped, and searchable.

## Features

- **Instant capture**: Quick one-liners or open your editor for longer notes
- **Profile system**: Separate note databases for work, personal, projects
- **Tagging**: Organize with tags, filter by tag when searching
- **Date assignment**: Automatically organize notes chronologically
- **Powerful search**: Filter by tags, dates, content with multiple output formats
- **Editor integration**: Opens `$EDITOR` with TOML frontmatter for structured notes
- **Shell completions**: Built-in support for bash, zsh, fish, powershell, elvish
- **Scripting-friendly**: Quiet mode and ID-only output for pipeline integration
- **XDG-compliant**: Respects standard Linux directory conventions

## Installation

### From source

```bash
# Build release binary
cargo build --release --package jot-cli

# Binary will be at: target/release/jot-cli
# Move to your PATH:
sudo mv target/release/jot-cli /usr/local/bin/jot
```

### Shell completions

```bash
# Bash
jot completion bash > /etc/bash_completion.d/jot

# Zsh
jot completion zsh > ~/.zsh/completion/_jot

# Fish
jot completion fish > ~/.config/fish/completions/jot.fish
```

## Quick Start

### Basic note capture

```bash
# Quick one-liner
jot down "meeting at 3pm tomorrow"

# With tags
jot note add -t work,urgent "production deployment tonight"

# Open your editor for a longer note
jot note add -e

# Assign a date
jot down "quarterly review notes" -d today
```

### Searching notes

```bash
# List all notes (alias for 'note search')
jot ls

# Search by content
jot ls "meeting"

# Filter by tags
jot ls -t work,urgent

# Combine filters and limit results
jot note search "bug" -t backend -n 5

# Show content preview (first 3 lines)
jot ls -L 3
```

### Get the latest note

```bash
# Display the most recent note
jot note last

# Alternative command (same thing)
jot note latest
```

### Managing notes

```bash
# Edit a note (opens in $EDITOR)
jot note edit <note-id>

# Delete a note
jot note delete <note-id>
```

### Profiles

Switch between different note databases for different contexts:

```bash
# Show current profile
jot profile

# Switch to work profile (creates if doesn't exist)
jot profile use work

# List all profiles
jot profile list

# Use a specific profile for one command
jot -p personal down "buy groceries"
```

Profiles can have default tags in their config files (`~/.config/jot/profiles/<name>.toml`):

```toml
default_tags = ["work", "backend"]
```

## Output Formats

Jot supports multiple output formats for different use cases:

```bash
# Pretty (default): Colored, human-readable
jot ls

# Plain: No colors, for logging/piping
jot ls --output plain

# JSON: Machine-readable structured data
jot ls --output json

# ID: Just note IDs, one per line (for scripting)
jot ls --output id
```

## Scripting & Automation

### Quiet mode

When creating notes in scripts, use `-q` to output only the note ID:

```bash
NOTE_ID=$(jot note add -q "automated backup completed")
echo "Created note: $NOTE_ID"
```

### Pipeline integration

Get IDs and pipe to other commands:

```bash
# Delete all notes tagged 'temp'
jot ls -t temp --output id | while read id; do
  jot note delete "$id"
done

# Count notes by tag
jot ls -t work --output id | wc -l
```

## Editor Integration

When you use `-e` to open your editor, Jot creates a template with TOML frontmatter:

```markdown
tags = []
date = "2025-11-19"

+++

# Your note content goes here
```

Edit the tags and date in the frontmatter, write your content below the `+++` delimiter, then save and exit.

## Command Reference

### Commands

- `jot down <content>` - Quick note capture (alias for `note add`)
- `jot ls [term]` - List/search notes (alias for `note search`)
- `jot note add` - Create a new note
- `jot note search` - Search and filter notes
- `jot note last` - Show the most recent note
- `jot note edit <id>` - Edit an existing note
- `jot note delete <id>` - Delete a note
- `jot profile` - Show current profile (alias for `profile current`)
- `jot profile use <name>` - Switch to a profile
- `jot profile list` - List all profiles
- `jot config` - Display current configuration
- `jot completion <shell>` - Generate shell completions

### Common Flags

- `-t, --tags <tags>` - Comma-separated tags
- `-e, --editor` - Open editor for note content
- `-d, --date <date>` - Assign a date (today, yesterday, YYYY-MM-DD)
- `-n, --limit <n>` - Limit number of results
- `-L, --lines <n>` - Show first N lines of content
- `-q, --quiet` - Quiet mode (output only IDs)
- `-p, --profile <name>` - Use a specific profile
- `--output <format>` - Output format (pretty, plain, json, id)

## Configuration

### Directory structure

Jot follows XDG Base Directory conventions:

```
~/.config/jot/
├── current                      # Active profile name
└── profiles/
    ├── default.toml            # Profile configurations
    ├── work.toml
    └── personal.toml

~/.local/share/jot/
└── profiles/
    ├── default/notes.db        # SQLite databases (one per profile)
    ├── work/notes.db
    └── personal/notes.db
```

### Environment variables

- `JOT_PROFILE` - Override current profile
- `EDITOR` or `VISUAL` - Editor to use for `-e` flag
- `XDG_CONFIG_HOME` - Config directory (defaults to `~/.config`)
- `XDG_DATA_HOME` - Data directory (defaults to `~/.local/share`)

### Profile configuration

Profile TOML files can set default tags:

```toml
# ~/.config/jot/profiles/work.toml
default_tags = ["work"]
```

These tags are automatically applied to all notes in that profile (unless overridden with `-t`).

## Examples

### Daily standup notes

```bash
# Switch to work profile
jot profile use work

# Quick standup note with tags
jot down "Completed API refactor, working on auth tests today" \
  -t standup,backend -d today
```

### Meeting notes with editor

```bash
# Open editor for detailed notes
jot note add -e -t meeting,planning

# In your editor, you'll see:
# tags = ["meeting", "planning"]
# date = "2025-11-19"
#
# +++
#
# [write your notes here]
```

### Search and review

```bash
# Find all meeting notes from last week
jot ls -t meeting --date 2025-11-12

# Get last 10 backend notes with 5-line previews
jot ls -t backend -n 10 -L 5

# Export all work notes as JSON
jot ls -t work --output json > work-notes.json
```

### Scripting example

```bash
#!/bin/bash
# Create a daily log entry

DATE=$(date +%Y-%m-%d)
CONTENT="Daily log for $DATE"

# Create note and get ID
NOTE_ID=$(jot down "$CONTENT" -t log -d today -q)

# Open in editor to add details
jot note edit "$NOTE_ID"
```

## Development

This is part of the Jot workspace. See the main repository README for development setup.

### Running tests

```bash
# Run CLI tests
cargo test --package jot-cli

# Run with output
cargo test --package jot-cli -- --nocapture
```

### Running from source

```bash
# From workspace root
cargo run --bin jot-cli -- <command>

# Example
cargo run --bin jot-cli -- down "test note"
```

## Version

Current version: **0.2.0**

## License

See the main repository for license information.
