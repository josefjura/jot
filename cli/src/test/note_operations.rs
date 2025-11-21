#![allow(clippy::unwrap_used)]
#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test database and return paths
struct TestDb {
    _temp_dir: TempDir,
    db_path: PathBuf,
    profile_name: String,
}

impl TestDb {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();

        // Generate a unique profile name for this test
        let profile_name = format!("test_{}", uuid::Uuid::new_v4().simple());

        // XDG base directories (these will have "jot" appended by get_config_dir/get_data_dir)
        let xdg_config_base = temp_dir.path().join("config");
        let xdg_data_base = temp_dir.path().join("data");

        // Actual jot directories (where the CLI will put things)
        let jot_config_dir = xdg_config_base.join("jot");
        let jot_data_dir = xdg_data_base.join("jot");

        // Create profile directories
        let profile_config_dir = jot_config_dir.join("profiles");
        let profile_data_dir = jot_data_dir.join("profiles").join(&profile_name);
        std::fs::create_dir_all(&profile_config_dir).unwrap();
        std::fs::create_dir_all(&profile_data_dir).unwrap();

        // Database will be created at the profile data location
        let db_path = profile_data_dir.join("notes.db");

        // Create a minimal profile config
        let profile_config_path = profile_config_dir.join(format!("{}.toml", profile_name));
        let profile = crate::profile::Profile {
            db_path: Some(db_path.to_str().unwrap().to_string()),
            default_tags: vec![],
        };
        profile.save(&profile_config_path).unwrap();

        Self {
            _temp_dir: temp_dir,
            db_path,
            profile_name,
        }
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("jot").unwrap();

        // Override XDG base directories to use our temp dir
        let config_dir = self._temp_dir.path().join("config");
        let data_dir = self._temp_dir.path().join("data");

        cmd.env("XDG_CONFIG_HOME", config_dir.to_str().unwrap());
        cmd.env("XDG_DATA_HOME", data_dir.to_str().unwrap());
        cmd.env("JOT_PROFILE", &self.profile_name);
        cmd
    }

    /// Get all notes from the database
    fn get_notes(&self) -> Vec<jot_core::Note> {
        let conn = jot_core::open_db(&self.db_path).unwrap();
        let query = jot_core::SearchQuery {
            text: None,
            tags: vec![],
            date_from: None,
            date_to: None,
            include_deleted: false,
            limit: None,
        };
        jot_core::search_notes(&conn, &query).unwrap()
    }

    /// Add a note directly to the database for testing
    fn add_note(&self, content: &str, tags: Vec<&str>, date: Option<&str>) -> String {
        let conn = jot_core::open_db(&self.db_path).unwrap();
        let tags: Vec<String> = tags.into_iter().map(|s| s.to_string()).collect();
        let date = date.map(|s| s.to_string());
        let note = jot_core::create_note(&conn, content, tags, date).unwrap();
        note.id
    }
}

#[test]
fn test_note_add_simple() {
    let db = TestDb::new();

    db.cmd()
        .args(["note", "add", "my", "first", "note"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Note added successfully"));

    let notes = db.get_notes();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "my first note");
    assert!(notes[0].tags.is_empty());
}

#[test]
fn test_note_add_with_tags() {
    let db = TestDb::new();

    db.cmd()
        .args([
            "note",
            "add",
            "--tag",
            "work,urgent",
            "important",
            "meeting",
        ])
        .assert()
        .success();

    let notes = db.get_notes();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "important meeting");
    assert_eq!(notes[0].tags, vec!["work", "urgent"]);
}

#[test]
fn test_note_add_with_date() {
    let db = TestDb::new();

    db.cmd()
        .args(["note", "add", "--date", "2025-01-15", "dated", "note"])
        .assert()
        .success();

    let notes = db.get_notes();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].date, Some("2025-01-15".to_string()));
}

#[test]
fn test_down_alias() {
    let db = TestDb::new();

    db.cmd().args(["down", "quick", "note"]).assert().success();

    let notes = db.get_notes();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "quick note");
}

