# Jot - Technical Architecture

## Overview

Jot is a terminal-first note-taking application built with Rust, consisting of two main components:

1. **CLI** (`jot-cli`): Offline-first command-line client with local SQLite storage
2. **Server** (`jot-server`): Self-hosted sync and web UI server using Axum and SQLite

Both components share a common core library (`jot-core`) containing database operations and data models.

## Architecture Principles

### 1. Offline-First
- CLI works completely offline with local SQLite database
- Server is optional - adds sync and web UI capabilities
- Local database is source of truth
- Server acts as backup/sync layer

### 2. Per-User Database Isolation
- Server maintains separate SQLite database per user: `data/users/{user_id}.db`
- No multi-tenant complexity in queries
- Perfect isolation between users
- Simplifies permissions and backup

### 3. Incremental Sync
- Only changed notes are transmitted during sync
- Timestamp-based change detection (`updated_at` column)
- Efficient on slow/mobile connections
- Smart conflict resolution (last-write-wins)

### 4. Simplicity Over Features
- Single-user focused (no collaboration features)
- Last-write-wins conflict resolution (no CRDTs)
- Notes are editable (not append-only journal)
- No real-time sync (manual `jot sync` command)

## Data Model

### Database Schema

**Shared schema used by both CLI and Server:**

```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,              -- ULID (sortable, globally unique)
    content TEXT NOT NULL,            -- Note content (plain text/markdown)
    tags TEXT,                        -- JSON array: ["work", "urgent"]
    date TEXT,                        -- ISO 8601: "2025-01-16"
    created_at INTEGER NOT NULL,      -- Unix timestamp (milliseconds)
    updated_at INTEGER NOT NULL,      -- Unix timestamp (milliseconds)
    deleted_at INTEGER                -- NULL (active) or Unix timestamp (soft-deleted)
);

CREATE INDEX idx_updated_at ON notes(updated_at);
CREATE INDEX idx_deleted_at ON notes(deleted_at);
CREATE INDEX idx_tags ON notes(tags);  -- JSON search optimization

CREATE TABLE sync_state (
    key TEXT PRIMARY KEY,
    value TEXT
);
-- Stores: last_sync_timestamp, device_id, etc.

PRAGMA user_version = 1;  -- Schema version for migrations
```

### Why ULID Instead of UUID?

**ULID (Universally Unique Lexicographically Sortable Identifier):**
- Sortable by creation time (unlike UUID v4)
- 128-bit (same as UUID)
- Collision-resistant
- Generated locally without server coordination
- Better for "show last N notes" queries

Example ULID: `01ARZ3NDEKTSV4RRFFQ69G5FAV`

### Why Soft Deletes?

```sql
deleted_at INTEGER  -- NULL or timestamp
```

**Problem without soft deletes:**
```
Device A: Deletes note (removes from DB)
Device B: Still has note
Sync: Note reappears on Device A (B thinks it's new)
```

**Solution with soft deletes:**
- DELETE operation sets `deleted_at = current_timestamp`
- Sync includes tombstones
- Notes can be permanently pruned after N days (future feature)

### Tags as JSON

```sql
tags TEXT  -- '["work", "urgent", "bug"]'
```

**Why JSON string instead of separate table:**
- Simplicity (no joins needed)
- SQLite has JSON functions: `json_each()`, `json_extract()`
- Average note has 0-3 tags (no performance issues)
- Can migrate to normalized table later if needed

**Query example:**
```sql
-- Find notes with 'work' tag
SELECT * FROM notes
WHERE tags LIKE '%"work"%'
AND deleted_at IS NULL;

-- Or using JSON functions
SELECT * FROM notes
WHERE EXISTS (
    SELECT 1 FROM json_each(notes.tags)
    WHERE value = 'work'
);
```

## Component Architecture

### Project Structure

```
jot/
├── cli/                  # CLI binary
│   ├── src/
│   │   ├── main.rs      # Entry point, arg parsing
│   │   ├── commands/    # Command implementations
│   │   │   ├── add.rs
│   │   │   ├── search.rs
│   │   │   ├── sync.rs
│   │   │   └── edit.rs
│   │   ├── config.rs    # Profile management
│   │   └── formatters.rs
│   └── Cargo.toml
├── server/               # Server binary
│   ├── src/
│   │   ├── main.rs      # Server startup
│   │   ├── router/      # HTTP routes
│   │   │   ├── auth.rs
│   │   │   ├── sync.rs
│   │   │   └── web_ui.rs
│   │   ├── middleware.rs
│   │   ├── jwt.rs
│   │   └── state.rs
│   └── Cargo.toml
├── core/                 # Shared library
│   ├── src/
│   │   ├── lib.rs
│   │   ├── db.rs        # Database operations
│   │   ├── models.rs    # Data structures
│   │   └── schema.rs    # SQL migrations
│   └── Cargo.toml
└── docs/
    ├── PRODUCT_OVERVIEW.md
    └── TECHNICAL_ARCHITECTURE.md
```

