use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default)]
pub enum ReportingMode {
    /// Plain text
    #[default]
    Text,
    /// As a list of JSON objects
    Json,
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
    #[arg(short, long, default_value_t = 10)]
    pub block_threshold: usize,

    /// Set to increase the details that are output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Files to find the code blocks
    pub files: Vec<PathBuf>,

    /// How you want the information to be delivered
    ///
    /// Anything other than `ReportingMode::Text` will disable metadata reporting (e.g. reporting file
    /// information, verbosity, and other command line arguments, as well as the concluding remarks).
    #[arg(value_enum, long, default_value_t = ReportingMode::Text)]
    pub reporting_mode: ReportingMode,
}

impl Cli {
    pub fn print(&self) {
        if !self.verbose {
            return;
        }

        if self.reporting_mode != ReportingMode::Text {
            return;
        }

        eprint!("{} file(s)", self.files.len());
        if self.files.len() <= 10 {
            eprintln!(" {:?}", &self.files);
        } else {
            eprintln!(" {:?}...", &self.files[..10]);
        }

        eprintln!("Verbosity (-v): {}", self.verbose);
        eprintln!(
            "Comparison threshold (-t): {} ({})",
            self.lev_threshold,
            if self.lev_threshold > 0 {
                "Levenshtein distance"
            } else {
                "Strict equality"
            }
        );
        eprintln!(
            "Minimum length of first line before block consideration (-l): {}",
            self.line_threshold
        );
        eprintln!(
            "Minimum length of block before consideration (-b): {}",
            self.block_threshold
        );
    }
}
