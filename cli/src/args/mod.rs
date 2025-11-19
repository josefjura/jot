use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::utils::{date_source::DateSource, date_target::DateTarget};

#[derive(Parser, Debug)]
#[command(
    name = "jot",
    version,
    about,
    long_about = "Simple CLI for jotting down notes"
)]
pub struct CliArgs {
    #[clap(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub config: ConfigArgs,
}

#[derive(Debug, Args, Serialize)]
pub struct ConfigArgs {
    /// Path to profile configuration file
    #[arg(long, short, env = "JOT_PROFILE")]
    pub profile_path: Option<String>,
}

#[derive(Debug, Subcommand, Serialize, PartialEq)]
pub enum Command {
    /// Prints out curent configuration
    Config,
    /// Initializes a new profile
    Init,
    /// Notes subcommands
    #[clap(subcommand)]
    Note(NoteCommand),
    /// Creates a new note. Alias for 'note add'.
    Down(NoteAddArgs),
}

#[derive(Debug, Subcommand, Serialize, PartialEq)]
pub enum NoteCommand {
    /// Creates a new note.
    Add(NoteAddArgs),
    /// Lists notes.
    Search(NoteSearchArgs),
    /// Get latest note.
    Last(NoteLatestArgs),
    /// Edit a note.
    Edit(NoteEditArgs),
    /// Delete a note (soft delete).
    Delete(NoteDeleteArgs),
}

#[derive(Debug, Args, Serialize, PartialEq)]
pub struct NoteAddArgs {
    /// Assign to current day
    #[arg(long, short, value_parser = parse_date_source, default_value_t = DateSource::Today)]
    pub date: DateSource,
    /// Note content
    #[arg(trailing_var_arg = true)]
    pub content: Vec<String>,
    /// Open in external editor for interactive editing
    #[arg(long, short, default_value_t = false)]
    pub interactive: bool,
    /// Filter by tags (can be specified multiple times or comma-separated)
    #[arg(long, value_name = "TAGS", value_delimiter = ',')]
    pub tag: Vec<String>,
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    Pretty,
    Plain,
    Json,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Pretty
    }
}

#[derive(Debug, clap::Args, PartialEq, Serialize, Deserialize)]
#[command(about = "Search and list notes")]
pub struct NoteSearchArgs {
    /// Search term to filter notes
    #[arg(default_value = None)]
    pub term: Option<String>,

    /// Filter by tags (can be specified multiple times or comma-separated)
    #[arg(long, value_name = "TAGS", value_delimiter = ',')]
    pub tag: Vec<String>,

    /// Filter by date (e.g., "today", "last week", "2024-03-16")
    #[arg(long, value_name = "DATE", value_parser = parse_date_target)]
    pub date: Option<DateTarget>,

    /// Number of lines to display for each note (default: full content)
    #[arg(long, value_name = "N")]
    pub lines: Option<usize>,

    /// Maximum number of results to return
    #[arg(long, short = 'l')]
    pub limit: Option<i64>,

    /// Output format (pretty, plain, or json)
    #[arg(long, value_enum, default_value_t = OutputFormat::Pretty)]
    pub output: OutputFormat,
}

#[derive(Debug, clap::Args, PartialEq, Serialize, Deserialize)]
#[command(about = "Retrieve the latest order")]
pub struct NoteLatestArgs {
    /// Search term to filter notes
    #[arg(default_value = None)]
    pub term: Option<String>,

    /// Filter by tags (can be specified multiple times or comma-separated)
    #[arg(long, value_name = "TAGS", value_delimiter = ',')]
    pub tag: Vec<String>,

    /// Output format (pretty, plain, or json)
    #[arg(long, value_enum, default_value_t = OutputFormat::Pretty)]
    pub output: OutputFormat,
}

impl Default for NoteSearchArgs {
    fn default() -> Self {
        Self {
            term: None,
            tag: vec![],
            date: None,
            lines: None,
            limit: None,
            output: OutputFormat::Pretty,
        }
    }
}

pub fn parse_date_target(s: &str) -> anyhow::Result<DateTarget> {
    return s.parse();
}

pub fn parse_date_source(s: &str) -> anyhow::Result<DateSource> {
    return s.parse();
}

#[derive(Debug, Args, Serialize, PartialEq)]
pub struct NoteEditArgs {
    /// Note ID to edit (if not provided, edits the most recent note)
    #[arg(value_name = "ID")]
    pub id: Option<String>,
}

#[derive(Debug, Args, Serialize, PartialEq)]
pub struct NoteDeleteArgs {
    /// Note ID(s) to delete (if not provided, deletes the most recent note)
    #[arg(value_name = "ID")]
    pub ids: Vec<String>,

    /// Skip confirmation prompt
    #[arg(long, short = 'y')]
    pub yes: bool,
}