### Core Library (`jot-core`)

**Responsibilities:**
- Database schema and migrations
- CRUD operations for notes
- Sync logic (merge functions)
- Data models and serialization

**Key modules:**

```rust
// core/src/db.rs
pub fn open_db(path: &Path) -> Result<Connection>;
pub fn create_note(db: &Connection, content: &str, tags: Vec<String>) -> Result<Note>;
pub fn search_notes(db: &Connection, query: &SearchQuery) -> Result<Vec<Note>>;
pub fn update_note(db: &Connection, id: &str, content: &str) -> Result<()>;
pub fn soft_delete_note(db: &Connection, id: &str) -> Result<()>;
pub fn get_notes_since(db: &Connection, timestamp: i64) -> Result<Vec<Note>>;

// core/src/models.rs
pub struct Note {
    pub id: String,           // ULID
    pub content: String,
    pub tags: Vec<String>,
    pub date: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

pub struct SearchQuery {
    pub text: Option<String>,
    pub tags: Vec<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

pub struct SyncRequest {
    pub notes: Vec<Note>,
    pub last_sync: i64,  // Client's last sync timestamp
}

pub struct SyncResponse {
    pub notes: Vec<Note>,  // Server's newer notes
    pub conflicts: Vec<Conflict>,
}
```

### CLI Architecture

**Lifecycle:**

```rust
// main.rs
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Parse arguments
    let args = CliArgs::parse();

    // 2. Load profile (optional)
    let profile = Profile::from_path(&args.profile)?;

    // 3. Open local database
    let db_path = profile.db_path.unwrap_or("~/.jot/notes.db");
    let db = open_db(&db_path)?;

    // 4. Execute command
    match args.command {
        Command::Down(text) => commands::add::execute(&db, text)?,
        Command::Search(query) => commands::search::execute(&db, query)?,
        Command::Sync => commands::sync::execute(&db, &profile).await?,
        Command::Edit(id) => commands::edit::execute(&db, id)?,
    }

    Ok(())
}
```

**Connection management:**
- Single connection opened at startup
- Shared across all operations
- Closed when CLI exits
- No pooling needed (single-user, short-lived process)

**Profile system:**
```toml
# ~/.jot/profiles/default.toml
[local]
db_path = "~/.jot/notes.db"

[server]
url = "https://jot.example.com"
api_key_path = "~/.jot/api_key"
```

Multiple profiles allow switching between servers or local-only mode.

### Server Architecture

**Tech stack:**
- **Axum**: HTTP framework
- **SQLx**: Async SQLite driver
- **JWT**: Authentication tokens
- **Tokio**: Async runtime

**Request flow:**

```
HTTP Request
    ↓
Auth Middleware (extracts user from JWT)
    ↓
Route Handler (gets user_id)
    ↓
Open user DB: users/{user_id}.db
    ↓
Execute operation (using jot-core)
    ↓
Close DB
    ↓
HTTP Response
```

**Key endpoints:**

```rust
// Auth
POST   /auth/device         // Start device flow
POST   /auth/device/poll    // Poll for token
GET    /auth/page/{code}    // Web page for user to approve

// Sync
POST   /sync                // Sync notes (incremental)

// Web UI
GET    /                    // Web interface
GET    /api/notes           // List notes (for web UI)
GET    /api/notes/:id       // Get single note
POST   /api/notes           // Create note (from web)
PUT    /api/notes/:id       // Update note (from web)
DELETE /api/notes/:id       // Delete note (from web)
```

**Per-user database handling:**

```rust
pub async fn sync_notes(
    State(state): State<AppState>,
    user: AuthenticatedUser,  // From middleware
    Json(req): Json<SyncRequest>,
) -> Result<Json<SyncResponse>> {
    // Open user's database
    let db_path = format!("{}/users/{}.db", state.data_dir, user.id);
    let db = open_or_create_db(&db_path)?;

    // Merge notes using core library
    let merged = jot_core::sync::merge_notes(
        &db,
        req.notes,
        req.last_sync
    )?;

    Ok(Json(SyncResponse { notes: merged }))
    // db closes automatically (RAII)
}
```

**Why per-request DB open/close is fine:**
- Sync is infrequent (minutes/hours, not seconds)
- SQLite file open is fast (~1ms with OS page cache)
- No idle connections consuming memory
- Simple = reliable

