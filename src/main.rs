use clap::Parser;
use superdiff::*;

fn main() {
    let args = cli::Cli::parse();
    args.print();

    let matches = types::JsonRoot::from_matches(comp::get_all_matches(&args));

    printer::matches(&args, &matches);
    printer::conclusion(&args, &matches);
}
