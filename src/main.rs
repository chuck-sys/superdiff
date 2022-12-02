use clap::Parser;
use std::io;

mod cli;
mod comp;

fn print_arguments(args: &cli::Cli) {
    if args.verbose == 0 {
        return;
    }

    println!("{} files", args.files.len());

    println!("Verbosity (-v): {}", args.verbose);
    println!("Comparison threshold (-t): {} ({})",
                args.lev_threshold,
                if args.lev_threshold > 0 { "Levenshtein distance" } else { "Strict equality" });
    println!("Minimum length of first line before block consideration (-l): {}", args.line_threshold);
    println!("Minimum length of block before consideration (-b): {}", args.block_threshold);
}

type FlattenedMatches = Vec<Vec<comp::Match>>;

fn flatten_matches(m: comp::Matches) -> FlattenedMatches {
    let mut ret = vec![];

    for (initial_match, mut other_matches) in m {
        other_matches.insert(0, initial_match);
        ret.push(other_matches);
    }

    ret
}

fn vecmatches_to_json_string(v: &Vec<comp::Match>) -> String {
    format!("[{}]", v.iter().map(|x| x.to_json_string()).collect::<Vec<String>>().join(", "))
}

fn flattened_matches_to_string(args: &cli::Cli, m: FlattenedMatches) -> String {
    match args.reporting_mode {
        cli::ReportingMode::JSON => {
            format!("[ {} ]",
                m.iter()
                .map(vecmatches_to_json_string)
                .collect::<Vec<String>>()
                .join(", "))
        },
        cli::ReportingMode::Text => {
            format!("")
        },
    }
}

fn main() -> io::Result<()> {
    let args = cli::Cli::parse();

    if args.reporting_mode == cli::ReportingMode::Text {
        print_arguments(&args);
    }

    let matches = flatten_matches(comp::get_all_matches(&args));
    println!("{}", flattened_matches_to_string(&args, matches));

    Ok(())
}
