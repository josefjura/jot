use anyhow::{Context, Result};
use jot_core::Note;
use std::io::{self, Write};

#[derive(Debug, PartialEq)]
pub enum PruneAction {
    Keep,
    Delete,
}

#[derive(Debug)]
pub struct PruneDecision {
    pub note_id: String,
    pub action: PruneAction,
}

/// Generate the prune file content for editing
pub fn generate_prune_file(notes: &[Note]) -> String {
    let mut content = String::new();

    // Header with instructions
    content.push_str("# Interactive note cleanup\n");
    content.push_str("# \n");
    content.push_str("# Commands:\n");
    content.push_str("#   keep   - keep this note (default)\n");
    content.push_str("#   delete - permanently delete this note\n");
    content.push_str("#\n");
    content.push_str("# Lines starting with '#' are ignored.\n");
    content.push_str("# Edit the command word (keep/delete) on each line.\n");
    content.push('\n');

    // Add each note as a single line with preview
    for note in notes {
        let date_str = note
            .subject_date
            .as_ref()
            .map(|d| format!("[{}]", d))
            .unwrap_or_else(|| String::from(""));

        let tags_str = if note.tags.is_empty() {
            String::new()
        } else {
            format!(
                " #{}",
                note.tags
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(" #")
            )
        };

        // Get first line of content for preview
        let preview = note
            .content
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(80)
            .collect::<String>();

        let preview_suffix = if note.content.lines().count() > 1 || preview.len() >= 80 {
            "..."
        } else {
            ""
        };

        content.push_str(&format!(
            "keep {} {}{} {}{}\n",
            note.id, date_str, tags_str, preview, preview_suffix
        ));
    }

    content
}

/// Parse the edited prune file and extract decisions
pub fn parse_prune_file(content: &str) -> Result<Vec<PruneDecision>> {
    let mut decisions = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse the line: <action> <id> <rest...>
        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        if parts.len() < 2 {
            return Err(anyhow::anyhow!(
                "Invalid format at line {}: expected '<action> <id> ...'",
                line_num + 1
            ));
        }

        let action = match parts[0] {
            "keep" => PruneAction::Keep,
            "delete" => PruneAction::Delete,
            other => {
                return Err(anyhow::anyhow!(
                    "Invalid action '{}' at line {}. Expected 'keep' or 'delete'",
                    other,
                    line_num + 1
                ))
            }
        };

        let note_id = parts[1].to_string();

        decisions.push(PruneDecision { note_id, action });
    }

    Ok(decisions)
}

/// Open editor with the prune file
pub fn open_prune_editor(initial_content: &str) -> Result<String> {
    // Read VISUAL or EDITOR environment variable
    let editor = std::env::var("VISUAL")
        .unwrap_or_else(|_| std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string()));

    // Create temporary file
    let mut tempfile = tempfile::NamedTempFile::new().context("Failed to create temporary file")?;

    // Write initial content
    tempfile
        .write_all(initial_content.as_bytes())
        .context("Failed to write initial content")?;

    // Flush to ensure content is written
    tempfile.flush().context("Failed to flush temp file")?;

    // Open editor
    let mut child = std::process::Command::new(editor)
        .arg(tempfile.path())
        .spawn()
        .context("Failed to open editor")?;

    let status = child.wait().context("Failed to wait for editor")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Editor returned non-zero exit code"));
    }

    // Read edited content
    let edited_content =
        std::fs::read_to_string(tempfile.path()).context("Failed to read edited file")?;

    Ok(edited_content)
}