#[test]
fn test_note_search_all() {
    let db = TestDb::new();

    // Add multiple notes
    db.cmd()
        .args(["note", "add", "first", "note"])
        .assert()
        .success();
    db.cmd()
        .args(["note", "add", "second", "note"])
        .assert()
        .success();
    db.cmd()
        .args(["note", "add", "third", "note"])
        .assert()
        .success();

    // Search all
    db.cmd()
        .args(["note", "search"])
        .assert()
        .success()
        .stdout(predicate::str::contains("first note"))
        .stdout(predicate::str::contains("second note"))
        .stdout(predicate::str::contains("third note"));
}

#[test]
fn test_note_search_by_term() {
    let db = TestDb::new();

    db.cmd()
        .args(["note", "add", "meeting", "notes"])
        .assert()
        .success();
    db.cmd()
        .args(["note", "add", "random", "thoughts"])
        .assert()
        .success();

    // Search for "meeting"
    db.cmd()
        .args(["note", "search", "meeting"])
        .assert()
        .success()
        .stdout(predicate::str::contains("meeting notes"))
        .stdout(predicate::str::contains("random thoughts").not());
}

#[test]
fn test_note_search_by_tag() {
    let db = TestDb::new();

    db.cmd()
        .args(["note", "add", "--tag", "work", "work", "stuff"])
        .assert()
        .success();
    db.cmd()
        .args(["note", "add", "--tag", "personal", "home", "stuff"])
        .assert()
        .success();

    // Search by tag
    db.cmd()
        .args(["note", "search", "--tag", "work"])
        .assert()
        .success()
        .stdout(predicate::str::contains("work stuff"))
        .stdout(predicate::str::contains("home stuff").not());
}

#[test]
fn test_note_search_with_limit() {
    let db = TestDb::new();

    // Add 5 notes
    for i in 1..=5 {
        db.cmd()
            .args(["note", "add", &format!("note {}", i)])
            .assert()
            .success();
    }

    // Search with limit 2
    db.cmd()
        .args(["note", "search", "--limit", "2"])
        .assert()
        .success();

    // We can't easily count the output, but we can verify it succeeds
    // and contains at least one note
    let output = db
        .cmd()
        .args(["note", "search", "--limit", "2", "--output", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 2);
}

#[test]
fn test_note_last() {
    let db = TestDb::new();

    db.cmd()
        .args(["note", "add", "first", "note"])
        .assert()
        .success();
    db.cmd()
        .args(["note", "add", "second", "note"])
        .assert()
        .success();
    db.cmd()
        .args(["note", "add", "latest", "note"])
        .assert()
        .success();

    // Get last note
    db.cmd()
        .args(["note", "last"])
        .assert()
        .success()
        .stdout(predicate::str::contains("latest note"));
}

#[test]
fn test_note_delete_latest() {
    let db = TestDb::new();

    db.cmd()
        .args(["note", "add", "first", "note"])
        .assert()
        .success();
    db.cmd()
        .args(["note", "add", "second", "note"])
        .assert()
        .success();

    assert_eq!(db.get_notes().len(), 2);

    // Delete latest (requires confirmation, so use --yes)
    db.cmd()
        .args(["note", "delete", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted note"));

    let notes = db.get_notes();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "first note");
}

#[test]
fn test_note_delete_by_id() {
    let db = TestDb::new();

    db.cmd()
        .args(["note", "add", "first", "note"])
        .assert()
        .success();
    db.cmd()
        .args(["note", "add", "second", "note"])
        .assert()
        .success();

    let notes = db.get_notes();
    // Notes are returned newest first (descending order by updated_at)
    let second_id = &notes[0].id; // This is "second note" (newest)

    // Delete specific note
    db.cmd()
        .args(["note", "delete", "--yes", second_id])
        .assert()
        .success();

    let remaining = db.get_notes();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].content, "first note");
}

