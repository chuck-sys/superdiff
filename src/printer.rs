use crate::cli::{Cli, ReportingMode};
use crate::math::combinations;
use crate::types::{CompFile, JsonRoot};

use std::path::PathBuf;

fn truncate_from_right(s: &String) -> String {
    let chars = s.chars();
    let size = chars.clone().count();
    if size <= 30 {
        return s.to_owned();
    }

    format!("...{}", chars.skip(size - 27).collect::<String>())
}

fn truncate_path(p: PathBuf) -> String {
    truncate_from_right(&p.into_os_string().to_string_lossy().into_owned())
}

pub fn now_comparing(args: &Cli, f1: &CompFile, f2: &CompFile) {
    if args.verbose && args.reporting_mode == ReportingMode::Text {
        if f1.file == f2.file {
            eprint!(
                "\rNow comparing '{}' ({:>4}/{:>4})",
                truncate_path(f1.file.clone()),
                f1.start,
                f1.lines.len()
            );
        } else {
            eprint!(
                "\rNow comparing {:?} and {:?} ({:>4}/{:>4})",
                truncate_path(f1.file.clone()),
                truncate_path(f2.file.clone()),
                f1.start,
                f1.lines.len()
            );
        }
    }
}

pub fn done_comparison(args: &Cli, nth: usize) {
    if args.verbose && args.reporting_mode == ReportingMode::Text {
        let total = combinations(args.files.len(), 2) + args.files.len();
        eprintln!("...done {nth}/{total}");
    }
}

pub fn skip_comparison(args: &Cli, f1: &PathBuf, f2: &PathBuf) {
    if args.verbose && args.reporting_mode == ReportingMode::Text {
        if f1 == f2 {
            eprintln!("Unable to open {} for reading", truncate_path(f1.clone()));
        } else {
            eprintln!("Unable to open {} and {} for reading", truncate_path(f1.clone()), truncate_path(f2.clone()));
        }
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
    if args.reporting_mode == ReportingMode::Text && args.verbose {
        eprintln!(
            "A total of {} unique match(es) were found in the {} file(s).",
            matches.unique_matches(),
            args.files.len()
        );
    }
}
