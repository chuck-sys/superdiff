use clap::Parser;

mod cli;
mod comp;
mod math;

fn print_matches(args: &cli::Cli, matches: &comp::FlattenedMatches) {
    match args.reporting_mode {
        cli::ReportingMode::JSON => {
            println!("{}", matches.to_json_string());
        }
        cli::ReportingMode::Text => {
            println!("{}", matches.to_friendly_string());
        }
    }
}

fn print_conclusion(args: &cli::Cli, matches: &comp::FlattenedMatches) {
    match args.reporting_mode {
        cli::ReportingMode::Text => {
            if args.verbose {
                println!(
                    "A total of {} unique match(es) were found in the {} file(s).",
                    matches.unique_matches(),
                    args.files.len()
                );
            }
        }
        _ => {}
    }
}

fn main() {
    let args = cli::Cli::parse();
    args.print();

    let matches = comp::FlattenedMatches::from_matches(comp::get_all_matches(&args));
    print_matches(&args, &matches);
    print_conclusion(&args, &matches);
}
