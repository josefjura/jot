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
    /// Profile name to use
    #[arg(long, short, env = "JOT_PROFILE")]
    pub profile: Option<String>,
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum Command {
    /// Prints out current configuration
    Config,
    /// Profile management (defaults to showing current profile)
    Profile {
        #[clap(subcommand)]
        command: Option<ProfileCommand>,
    },
    /// Notes subcommands
    #[clap(subcommand)]
    Note(NoteCommand),
    /// Creates a new note. Alias for 'note add'.
    Down(NoteAddArgs),
    /// Search notes. Alias for 'note search'.
    #[clap(name = "ls")]
    List(NoteSearchArgs),
    /// Generate shell completion scripts
    Completion {
        /// Shell type
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Debug, Subcommand, Serialize, PartialEq)]
pub enum ProfileCommand {
    /// Switch to a profile (creates it if it doesn't exist)
    Use { name: String },
    /// List all available profiles
    List,
    /// Show current active profile
    Current,
}

#[derive(Debug, Subcommand, Serialize, PartialEq)]
pub enum NoteCommand {
    /// Creates a new note.
    Add(NoteAddArgs),
    /// Search notes.
    #[clap(visible_alias = "ls")]
    Search(NoteSearchArgs),
    /// Get latest note.
    #[clap(visible_alias = "latest")]
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
    #[arg(long, short = 'e', default_value_t = false)]
    pub editor: bool,
    /// Add tags to note (can be specified multiple times or comma-separated)
    #[arg(long, short = 't', value_name = "TAGS", value_delimiter = ',')]
    pub tag: Vec<String>,
    /// Quiet mode: only output the note ID
    #[arg(long, short = 'q', default_value_t = false)]
    pub quiet: bool,
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Serialize, Deserialize, Default)]
pub enum OutputFormat {
    #[default]
    Pretty,
    Plain,
    Json,
    /// Output only note IDs (one per line)
    Id,
}

#[derive(Debug, clap::Args, PartialEq, Serialize, Deserialize)]
#[command(about = "Search and list notes")]
pub struct NoteSearchArgs {
    /// Search term to filter notes
    #[arg(default_value = None)]
    pub term: Option<String>,

    /// Filter by tags (can be specified multiple times or comma-separated)
    #[arg(long, short = 't', value_name = "TAGS", value_delimiter = ',')]
    pub tag: Vec<String>,

    /// Filter by date (e.g., "today", "last week", "2024-03-16")
    #[arg(long, value_name = "DATE", value_parser = parse_date_target)]
    pub date: Option<DateTarget>,

    /// Number of lines to display for each note (default: full content)
    #[arg(long, short = 'L', value_name = "N")]
    pub lines: Option<usize>,

    /// Maximum number of results to return
    #[arg(long, short = 'n')]
    pub limit: Option<i64>,

    /// Output format (pretty, plain, or json)
    #[arg(long, value_enum, default_value_t = OutputFormat::Pretty)]
    pub output: OutputFormat,
}

#[derive(Debug, clap::Args, PartialEq, Serialize, Deserialize)]
#[command(about = "Retrieve the latest note")]
pub struct NoteLatestArgs {
    /// Search term to filter notes
    #[arg(default_value = None)]
    pub term: Option<String>,

    /// Filter by tags (can be specified multiple times or comma-separated)
    #[arg(long, short = 't', value_name = "TAGS", value_delimiter = ',')]
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
    s.parse()
}

pub fn parse_date_source(s: &str) -> anyhow::Result<DateSource> {
    s.parse()
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