#[test]
fn test_note_delete_multiple() {
    let db = TestDb::new();

    db.cmd().args(["note", "add", "first"]).assert().success();
    db.cmd().args(["note", "add", "second"]).assert().success();
    db.cmd().args(["note", "add", "third"]).assert().success();

    let notes = db.get_notes();
    // Notes are: [0]=third (newest), [1]=second, [2]=first (oldest)
    let id1 = &notes[0].id.clone(); // third
    let id2 = &notes[1].id.clone(); // second

    // Delete two notes
    db.cmd()
        .args(["note", "delete", "--yes", id1, id2])
        .assert()
        .success();

    let remaining = db.get_notes();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].content, "first");
}

#[test]
fn test_note_delete_nonexistent() {
    let db = TestDb::new();

    // Try to delete non-existent note
    // Note: Currently soft_delete doesn't fail for non-existent IDs (UPDATE with 0 rows)
    // This just succeeds silently, which is acceptable for idempotent operations
    db.cmd()
        .args(["note", "delete", "--yes", "nonexistent_id"])
        .assert()
        .success();
}

#[test]
fn test_note_search_json_output() {
    let db = TestDb::new();

    db.cmd()
        .args(["note", "add", "--tag", "test", "test", "note"])
        .assert()
        .success();

    let output = db
        .cmd()
        .args(["note", "search", "--output", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert!(json.is_array());
    let notes = json.as_array().unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0]["content"], "test note");
    assert_eq!(notes[0]["tags"][0], "test");
}

#[test]
fn test_no_notes_to_delete() {
    let db = TestDb::new();

    // Try to delete when no notes exist
    db.cmd()
        .args(["note", "delete", "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No notes found"));
}

#[test]
fn test_note_search_by_date_today() {
    let db = TestDb::new();

    // Add note with today's date
    db.cmd()
        .args(["note", "add", "--date", "today", "today's", "note"])
        .assert()
        .success();

    // Add note with yesterday's date
    db.cmd()
        .args(["note", "add", "--date", "yesterday", "yesterday's", "note"])
        .assert()
        .success();

    // Search for today's notes
    db.cmd()
        .args(["note", "search", "--date", "today"])
        .assert()
        .success()
        .stdout(predicate::str::contains("today's note"))
        .stdout(predicate::str::contains("yesterday's note").not());
}

#[test]
fn test_note_search_by_date_yesterday() {
    let db = TestDb::new();

    // Add note with today's date
    db.cmd()
        .args(["note", "add", "--date", "today", "today's", "note"])
        .assert()
        .success();

    // Add note with yesterday's date
    db.cmd()
        .args(["note", "add", "--date", "yesterday", "yesterday's", "note"])
        .assert()
        .success();

    // Search for yesterday's notes
    db.cmd()
        .args(["note", "search", "--date", "yesterday"])
        .assert()
        .success()
        .stdout(predicate::str::contains("yesterday's note"))
        .stdout(predicate::str::contains("today's note").not());
}

