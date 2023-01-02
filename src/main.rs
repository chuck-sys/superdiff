use clap::Parser;
use superdiff::*;

fn main() {
    let mut args = cli::Cli::parse();
    if args.files_from_stdin() {
        args.populate_files_from_stdin();
    }
    args.print();

    let matches = types::JsonRoot::from(comp::get_all_matches(&args));

    printer::matches(&args, &matches);
    printer::conclusion(&args, &matches);
}
