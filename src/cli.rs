use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(name = "superdiff")]
#[command(version = "1.0.0")]
#[command(about = "Find duplicate code blocks", long_about = None)]
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

    /// File to find the code blocks (defaults to stdin)
    pub file: Option<PathBuf>,
}