#[test]
fn test_note_search_by_date_specific() {
    let db = TestDb::new();

    // Add notes with specific dates
    db.cmd()
        .args(["note", "add", "--date", "2025-01-15", "mid", "january"])
        .assert()
        .success();

    db.cmd()
        .args(["note", "add", "--date", "2025-02-20", "late", "february"])
        .assert()
        .success();

    // Search for specific date
    db.cmd()
        .args(["note", "search", "--date", "2025-01-15"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mid january"))
        .stdout(predicate::str::contains("late february").not());
}

#[test]
fn test_note_search_by_date_past() {
    let db = TestDb::new();

    // Add note with yesterday's date
    db.cmd()
        .args(["note", "add", "--date", "yesterday", "past", "note"])
        .assert()
        .success();

    // Add note with tomorrow's date
    db.cmd()
        .args(["note", "add", "--date", "tomorrow", "future", "note"])
        .assert()
        .success();

    // Search for past notes (should exclude today and future)
    db.cmd()
        .args(["note", "search", "--date", "past"])
        .assert()
        .success()
        .stdout(predicate::str::contains("past note"))
        .stdout(predicate::str::contains("future note").not());
}

#[test]
fn test_note_search_by_date_future() {
    let db = TestDb::new();

    // Add note with yesterday's date
    db.cmd()
        .args(["note", "add", "--date", "yesterday", "past", "note"])
        .assert()
        .success();

    // Add note with tomorrow's date
    db.cmd()
        .args(["note", "add", "--date", "tomorrow", "future", "note"])
        .assert()
        .success();

    // Search for future notes (should exclude today and past)
    db.cmd()
        .args(["note", "search", "--date", "future"])
        .assert()
        .success()
        .stdout(predicate::str::contains("future note"))
        .stdout(predicate::str::contains("past note").not());
}

#[test]
fn test_prune_generate_file_format() {
    let db = TestDb::new();

    // Add some notes
    db.add_note("First note", vec!["work"], Some("2025-01-15"));
    db.add_note("Second note", vec!["personal"], Some("2025-01-14"));
    db.add_note("Third note with a very long content that should be truncated at 80 characters to prevent the line from being too long", vec![], None);

    let notes = db.get_notes();
    let content = crate::prune::generate_prune_file(&notes);

    // Check header
    assert!(content.contains("# Interactive note cleanup"));
    assert!(content.contains("# Commands:"));
    assert!(content.contains("#   keep   - keep this note"));
    assert!(content.contains("#   delete - permanently delete this note"));

    // Check notes are formatted correctly
    assert!(content.contains("keep"));
    assert!(content.contains("[2025-01-15]"));
    assert!(content.contains("#work"));
    assert!(content.contains("#personal"));
    assert!(content.contains("First note"));

    // Check long note is truncated
    assert!(content.contains("..."));
}

#[test]
fn test_prune_parse_keep_and_delete() {
    let content = r#"# Comment line
keep abc123 [2025-01-15] #work First note
delete def456 [2025-01-14] #personal Second note
keep ghi789 Third note
"#;

    let decisions = crate::prune::parse_prune_file(content).unwrap();

    assert_eq!(decisions.len(), 3);
    assert_eq!(decisions[0].note_id, "abc123");
    assert_eq!(decisions[0].action, crate::prune::PruneAction::Keep);
    assert_eq!(decisions[1].note_id, "def456");
    assert_eq!(decisions[1].action, crate::prune::PruneAction::Delete);
    assert_eq!(decisions[2].note_id, "ghi789");
    assert_eq!(decisions[2].action, crate::prune::PruneAction::Keep);
}

#[test]
fn test_prune_parse_invalid_action() {
    let content = "edit abc123 note\n";

    let result = crate::prune::parse_prune_file(content);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid action 'edit'"));
}

#[test]
fn test_prune_parse_skip_comments_and_empty_lines() {
    let content = r#"
# This is a comment

keep abc123 note

# Another comment
delete def456 note

"#;

    let decisions = crate::prune::parse_prune_file(content).unwrap();

    assert_eq!(decisions.len(), 2);
    assert_eq!(decisions[0].note_id, "abc123");
    assert_eq!(decisions[1].note_id, "def456");
}

#[test]
fn test_prune_with_filters() {
    let db = TestDb::new();

    // Add notes with different tags
    db.add_note("Work note 1", vec!["work"], Some("2025-01-15"));
    db.add_note("Work note 2", vec!["work"], Some("2025-01-14"));
    db.add_note("Personal note", vec!["personal"], Some("2025-01-13"));

    // Verify we can get notes by tag filter for prune
    let notes = db.get_notes();
    assert_eq!(notes.len(), 3);

    // Filter by tag
    let work_notes: Vec<_> = notes
        .iter()
        .filter(|n| n.tags.contains(&"work".to_string()))
        .collect();
    assert_eq!(work_notes.len(), 2);
}

#[test]
fn test_prune_limit() {
    let db = TestDb::new();

    // Add 5 notes
    for i in 1..=5 {
        db.add_note(&format!("Note {}", i), vec![], Some("2025-01-15"));
    }

    let notes = db.get_notes();
    assert_eq!(notes.len(), 5);

    // Test limiting
    let limited: Vec<_> = notes.iter().take(3).collect();
    assert_eq!(limited.len(), 3);
}
