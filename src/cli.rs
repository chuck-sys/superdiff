use clap::{Parser, ValueEnum};
use std::io;
use std::path::PathBuf;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default)]
pub enum ReportingMode {
    /// Plain text
    #[default]
    Text,
    /// As a list of JSON objects
    ///
    /// Can be piped into `jq` for querying or loaded into `vim` with the plugin.
    Json,
}

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Levenshtein distance threshold (0 uses string comparison)
    #[arg(short = 't', long, default_value_t = 0)]
    pub lev_threshold: usize,

    /// Length of line before initial consideration
    ///
    /// If the length of the line does not reach this threshold, skip over it. For example, if you
    /// don't want a match to start on an empty line, use a positive integer. If you want the match
    /// to not start on lines that only have a single curly brace, use a positive integer that is
    /// greater than 1.
    ///
    /// Ignores whitespace.
    #[arg(short, long, default_value_t = 1)]
    pub line_threshold: usize,

    /// Minimum length of a matching block (cluster of lines)
    #[arg(short, long, default_value_t = 10)]
    pub block_threshold: usize,

    /// Set to increase the details that are output
    ///
    /// Output preamble on the command line options that are specified, as well as a progress bar.
    /// Also output a conclusion at the end.
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Number of worker threads to spawn
    ///
    /// Comparisons between files are distributed evenly across all worker threads. This may not
    /// result in good caching efficiency because there is a possibility that all of the
    /// comparisons are on different files. It probably all comes down to your operating system.
    ///
    /// Thus, this option doesn't do anything substantial if you are only working with 1 file.
    #[arg(long, default_value_t = 1)]
    pub worker_threads: usize,

    /// Files to find the code blocks (leave empty to read from stdin)
    pub files: Vec<PathBuf>,

    /// How you want the information to be delivered
    ///
    /// Anything other than `ReportingMode::Text` will disable metadata reporting (e.g. reporting file
    /// information, verbosity, and other command line arguments, as well as the concluding remarks).
    #[arg(value_enum, long, default_value_t = ReportingMode::Text)]
    pub reporting_mode: ReportingMode,
}

impl Cli {
    pub fn populate_files_from_stdin(&mut self) {
        let mut files: Vec<PathBuf> = Vec::new();

        for line in io::stdin().lines() {
            match line {
                Ok(f) => files.push(f.into()),
                Err(e) => panic!("{e}"),
            }
        }

        self.files = files;
    }

    pub fn files_from_stdin(&self) -> bool {
        self.files.is_empty()
    }

    pub fn print(&self) {
        if !self.verbose {
            return;
        }

        eprint!("{} file(s)", self.files.len());
        if self.files.len() <= 10 {
            eprintln!(" {:?}", &self.files);
        } else {
            eprintln!(" {:?}...", &self.files[..10]);
        }

        eprintln!("Worker threads: {}", self.worker_threads);
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