**If optimization needed later:**
- Add LRU cache of N most recent user connections
- Monitor with metrics before optimizing

## Sync Protocol

### Incremental Sync Algorithm

**Goal:** Only send changed notes, not entire database

**Client-side:**
```rust
async fn sync() -> Result<()> {
    let db = open_db()?;

    // 1. Get last sync timestamp from local state
    let last_sync = get_last_sync_timestamp(&db)?;

    // 2. Get all notes changed since last sync
    let changed_notes = get_notes_since(&db, last_sync)?;

    // 3. Send to server
    let response: SyncResponse = client
        .post("/sync")
        .json(&SyncRequest {
            notes: changed_notes,
            last_sync,
        })
        .send()
        .await?
        .json()
        .await?;

    // 4. Apply server's newer notes
    for note in response.notes {
        upsert_note(&db, note)?;
    }

    // 5. Update last sync timestamp
    set_last_sync_timestamp(&db, now())?;

    Ok(())
}
```

**Server-side:**
```rust
fn merge_notes(
    db: &Connection,
    client_notes: Vec<Note>,
    client_last_sync: i64,
) -> Result<Vec<Note>> {
    let mut to_send = Vec::new();

    // 1. Process each incoming note
    for client_note in client_notes {
        let server_note = get_note_by_id(db, &client_note.id)?;

        match server_note {
            None => {
                // New note from client
                insert_note(db, &client_note)?;
            }
            Some(server_note) => {
                // Conflict: Last-write-wins
                if client_note.updated_at > server_note.updated_at {
                    update_note(db, &client_note)?;
                } else if server_note.updated_at > client_note.updated_at {
                    to_send.push(server_note);
                }
                // If equal timestamps, no action needed
            }
        }
    }

    // 2. Find server notes newer than client's last sync
    let server_new = get_notes_since(db, client_last_sync)?;
    to_send.extend(server_new);

    Ok(to_send)
}
```

### Conflict Resolution

**Strategy: Last-Write-Wins (LWW)**

```
Device A: Edit note at 14:00:00
Device B: Edit same note at 14:00:05

Winner: Device B (newer timestamp)
```

**Why not manual conflict resolution?**
- Adds significant UX complexity
- Rare in single-user scenarios
- Document the limitation clearly
- Can add later if users demand it

**Limitations:**
- Clock skew can cause issues (mitigated by NTP)
- No merge of concurrent edits (later writer wins)
- Acceptable for quick notes use case

**Future enhancement: Vector clocks**
- Track logical time instead of wall-clock time
- Detect true conflicts
- For now: YAGNI (You Aren't Gonna Need It)

## Authentication

### Device Flow (OAuth 2.0 Device Authorization Grant)

**Why device flow?**
- Great UX for CLI tools (no embedded browser)
- User authorizes in their default browser
- CLI polls for completion
- Same flow as `gh` (GitHub CLI), `az` (Azure CLI)

**Flow:**

```
CLI                         Server                      Browser
 |                            |                            |
 |-- POST /auth/device ------>|                            |
 |                            |                            |
 |<---- device_code ----------|                            |
 |     user_code              |                            |
 |     verification_uri       |                            |
 |                            |                            |
 |-- Open browser ----------->|                            |
 |    (verification_uri)      |                            |
 |                            |                            |
 |                            |<-- GET /auth/page/{code} --|
 |                            |--- Login form ------------->|
 |                            |<-- User approves -----------|
 |                            |--- Store approval ---------|
 |                            |                            |
 |-- Poll: POST /auth/poll -->|                            |
 |<---- pending --------------|                            |
 |                            |                            |
 |-- Poll: POST /auth/poll -->|                            |
 |<---- JWT token ------------|                            |
 |                            |                            |
```

**Database:**
```sql
-- Server only
CREATE TABLE device_auth (
    device_code TEXT PRIMARY KEY,
    user_code TEXT UNIQUE,
    user_id TEXT,              -- NULL until approved
    expires_at INTEGER,
    created_at INTEGER
);

CREATE TABLE users (
    id TEXT PRIMARY KEY,       -- ULID
    username TEXT UNIQUE,
    password_hash TEXT,
    created_at INTEGER
);
```

**JWT payload:**
```json
{
  "sub": "user_id",
  "exp": 1234567890,
  "iat": 1234567890
}
```

**Storage:**
```bash
# CLI stores token
~/.jot/api_key  # Contains JWT (chmod 600 on Unix)
```

## Testing Strategy

### Unit Tests

**Core library:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_note() {
        let dir = TempDir::new().unwrap();
        let db = open_db(&dir.path().join("test.db")).unwrap();

        let note = create_note(&db, "test", vec!["tag1"]).unwrap();

        assert_eq!(note.content, "test");
        assert_eq!(note.tags, vec!["tag1"]);
    }

    #[test]
    fn test_soft_delete() {
        let dir = TempDir::new().unwrap();
        let db = open_db(&dir.path().join("test.db")).unwrap();

        let note = create_note(&db, "test", vec![]).unwrap();
        soft_delete_note(&db, &note.id).unwrap();

        let found = get_note_by_id(&db, &note.id).unwrap();
        assert!(found.deleted_at.is_some());
    }
}
```

**No mocking needed:**
- Create temp SQLite database
- Run real operations
- Fast (SQLite in-memory or tmpfs)

### Integration Tests (CLI)

**Before (problematic):**
```rust
// E2E test that compiles binary and runs it
#[test]
fn test_login() {
    Command::cargo_bin("jot")
        .arg("login")
        .assert()
        .success();
    // Problem: Opens real browser, needs server running
}
```

**After (clean):**
```rust
#[test]
fn test_add_note() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("notes.db");

    Command::cargo_bin("jot")
        .env("JOT_DB", db_path)
        .arg("down")
        .arg("test note")
        .assert()
        .success();

    // Verify in database
    let db = open_db(&db_path).unwrap();
    let notes = get_all_notes(&db).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "test note");
}
```

**Key insight:**
- Test against real SQLite
- No HTTP mocking
- No browser automation
- Fast, reliable, simple

### Server Tests

```rust
use axum_test::TestServer;

