use clap::Parser;
use superdiff::*;

fn main() {
    let mut args = cli::Cli::parse();
    if args.files_from_stdin() {
        args.populate_files_from_stdin();
    }
    args.print();

    let mut pool = threadpool::ThreadPool::from(args.clone());
    let matches = pool.run_and_get_results();

    printer::matches(&args, &matches);
    printer::conclusion(&args, &matches);
}
