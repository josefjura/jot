use crate::args::{NoteSearchArgs, OutputFormat};
use jot_core::Note;
use std::io::{self, Write};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

pub struct NoteSearchFormatter {
    args: NoteSearchArgs,
    writer: BufferWriter,
}

impl NoteSearchFormatter {
    pub fn new(args: NoteSearchArgs) -> Self {
        let color_choice = match args.output {
            OutputFormat::Plain => ColorChoice::Never,
            OutputFormat::Json => ColorChoice::Never,
            OutputFormat::Id => ColorChoice::Never,
            OutputFormat::Pretty => ColorChoice::Auto,
        };

        Self {
            args,
            writer: BufferWriter::stdout(color_choice),
        }
    }

    pub fn print_notes(&mut self, notes: &[Note]) -> io::Result<()> {
        let mut buffer = self.writer.buffer();

        match self.args.output {
            OutputFormat::Json => {
                self.print_json(notes, &mut buffer)?;
            }
            OutputFormat::Id => {
                for note in notes {
                    writeln!(buffer, "{}", note.id)?;
                }
            }
            _ => {
                if notes.is_empty() {
                    writeln!(buffer, "No notes found")?;
                } else {
                    for note in notes {
                        self.print_note(
                            &mut buffer,
                            note,
                            self.args.output == OutputFormat::Pretty,
                        )?;
                    }
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

        // Show note date if present
        if let Some(ref date) = note.date {
            writeln!(buffer, "\u{1F4C5} {}", date)?;
        }

        if !note.tags.is_empty() {
            write!(buffer, "\u{1F516}")?;
            writeln!(buffer, " {}", note.tags.join(","))?;
        }

        buffer.reset()?;

        Ok(())
    }

    fn print_metadata(&self, buffer: &mut termcolor::Buffer, note: &Note) -> io::Result<()> {
        let mut metadata = Vec::new();

        metadata.push(note.id[..8].to_string()); // Show first 8 chars of ULID

        // Show note date if present
        if let Some(ref date) = note.date {
            metadata.push(format!("[{}]", date));
        }

        if !note.tags.is_empty() {
            metadata.push(format!("[{}]", note.tags.join(",")));
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
            .map_err(io::Error::other)?;

        writeln!(buffer, "{}", json)?;
        Ok(())
    }

    fn create_preview(&self, content: &str) -> String {
        let max_lines = self.args.lines.unwrap_or(usize::MAX);
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