/// Show summary and confirm deletion
pub fn confirm_deletions(notes_to_delete: &[&Note]) -> Result<bool> {
    if notes_to_delete.is_empty() {
        println!("No notes to delete.");
        return Ok(false);
    }

    println!("\nReviewing changes:");
    println!("  Delete: {} note(s)", notes_to_delete.len());
    println!();
    println!("Delete these notes?");

    for note in notes_to_delete {
        let date_str = note
            .subject_date
            .as_ref()
            .map(|d| format!("[{}]", d))
            .unwrap_or_else(|| String::from(""));

        let tags_str = if note.tags.is_empty() {
            String::new()
        } else {
            format!(
                " #{}",
                note.tags
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(" #")
            )
        };

        let preview = note
            .content
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(60)
            .collect::<String>();

        let preview_suffix = if note.content.lines().count() > 1 || preview.len() >= 60 {
            "..."
        } else {
            ""
        };

        println!("  {}{} {}{}", date_str, tags_str, preview, preview_suffix);
    }

    print!("\nProceed? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    fn create_test_note(id: &str, content: &str, tags: Vec<&str>, date: Option<&str>) -> Note {
        Note {
            id: id.to_string(),
            content: content.to_string(),
            tags: tags.into_iter().map(|t| t.to_string()).collect(),
            subject_date: date.map(|d| d.to_string()),
            created_at: 0,
            updated_at: 0,
            deleted_at: None,
        }
    }

    #[test]
    fn test_generate_prune_file() {
        let notes = vec![
            create_test_note("abc123", "First note", vec!["work"], Some("2025-01-15")),
            create_test_note(
                "def456",
                "Second note",
                vec!["personal"],
                Some("2025-01-14"),
            ),
            create_test_note("ghi789", "Third note", vec![], None),
        ];

        let content = generate_prune_file(&notes);

        assert!(content.contains("keep abc123 [2025-01-15] #work First note"));
        assert!(content.contains("keep def456 [2025-01-14] #personal Second note"));
        assert!(content.contains("keep ghi789  Third note"));
        assert!(content.contains("# Commands:"));
    }

    #[test]
    fn test_generate_prune_file_multiline_note() {
        let notes = vec![create_test_note(
            "abc123",
            "First line\nSecond line\nThird line",
            vec![],
            None,
        )];

        let content = generate_prune_file(&notes);

        // Should only show first line with ellipsis
        assert!(content.contains("keep abc123  First line..."));
        assert!(!content.contains("Second line"));
    }

    #[test]
    fn test_generate_prune_file_long_note() {
        let long_content = "a".repeat(100);
        let notes = vec![create_test_note("abc123", &long_content, vec![], None)];

        let content = generate_prune_file(&notes);

        // Should truncate at 80 characters
        assert!(content.contains("..."));
        let line = content.lines().find(|l| l.contains("abc123")).unwrap_or("");
        assert!(line.len() < 120); // Should be reasonably short
    }

    #[test]
    fn test_parse_prune_file_basic() {
        let content = "keep abc123 [2025-01-15] note text\ndelete def456 [2025-01-14] old note";

        let decisions = parse_prune_file(content).expect("Failed to parse");

        assert_eq!(decisions.len(), 2);
        assert_eq!(decisions[0].note_id, "abc123");
        assert_eq!(decisions[0].action, PruneAction::Keep);
        assert_eq!(decisions[1].note_id, "def456");
        assert_eq!(decisions[1].action, PruneAction::Delete);
    }

    #[test]
    fn test_parse_prune_file_with_comments() {
        let content =
            "# This is a comment\nkeep abc123 note\n\n# Another comment\ndelete def456 old";

        let decisions = parse_prune_file(content).expect("Failed to parse");

        assert_eq!(decisions.len(), 2);
        assert_eq!(decisions[0].note_id, "abc123");
        assert_eq!(decisions[1].note_id, "def456");
    }

    #[test]
    fn test_parse_prune_file_invalid_action() {
        let content = "edit abc123 note text";

        let result = parse_prune_file(content);

        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.to_string().contains("Invalid action 'edit'"));
        }
    }

    #[test]
    fn test_parse_prune_file_invalid_format() {
        let content = "keep";

        let result = parse_prune_file(content);

        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.to_string().contains("Invalid format"));
        }
    }

    #[test]
    fn test_parse_prune_file_empty() {
        let content = "# Just comments\n\n# Nothing else";

        let decisions = parse_prune_file(content).expect("Failed to parse");

        assert_eq!(decisions.len(), 0);
    }

    #[test]
    fn test_generate_prune_file_multiple_tags() {
        let notes = vec![create_test_note(
            "abc123",
            "Note with tags",
            vec!["work", "important", "urgent"],
            Some("2025-01-15"),
        )];

        let content = generate_prune_file(&notes);

        assert!(content.contains("#work #important #urgent"));
    }
}