#[tokio::test]
async fn test_sync_endpoint() {
    let server = TestServer::new(app).unwrap();

    // Create user and get token
    let token = create_test_user_and_login(&server).await;

    // Sync notes
    let response = server
        .post("/sync")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&SyncRequest {
            notes: vec![test_note()],
            last_sync: 0,
        })
        .await;

    response.assert_status_ok();

    // Verify in user's database
    let db = open_user_db("test_user").unwrap();
    let notes = get_all_notes(&db).unwrap();
    assert_eq!(notes.len(), 1);
}
```

## Performance Considerations

### Local Search Performance

**Scenario:** 50,000 notes (~20MB database)

**Query:**
```sql
SELECT * FROM notes
WHERE content LIKE '%keyword%'
  AND deleted_at IS NULL
LIMIT 100;
```

**Performance:**
- SQLite full-text search: ~10ms
- Sequential scan: ~50ms (acceptable)
- Can add FTS5 virtual table later if needed

**For now:** Simple `LIKE` queries are fast enough

### Sync Performance

**Scenario:** First sync with 10,000 notes

**Without incremental sync:**
```
Download entire 20MB database
Time: ~30 seconds on slow mobile
```

**With incremental sync:**
```
Client has 0 notes
Server sends all 10,000 (first sync exception)
Subsequent syncs: Only changed notes
```

**Optimization: Progressive first sync**
```rust
// Future enhancement
async fn first_sync() {
    // 1. Sync newest 1000 notes first (usable immediately)
    sync_recent(1000).await?;

    // 2. Background: Sync remaining in batches
    tokio::spawn(async {
        sync_remaining().await
    });
}
```

### Server Scalability

**Current architecture scales to:**
- ~100 concurrent users (single server)
- ~10,000 users total (disk space limited)
- ~100,000 notes per user

**Bottlenecks:**
- Disk I/O (mitigated by OS page cache)
- File descriptor limits (can tune OS)

**When to scale:**
- Multiple servers: Shard users by hash(user_id)
- Shared database: Migrate to PostgreSQL with partitioning
- CDN: Cache static web UI assets

**For now:** Single server is plenty

## Deployment

### Server Deployment

**Docker:**
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p jot-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libsqlite3-0 ca-certificates
COPY --from=builder /app/target/release/jot-server /usr/local/bin/
COPY --from=builder /app/server/static /static

ENV JOT_HOST=0.0.0.0
ENV JOT_PORT=9000
ENV JOT_DATA_DIR=/data
ENV JOT_JWT_SECRET=CHANGE_ME

EXPOSE 9000
VOLUME ["/data"]

CMD ["jot-server"]
```

**docker-compose.yml:**
```yaml
version: '3.8'
services:
  jot:
    build: .
    ports:
      - "9000:9000"
    volumes:
      - ./data:/data
    environment:
      - JOT_JWT_SECRET=${JOT_JWT_SECRET}
    restart: unless-stopped
```

