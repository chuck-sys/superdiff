use std::path::PathBuf;
use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ReportingMode {
    /// Plain text
    Text,
    /// As a list of JSON objects
    JSON,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Levenshtein distance threshold (0 uses string comparison)
    #[arg(short = 't', long, default_value_t = 0)]
    pub lev_threshold: usize,

    /// Length of line before initial consideration
    #[arg(short, long, default_value_t = 1)]
    pub line_threshold: usize,

    /// Length of block (cluster of lines) before making comparisons
    #[arg(short, long, default_value_t = 2)]
    pub block_threshold: usize,

    /// Verbosity levels
    #[arg(short, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Files to find the code blocks
    pub files: Vec<PathBuf>,

    /// How you want the information to be delivered
    ///
    /// Anything other than `ReportingMode::Text` will disable metadata reporting (e.g. reporting file
    /// information, verbosity, and other command line arguments, as well as the concluding remarks).
    #[arg(value_enum, long, default_value_t = ReportingMode::Text)]
    pub reporting_mode: ReportingMode,
}
