# Jot - Terminal-First Note Taking

## What is Jot?

Jot is a fast, offline-first note-taking tool designed for developers who live in the terminal. Capture thoughts instantly from the command line, search your notes at lightning speed, and optionally sync across devices with your own self-hosted server.

## The Problem

Traditional note-taking apps (Obsidian, OneNote, Notion) introduce friction when you just want to capture a quick thought:

- **Too much overhead**: Where should I save this? What should I name it?
- **Context switching**: Terminal → GUI app → back to terminal
- **Vendor lock-in**: Your notes trapped in proprietary formats or cloud services
- **Slow sync**: Re-downloading entire databases just to add one note
- **Requires internet**: Can't work offline effectively

For developers who always have a terminal open, this friction kills productivity.

## The Solution

```bash
# Instant capture (works offline)
jot down "remember to check that API bug"

# Fast search
jot search "API bug"

# Optional sync (only sends changes)
jot sync
```

That's it. No naming files, no deciding where to save, no opening apps.

## Key Features

### 🚀 **Instant Capture**
- Type `jot down "your thought"` and you're done
- No naming, no navigation, no friction
- Works offline - internet not required

### ⚡ **Lightning Fast Search**
- Search thousands of notes in milliseconds
- Local SQLite database with full-text indexing
- Filter by tags, dates, keywords

### 🔄 **Smart Sync (Optional)**
- Self-hosted server for multi-device access
- **Incremental sync**: Only sends changed notes (not entire database)
- Works on terrible WiFi - syncs efficiently even on slow connections
- Smart conflict resolution (last-write-wins by timestamp)

### 🌐 **Web UI**
- Browse and search notes from any browser
- No CLI needed on mobile/tablet
- Same data, different interface

### 📝 **Flexible Organization**
- Tag notes: `--tag work,urgent`
- Date assignment: `--date today`, `--date 2025-01-15`
- Edit notes anytime (not append-only like journals)
- Export to Markdown for portability

### 🔒 **Own Your Data**
- Notes stored in local SQLite database
- Plain text accessible (export to Markdown anytime)
- Self-hosted server (no vendor lock-in)
- Open source - audit the code yourself

## How It Compares

| Feature | Jot | Obsidian Sync | Joplin | Dropbox + nb | Notion |
|---------|-----|---------------|--------|--------------|--------|
| **Offline-first** | ✅ | ✅ | ✅ | ✅ | ❌ |
| **Terminal native** | ✅ | ❌ | ❌ | ✅ | ❌ |
| **Incremental sync** | ✅ | ✅ | ✅ | ❌ | ✅ |
| **Web UI** | ✅ | ✅ | ✅ | ❌ | ✅ |
| **Self-hosted** | ✅ | ❌ | ✅ | ⚠️ | ❌ |
| **Fast (Rust)** | ✅ | ❌ | ❌ | ⚠️ | ❌ |
| **Free** | ✅ | ❌ ($8/mo) | ✅ | ✅ | ❌ (limited) |
| **Zero config** | ✅ | ❌ | ❌ | ❌ | ❌ |

### vs. Obsidian
- **Obsidian**: Feature-rich GUI, great for long-form writing, $8/month for sync
- **Jot**: Terminal-first, instant capture, free self-hosted sync, perfect for quick notes

### vs. Joplin
- **Joplin**: Desktop/mobile apps, requires their client, complex setup
- **Jot**: Simple CLI, works anywhere, web UI for mobile

### vs. nb / jrnl
- **nb/jrnl**: File-based, full database sync, no web UI
- **Jot**: Incremental sync (efficient), web UI, smart merging

### vs. Notion / Evernote
- **Notion/Evernote**: Cloud-only, vendor lock-in, requires internet
- **Jot**: Offline-first, self-hosted, own your data

## Use Cases

### Quick Capture
```bash
# During coding
jot down "that edge case with null values"

# Meeting notes
jot down "follow up with Alice about deployment" --tag work,urgent

# Ideas
jot down "blog post idea: async rust patterns" --tag blog
```

### Search & Retrieve
```bash
# Find that thing you wrote last week
jot search "edge case"

# Filter by tags
jot search --tag work

# Date-based
jot search --date today
```

### Multi-Device Workflow
```bash
# At desk: Quick note
jot down "check production logs tomorrow morning"
jot sync

# On phone later: Open web UI
# → See the note
# → Add: "also check error rates"

# Back at laptop:
jot sync  # Pulls phone changes (only ~1KB transferred)
jot search "production"  # Found it
```

### Offline Work
```bash
# On plane (no internet)
jot down "refactor auth module"
jot down "investigate memory leak"
jot search "auth"  # Works perfectly offline

# Back online
jot sync  # Pushes changes
```

## Philosophy

**Jot is built on three principles:**

1. **Instant capture**: No friction between thought and storage
2. **Offline-first**: Never blocked by connectivity
3. **Simplicity**: Does one thing well (quick notes)

We don't try to be:
- A task manager (use Todoist)
- A knowledge base (use Obsidian)
- A team wiki (use Notion)

We're the fastest way to capture and find your thoughts from the terminal.

## Getting Started

### Install CLI
```bash
cargo install jot-cli
```

### Basic Usage
```bash
# First note
jot down "my first note"

# With tags and date
jot down "work meeting notes" --tag work --date today

# Search
jot search "meeting"

# Edit a note
jot edit <note-id>
```

### Optional: Self-Hosted Sync

```bash
# Run server (Docker)
docker run -d -p 9000:9000 jot-server

# Configure CLI
jot init
# Enter server URL when prompted

# Login (OAuth-like device flow)
jot login

# Sync
jot sync
```

Now access web UI at `http://localhost:9000` to browse/search from browser.

## Roadmap

**Current (v0.1)**
- ✅ CLI with local SQLite storage
- ✅ Fast search and tagging
- ✅ Self-hosted server
- ✅ Device flow authentication
- ✅ Basic web UI

**Planned (v0.2)**
- [ ] Incremental sync implementation
- [ ] Rich web UI with editor
- [ ] Export/import to Markdown
- [ ] Note templates

**Future**
- [ ] End-to-end encryption
- [ ] Note sharing (public links)
- [ ] Mobile-optimized web UI
- [ ] Plugin system

## Why "Jot"?

Because that's what you do - you **jot** things down quickly. No ceremony, no overhead, just capture and move on.

## License

MIT - Own your code, own your notes.

## Links

- GitHub: [github.com/your-repo/jot]
- Documentation: [jot-docs.example.com]
- Self-hosting guide: [docs/SELF_HOSTING.md]