**Reverse proxy (nginx):**
```nginx
server {
    listen 443 ssl http2;
    server_name jot.example.com;

    ssl_certificate /etc/letsencrypt/live/jot.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/jot.example.com/privkey.pem;

    location / {
        proxy_pass http://localhost:9000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### CLI Installation

**Cargo:**
```bash
cargo install jot-cli
```

**Binary releases:**
```
GitHub Releases:
- jot-cli-linux-x86_64.tar.gz
- jot-cli-macos-arm64.tar.gz
- jot-cli-windows-x86_64.zip
```

**First-time setup:**
```bash
# Local only
jot down "my first note"

# With server
jot init  # Creates profile
jot login  # Device flow
jot sync   # First sync
```

## Security Considerations

### Authentication
- JWT tokens with expiration
- Secure token storage (chmod 600 on Unix)
- Device flow prevents token interception
- HTTPS required for production

### Data Protection
- SQLite databases chmod 600 (Unix)
- No encryption at rest (future: add option)
- Server validates all user input
- SQL injection: Prevented by parameterized queries (SQLx)

### Server Security
- Rate limiting on auth endpoints
- JWT secret from environment (not hardcoded)
- Per-user DB isolation (can't access other users' notes)
- Input validation on all endpoints

## Migration Strategy

### Schema Versioning

```sql
PRAGMA user_version = 1;
```

**Migration path:**
```rust
fn migrate_db(db: &Connection) -> Result<()> {
    let version: i32 = db.pragma_query_value(None, "user_version", |row| row.get(0))?;

    match version {
        0 => {
            // Initial migration
            db.execute_batch(include_str!("schema_v1.sql"))?;
            db.pragma_update(None, "user_version", &1)?;
        }
        1 => {
            // Already up to date
        }
        _ => {
            bail!("Unknown schema version: {}", version);
        }
    }

    Ok(())
}
```

**Adding new fields (future):**
```sql
-- Migration v1 -> v2
ALTER TABLE notes ADD COLUMN archived INTEGER DEFAULT 0;
PRAGMA user_version = 2;
```

### Breaking Changes

**If schema changes are incompatible:**
1. Increment major version
2. Provide export tool: `jot export --format md`
3. User exports notes
4. Reinstalls new version
5. Import: `jot import ~/notes-backup/`

**Goal:** Avoid breaking changes through careful design now

## Future Enhancements

### Phase 1 (v0.2) - Incremental Sync
- [ ] Implement sync protocol
- [ ] Web UI for note browsing
- [ ] Export to Markdown
- [ ] Import from Markdown

### Phase 2 (v0.3) - Rich Features
- [ ] Full-text search (FTS5)
- [ ] Note templates
- [ ] Bulk operations
- [ ] Note archiving

### Phase 3 (v0.4) - Advanced
- [ ] End-to-end encryption (optional)
- [ ] Public note sharing (generate link)
- [ ] Collaborative editing (operational transforms)
- [ ] Plugin system (Lua scripting?)

### Not Planned
- ❌ Mobile native apps (web UI sufficient)
- ❌ Real-time sync (manual is fine)
- ❌ Team features (single-user focus)
- ❌ Rich text editor (plain text is the goal)

## Development Workflow

### Setup
```bash
# Clone
git clone https://github.com/your-repo/jot
cd jot

# Install dependencies
rustup update stable

# Run tests
cargo test --workspace

# Run CLI locally
cargo run -p jot-cli -- down "test note"

# Run server locally
cd server
cp .env.example .env
# Edit .env with your values
cargo run

# Open http://localhost:9000
```

### Code Style
- `cargo fmt` before commit
- `cargo clippy` must pass
- Deny unwrap/expect in production code
- Add tests for new features

### Release Process
1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Tag: `git tag v0.1.0`
4. Push: `git push --tags`
5. GitHub Actions builds binaries
6. Publish to crates.io: `cargo publish`

## Conclusion

Jot's architecture prioritizes:
1. **Simplicity**: SQLite everywhere, no complex sync protocols
2. **Reliability**: Offline-first, local data is truth
3. **Performance**: Incremental sync, fast local search
4. **Maintainability**: Shared core library, clear separation

The per-user database model eliminates entire classes of bugs while keeping the codebase simple. The offline-first approach means users are never blocked by connectivity issues.

This architecture can scale to thousands of users on modest hardware while remaining easy to understand and maintain.

## References

- SQLite: https://sqlite.org/
- ULID: https://github.com/ulid/spec
- OAuth Device Flow: https://oauth.net/2/device-flow/
- Axum: https://docs.rs/axum/
- SQLx: https://docs.rs/sqlx/
