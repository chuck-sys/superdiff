use clap::Parser;
use std::io;
use std::path::PathBuf;
use std::fs;

mod cli;
mod comp;

fn get_lines_from_file(file: &PathBuf) -> io::Result<Vec<String>> {
    Ok(fs::read_to_string(file)?
        .split("\n")
        .map(|line| line.to_string())
        .collect::<Vec<String>>())
}

fn get_lines_from_stdin() -> io::Result<Vec<String>> {
    Ok(io::stdin()
        .lines()
        .map(|line| line.unwrap().to_string())
        .collect::<Vec<String>>())
}

fn get_lines(args: &cli::Cli) -> io::Result<Vec<String>> {
    match args.file {
        Some(ref file) => get_lines_from_file(file),
        None => get_lines_from_stdin(),
    }
}

fn print_arguments(args: &cli::Cli, lines: &Vec<String>) {
    if args.verbose == 0 {
        return;
    }

    if let Some(ref file) = args.file {
        println!("File: {:?} ({} lines)", file, lines.len());
    } else {
        println!("Standard input ({} lines)", lines.len());
    }

    println!("Verbosity (-v): {}", args.verbose);
    println!("Comparison threshold (-t): {} ({})",
                args.lev_threshold,
                if args.lev_threshold > 0 { "Levenshtein distance" } else { "Strict equality" });
    println!("Minimum length of first line before block consideration (-l): {}", args.line_threshold);
    println!("Minimum length of block before consideration (-b): {}", args.block_threshold);
}

fn main() -> io::Result<()> {
    let args = cli::Cli::parse();
    let original_lines = get_lines(&args)?;
    let trimmed_lines = original_lines.iter().map(|line| line.trim().to_string()).collect();

    if args.reporting_mode == cli::ReportingMode::Text {
        print_arguments(&args, &trimmed_lines);
    }

    let blocks = comp::global_compare_lines(&args, &trimmed_lines);
    comp::print_blocks(&args, &blocks, &original_lines);

    if args.reporting_mode == cli::ReportingMode::Text {
        comp::print_ending_status(&args, &blocks);
    }

    Ok(())
}
