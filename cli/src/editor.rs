use std::{
    collections::HashSet,
    io::{self, Read, Write},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{args::NoteAddArgs, utils::date_source::DateSource};

#[derive(Debug, Deserialize, Serialize)]
pub struct EditorTemplate {
    #[serde(default)]
    pub tags: HashSet<String>,
    #[serde(default)]
    pub date: DateSource,
    #[serde(default)]
    pub today: bool,
    #[serde(skip)]
    pub content: String,
}

impl EditorTemplate {
    pub fn new() -> Self {
        EditorTemplate {
            tags: HashSet::new(),
            date: DateSource::Today,
            today: false,
            content: String::new(),
        }
    }
}

pub struct Editor {
    template: String,
}

impl Editor {
    pub fn new(template: &str) -> Self {
        Editor {
            template: template.to_string(),
        }
    }

    /// Format error message as safe TOML comments
    fn format_error_header(error: &anyhow::Error, content: &str) -> String {
        // Each line of the error message gets prefixed with "# " to make it a TOML comment
        let error_lines = format!("{}", error)
            .lines()
            .map(|line| format!("# {}", line))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "# ===== PARSING ERROR =====\n{}\n# ===== Fix the issue below and save again =====\n\n{}",
            error_lines, content
        )
    }

    fn read_from_file(&self, tempfile: tempfile::NamedTempFile) -> anyhow::Result<String> {
        // Read VISUAL or EDITOR environment variable
        let editor = std::env::var("VISUAL")
            .unwrap_or_else(|_| std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string()));

        let mut child = std::process::Command::new(editor)
            .arg(tempfile.path())
            .spawn()
            .context("Failed to open editor")?;

        let status = child.wait().context("Failed to wait for editor")?;

        if !status.success() {
            return Err(anyhow::anyhow!("Editor returned non-zero exit code"));
        }

        // Read content of the tempfile
        let mut content = String::new();
        let mut file = std::fs::File::open(tempfile.path())
            .context("Failed to open temporary file".to_string())?;
        file.read_to_string(&mut content)
            .context("Failed to read temporary file".to_string())?;

        Ok(content)
    }

    pub fn open(&self, args: &NoteAddArgs) -> anyhow::Result<EditorTemplate> {
        print!("\x1B[?1049h");
        io::stdout().flush()?;

        let mut current_content = self.template.to_string();

        loop {
            let edited_content = self.with_initial_content(&current_content, &args.content.join(" "))?;

            match edited_content.parse_template() {
                Ok(parsed) => {
                    // Success! Restore terminal and return
                    print!("\x1B[?1049l\x1B[H\x1B[2J");
                    io::stdout().flush()?;
                    return Ok(parsed);
                }
                Err(e) => {
                    // Restore terminal for prompt
                    print!("\x1B[?1049l\x1B[H\x1B[2J");
                    io::stdout().flush()?;

                    // Show error and prompt user
                    println!("Error parsing note: {}\n", e);
                    println!("Your changes have been preserved in the editor.");
                    println!("Do you want to:");
                    println!("  [R]etry (re-open editor with your changes)");
                    println!("  [S]ave anyway (ignore frontmatter, save as plain text)");
                    println!("  [A]bort (discard changes)");
                    print!("Choice (R/s/a): ");
                    io::stdout().flush()?;

                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    let choice = input.trim().to_lowercase();

                    match choice.as_str() {
                        "" | "r" => {
                            // Retry - prepend error message and re-open
                            current_content = Self::format_error_header(&e, &edited_content);
                            // Re-enter alternate screen for next iteration
                            print!("\x1B[?1049h");
                            io::stdout().flush()?;
                            continue;
                        }
                        "s" => {
                            // Save anyway - treat everything as content
                            print!("\x1B[?1049l\x1B[H\x1B[2J");
                            io::stdout().flush()?;
                            return Ok(EditorTemplate {
                                tags: HashSet::new(),
                                date: args.date.clone(),
                                today: false,
                                content: edited_content,
                            });
                        }
                        "a" => {
                            // Abort
                            return Err(anyhow::anyhow!("User aborted note creation"));
                        }
                        _ => {
                            // Invalid input - retry prompt
                            println!("\nInvalid choice. Please enter R, S, or A.");
                            // Prepend error for next iteration
                            current_content = Self::format_error_header(&e, &edited_content);
                            // Re-enter alternate screen for next iteration
                            print!("\x1B[?1049h");
                            io::stdout().flush()?;
                            continue;
                        }
                    }
                }
            }
        }
    }

    pub fn with_initial_content(&self, template: &str, _content: &str) -> anyhow::Result<String> {
        let mut tempfile =
            tempfile::NamedTempFile::new().context("Failed to create temporary file")?;

        // Write initial content
        std::io::Write::write_all(&mut tempfile, template.as_bytes())
            .context("Failed to write initial content")?;

        self.read_from_file(tempfile)
    }

    /// Open editor with error recovery for editing existing notes
    pub fn open_with_recovery(&self, initial_content: &str) -> anyhow::Result<EditorTemplate> {
        print!("\x1B[?1049h");
        io::stdout().flush()?;

        let mut current_content = initial_content.to_string();

        loop {
            let edited_content = self.with_initial_content(&current_content, "")?;

            match edited_content.parse_template() {
                Ok(parsed) => {
                    // Success! Restore terminal and return
                    print!("\x1B[?1049l\x1B[H\x1B[2J");
                    io::stdout().flush()?;
                    return Ok(parsed);
                }
                Err(e) => {
                    // Restore terminal for prompt
                    print!("\x1B[?1049l\x1B[H\x1B[2J");
                    io::stdout().flush()?;

                    // Show error and prompt user
                    println!("Error parsing note: {}\n", e);
                    println!("Your changes have been preserved in the editor.");
                    println!("Do you want to:");
                    println!("  [R]etry (re-open editor with your changes)");
                    println!("  [S]ave anyway (ignore frontmatter, save as plain text)");
                    println!("  [A]bort (discard changes)");
                    print!("Choice (R/s/a): ");
                    io::stdout().flush()?;

                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    let choice = input.trim().to_lowercase();

                    match choice.as_str() {
                        "" | "r" => {
                            // Retry - prepend error message and re-open
                            current_content = Self::format_error_header(&e, &edited_content);
                            // Re-enter alternate screen for next iteration
                            print!("\x1B[?1049h");
                            io::stdout().flush()?;
                            continue;
                        }
                        "s" => {
                            // Save anyway - treat everything as content with default date
                            print!("\x1B[?1049l\x1B[H\x1B[2J");
                            io::stdout().flush()?;
                            return Ok(EditorTemplate {
                                tags: HashSet::new(),
                                date: DateSource::Today,
                                today: false,
                                content: edited_content,
                            });
                        }
                        "a" => {
                            // Abort
                            return Err(anyhow::anyhow!("User aborted edit"));
                        }
                        _ => {
                            // Invalid input - retry prompt
                            println!("\nInvalid choice. Please enter R, S, or A.");
                            // Prepend error for next iteration
                            current_content = Self::format_error_header(&e, &edited_content);
                            // Re-enter alternate screen for next iteration
                            print!("\x1B[?1049h");
                            io::stdout().flush()?;
                            continue;
                        }
                    }
                }
            }
        }
    }
}

