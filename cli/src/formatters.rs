use crate::{
    args::{NoteSearchArgs, OutputFormat},
};
use chrono::{DateTime, Utc};
use jot_core::Note;
use std::io::{self, Write};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

fn timestamp_to_datetime(ts: i64) -> DateTime<Utc> {
    DateTime::from_timestamp_millis(ts).unwrap_or_else(|| Utc::now())
}

pub struct NoteSearchFormatter {
    args: NoteSearchArgs,
    writer: BufferWriter,
}

impl NoteSearchFormatter {
    pub fn new(args: NoteSearchArgs) -> Self {
        let color_choice = match args.output {
            OutputFormat::Plain => ColorChoice::Never,
            OutputFormat::Json => ColorChoice::Never,
            OutputFormat::Pretty => ColorChoice::Auto,
        };

        Self {
            args,
            writer: BufferWriter::stdout(color_choice),
        }
    }

    pub fn print_notes(&mut self, notes: &[Note]) -> io::Result<()> {
        let mut buffer = self.writer.buffer();

        if self.args.output == OutputFormat::Json {
            self.print_json(notes, &mut buffer)?;
        } else {
            if notes.is_empty() {
                writeln!(buffer, "No notes found")?;
            } else {
                for note in notes {
                    self.print_note(&mut buffer, note, self.args.output == OutputFormat::Pretty)?;
                }
            }
        }

        self.writer.print(&buffer)?;
        Ok(())
    }

    fn print_note(
        &mut self,
        buffer: &mut termcolor::Buffer,
        note: &Note,
        do_pretty_print: bool,
    ) -> io::Result<()> {
        if do_pretty_print {
            self.pretty_print_metadata(buffer, note)?;
        } else {
            self.print_metadata(buffer, note)?;
        }
        // Print content
        self.print_content(buffer, &note.content)?;

        // Line break if pretty print
        if do_pretty_print {
            writeln!(buffer)?;
        }

        Ok(())
    }

    fn pretty_print_metadata(&self, buffer: &mut termcolor::Buffer, note: &Note) -> io::Result<()> {
        buffer.set_color(
            ColorSpec::new()
                .set_fg(Some(Color::Cyan))
                .set_intense(false),
        )?;

        writeln!(buffer, "\u{1F4CB} {}", &note.id[..8])?; // Show first 8 chars of ULID

        write!(
            buffer,
            "\u{1F4C5} [{}]",
            timestamp_to_datetime(note.created_at).format("%Y-%m-%d %H:%M")
        )?;
        writeln!(buffer, "[{}]", timestamp_to_datetime(note.updated_at).format("%Y-%m-%d %H:%M"))?;

        if !note.tags.is_empty() {
            write!(buffer, "\u{1F516}")?;
            writeln!(buffer, " {}", note.tags.join(","))?;
        }

        buffer.reset()?;

        Ok(())
    }

    fn print_metadata(&self, buffer: &mut termcolor::Buffer, note: &Note) -> io::Result<()> {
        let mut metadata = Vec::new();

        metadata.push(format!("{}", &note.id[..8])); // Show first 8 chars of ULID

        metadata.push(format!("[{}]", timestamp_to_datetime(note.created_at).format("%Y-%m-%d %H:%M")));
        metadata.push(format!("[{}]", timestamp_to_datetime(note.updated_at).format("%Y-%m-%d %H:%M")));

        if !note.tags.is_empty() {
            metadata.push(format!(" [{}]", note.tags.join(",")));
        }

        write!(buffer, "{} ", metadata.join(" "))?;

        Ok(())
    }

    fn print_content(&self, buffer: &mut termcolor::Buffer, content: &str) -> io::Result<()> {
        let content = self.create_preview(content);

        writeln!(buffer, "{}", content)?;

        Ok(())
    }

    fn print_json(&mut self, notes: &[Note], buffer: &mut termcolor::Buffer) -> io::Result<()> {
        let json = serde_json::to_string_pretty(notes)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        writeln!(buffer, "{}", json)?;
        Ok(())
    }

    fn create_preview(&self, content: &str) -> String {
        let max_lines = self.args.lines.unwrap_or(std::usize::MAX);
        let preview: String = content
            .lines()
            .take(max_lines)
            .collect::<Vec<_>>()
            .join("\n");

        if content.lines().count() > max_lines {
            format!("{}...", preview)
        } else {
            preview
        }
    }
}

#[test]
fn test_note_search_formatter_create_preview_one_line() {
    let formatter = NoteSearchFormatter::new(NoteSearchArgs {
        lines: Some(1),
        ..Default::default()
    });

    assert_eq!(formatter.create_preview("One\nTwo\nThree\nFour"), "One...");

    assert_eq!(formatter.create_preview("One"), "One");

    assert_eq!(formatter.create_preview("Short note"), "Short note");

    assert_eq!(
        formatter.create_preview("Multi-line note\nWith several\nDistinct lines\nTo test preview"),
        "Multi-line note..."
    );
}

#[test]
fn test_note_search_formatter_create_preview_two_lines() {
    let formatter = NoteSearchFormatter::new(NoteSearchArgs {
        lines: Some(2),
        ..Default::default()
    });

    assert_eq!(
        formatter.create_preview("One\nTwo\nThree\nFour"),
        "One\nTwo..."
    );

    assert_eq!(formatter.create_preview("One\nTwo"), "One\nTwo");
}
