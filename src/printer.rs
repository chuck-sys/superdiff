use crate::cli::{Cli, ReportingMode};
use crate::math::combinations;
use crate::types::JsonRoot;

use std::thread;
use std::sync::mpsc;

/// Spawn a thread that prints progress Text
///
/// Will not spawn any thread at all if `verbose` is not set.
pub fn spawn_processing_text(args: &Cli, rx: mpsc::Receiver<bool>) {
    if !args.verbose {
        return;
    }

    let total = combinations(args.files.len(), 2) + args.files.len();
    thread::spawn(move || {
        let mut i = 1usize;
        for _ in rx {
            let percentage = i * 100 / total;
            eprint!("{percentage}% completed\r");

            i += 1;
        }
        eprintln!();
    });
}

pub fn done_comparison(args: &Cli, nth: usize) {
    if args.verbose && args.reporting_mode == ReportingMode::Text {
        let total = combinations(args.files.len(), 2) + args.files.len();
        eprintln!("...done {nth}/{total}");
    }
}

pub fn matches(args: &Cli, matches: &JsonRoot) {
    match args.reporting_mode {
        ReportingMode::Json => {
            println!("{}", matches.json());
        }
        ReportingMode::Text => {
            println!("{matches}");
        }
    }
}

pub fn conclusion(args: &Cli, matches: &JsonRoot) {
    if args.verbose {
        eprintln!(
            "A total of {} unique match(es) were found in the {} file(s).",
            matches.unique_matches(),
            args.files.len()
        );
    }
}
