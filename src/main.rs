use clap::Parser;

mod cli;
mod comp;
mod math;
mod printer;
mod types;

use types::JsonRoot;

fn main() {
    let args = cli::Cli::parse();
    args.print();

    let matches = JsonRoot::from_matches(comp::get_all_matches(&args));

    printer::matches(&args, &matches);
    printer::conclusion(&args, &matches);
}
