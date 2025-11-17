use std::path::Path;

use jot_core::SearchQuery;

use crate::{
    args::{NoteCommand, NoteSearchArgs},
    db::LocalDb,
    editor::Editor,
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