pub trait ParseTemplate {
    fn parse_template(&self) -> anyhow::Result<EditorTemplate>;
}

impl ParseTemplate for String {
    fn parse_template(&self) -> anyhow::Result<EditorTemplate> {
        // Split on lines to find the +++ delimiter (must be on its own line)
        let lines: Vec<&str> = self.lines().collect();

        // Find the first line that is just +++ (with optional whitespace)
        let delimiter_pos = lines
            .iter()
            .position(|line| line.trim() == "+++");

        let (toml_lines, content_lines) = match delimiter_pos {
            Some(pos) => {
                let toml = &lines[..pos];
                let content = &lines[pos + 1..]; // Skip the delimiter line
                (toml, content)
            }
            None => {
                // No delimiter found - treat entire input as TOML frontmatter with no content
                (lines.as_slice(), &[] as &[&str])
            }
        };

        let toml_string = toml_lines.join("\n");
        let mut parsed_toml = toml::from_str::<EditorTemplate>(&toml_string)?;

        // Join content lines back together, preserving original line breaks
        if !content_lines.is_empty() {
            parsed_toml.content = content_lines.join("\n");
        }

        Ok(parsed_toml)
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_parse_template() {
        let template = r#"tags = ["work", "important"]
#tags = []
date = "today"
#date = "YYYY-MM-DD"
+++
Some content"#
            .to_string();

        let parsed = template.parse_template().unwrap();

        assert_eq!(parsed.tags.len(), 2);
        assert_eq!(parsed.date, DateSource::Today);
        assert_eq!(parsed.content, "Some content");
    }

    #[test]
    fn test_parse_template_no_content() {
        let template = r#"tags = ["work", "important"]
#tags = []
date = "today"
#date = "YYYY-MM-DD"
+++"#
            .to_string();

        let parsed = template.parse_template().unwrap();

        assert_eq!(parsed.tags.len(), 2);
        assert_eq!(parsed.date, DateSource::Today);
        assert_eq!(parsed.content, "");
    }

    #[test]
    fn test_parse_template_no_tags() {
        let template = r#"date = "today"
#date = "YYYY-MM-DD"
+++
Some content"#
            .to_string();

        let parsed = template.parse_template().unwrap();

        assert_eq!(parsed.tags.len(), 0);
        assert_eq!(parsed.date, DateSource::Today);
        assert_eq!(parsed.content, "Some content");
    }

    #[test]
    fn test_parse_template_no_date() {
        let template = r#"tags = ["work", "important"]
#tags = []
+++
Some content"#
            .to_string();

        let parsed = template.parse_template().unwrap();

        assert_eq!(parsed.tags.len(), 2);
        assert_eq!(parsed.date, DateSource::Today);
        assert_eq!(parsed.content, "Some content");
    }

    #[test]
    fn test_parse_template_with_plus_in_content() {
        // This is the critical bug fix test - content with +++ should not break parsing
        let template = r#"tags = ["programming"]
date = "today"
+++
Learning C+++ today
Some more content with +++ in the middle
And even more +++ here"#
            .to_string();

        let parsed = template.parse_template().unwrap();

        assert_eq!(parsed.tags.len(), 1);
        assert!(parsed.tags.contains("programming"));
        assert_eq!(parsed.date, DateSource::Today);
        assert_eq!(
            parsed.content,
            "Learning C+++ today\nSome more content with +++ in the middle\nAnd even more +++ here"
        );
    }

    #[test]
    fn test_parse_template_multiline_content() {
        let template = r#"tags = ["work"]
+++
Line 1
Line 2
Line 3"#
            .to_string();

        let parsed = template.parse_template().unwrap();

        assert_eq!(parsed.content, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_parse_template_delimiter_with_whitespace() {
        // Delimiter can have whitespace around it
        let template = r#"tags = ["work"]
   +++
Content here"#
            .to_string();

        let parsed = template.parse_template().unwrap();

        assert_eq!(parsed.content, "Content here");
    }

    #[test]
    fn test_parse_template_no_delimiter() {
        // If no delimiter, entire input is TOML (no content)
        let template = r#"tags = ["work"]
date = "today""#
            .to_string();

        let parsed = template.parse_template().unwrap();

        assert_eq!(parsed.tags.len(), 1);
        assert_eq!(parsed.content, "");
    }

    #[test]
    fn test_format_error_header_escapes_special_chars() {
        // Simulate a TOML error with pipe symbols and special characters
        let error = anyhow::anyhow!("TOML parse error at line 2, column 3\n  |\n2 |   |\n  |   ^\ninvalid key");
        let original_content = r#"tags = ["work"
date = "today"
+++
My content"#;

        let formatted = Editor::format_error_header(&error, original_content);

        // Verify all error lines start with "# "
        let lines: Vec<&str> = formatted.lines().collect();
        assert!(lines[0].starts_with("# ====="));
        assert!(lines[1].starts_with("# "));  // Error message line

        // Verify original content is preserved after the error block
        assert!(formatted.contains("My content"));

        // Verify the formatted output can be parsed as valid TOML comments
        // (i.e., it won't crash the parser on the next attempt)
        assert!(formatted.contains("# TOML parse error"));
        assert!(formatted.contains("# invalid key"));
    }

    #[test]
    fn test_format_error_header_multiline_error() {
        let error = anyhow::anyhow!("Line 1 error\nLine 2 error\nLine 3 error");
        let content = "some content";

        let formatted = Editor::format_error_header(&error, content);

        // Each error line should be commented
        assert!(formatted.contains("# Line 1 error"));
        assert!(formatted.contains("# Line 2 error"));
        assert!(formatted.contains("# Line 3 error"));
        assert!(formatted.contains("some content"));
    }
}
