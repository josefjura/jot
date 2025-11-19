use std::path::Path;

use jot_core::SearchQuery;

use crate::{
    args::{NoteCommand, NoteSearchArgs},
    db::LocalDb,
    editor::{Editor, ParseTemplate},
    formatters::NoteSearchFormatter,
};

const TEMPLATE: &str = r#"tags = ["work", "important"]
#tags = [""]
#date = "YYYY-MM-DD"
+++"#;

pub fn note_cmd(
    db_path: &Path,
    subcommand: NoteCommand,
) -> Result<(), anyhow::Error> {
    let db = LocalDb::open(db_path)?;

    match subcommand {
        NoteCommand::Add(args) => {
            let note = if args.edit {
                let editor = Editor::new(TEMPLATE);
                let result = editor.open(&args)?;

                let tags = result.tags.iter().map(|t| t.to_string()).collect();
                let date = result.date.to_date().format("%Y-%m-%d").to_string();

                db.create_note(result.content, tags, Some(date))?
            } else {
                let date = args.date.to_date().format("%Y-%m-%d").to_string();
                db.create_note(args.content.join(" "), args.tag, Some(date))?
            };

            println!("Note added successfully ({})", note.id);
        }
        NoteCommand::Search(args) => {
            let query = build_search_query(&args);
            let notes = db.search_notes(&query)?;

            let mut formatter = NoteSearchFormatter::new(args);
            formatter
                .print_notes(&notes)
                .map_err(|e| anyhow::anyhow!("Error while formatting notes: {}", e))?;
        }
        NoteCommand::Last(args) => {
            let search_args = NoteSearchArgs {
                term: args.term,
                tag: args.tag,
                date: None,
                lines: None,
                limit: Some(1),
                output: args.output,
            };

            let query = build_search_query(&search_args);
            let notes = db.search_notes(&query)?;

            let mut formatter = NoteSearchFormatter::new(search_args);
            formatter
                .print_notes(&notes)
                .map_err(|e| anyhow::anyhow!("Error while formatting notes: {}", e))?;
        }
        NoteCommand::Edit(args) => {
            // Get the note to edit
            let note = if let Some(id) = args.id {
                // Edit specific note by ID
                db.get_note_by_id(&id)?
                    .ok_or_else(|| anyhow::anyhow!("Note with ID '{}' not found", id))?
            } else {
                // Edit most recent note
                let query = SearchQuery {
                    text: None,
                    tags: vec![],
                    date_from: None,
                    date_to: None,
                    include_deleted: false,
                    limit: Some(1),
                };
                let notes = db.search_notes(&query)?;
                notes
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("No notes found to edit"))?
            };

            // Create template with existing note data
            let tags_str = note
                .tags
                .iter()
                .map(|t| format!("\"{}\"", t))
                .collect::<Vec<_>>()
                .join(", ");
            let date_str = note.date.as_deref().unwrap_or("today");

            let template = format!(
                "tags = [{}]\ndate = \"{}\"\n+++\n{}",
                tags_str, date_str, note.content
            );

            // Open in editor
            let editor = Editor::new(&template);
            let edited_content = editor.with_initial_content(&template, "")?;
            let parsed = edited_content.parse_template()?;

            // Update the note
            let tags = parsed.tags.iter().map(|t| t.to_string()).collect();
            let date = parsed.date.to_date().format("%Y-%m-%d").to_string();

            db.update_note(&note.id, parsed.content, tags, Some(date))?;

            println!("Note updated successfully ({})", note.id);
        }
        NoteCommand::Delete(args) => {
            // Get note IDs to delete
            let ids_to_delete: Vec<String> = if args.ids.is_empty() {
                // Delete most recent note
                let query = SearchQuery {
                    text: None,
                    tags: vec![],
                    date_from: None,
                    date_to: None,
                    include_deleted: false,
                    limit: Some(1),
                };
                let notes = db.search_notes(&query)?;
                if notes.is_empty() {
                    return Err(anyhow::anyhow!("No notes found to delete"));
                }
                vec![notes[0].id.clone()]
            } else {
                args.ids
            };

            // Confirm deletion unless --yes flag is provided
            if !args.yes {
                for id in &ids_to_delete {
                    let note = db
                        .get_note_by_id(id)?
                        .ok_or_else(|| anyhow::anyhow!("Note with ID '{}' not found", id))?;

                    let preview = if note.content.len() > 60 {
                        format!("{}...", &note.content[..60])
                    } else {
                        note.content.clone()
                    };

                    print!("Delete note \"{}\"? [y/N]: ", preview);
                    std::io::Write::flush(&mut std::io::stdout())?;

                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;

                    if !input.trim().eq_ignore_ascii_case("y") {
                        println!("Skipped deleting note {}", id);
                        continue;
                    }

                    db.soft_delete_note(id)?;
                    println!("Deleted note {}", id);
                }
            } else {
                // Delete without confirmation
                for id in &ids_to_delete {
                    db.soft_delete_note(id)?;
                    println!("Deleted note {}", id);
                }
            }
        }
    };

    Ok(())
}

fn build_search_query(args: &NoteSearchArgs) -> SearchQuery {
    SearchQuery {
        text: args.term.clone(),
        tags: args.tag.clone(),
        date_from: None, // TODO: Implement date filtering
        date_to: None,
        include_deleted: false,
        limit: args.limit.map(|l| l as usize),
    }
}
