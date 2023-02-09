use std::thread;
use std::sync::mpsc;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::cli::Cli;
use crate::comp::{get_matches_from_2_files, comparison_lambda, update_matches};
use crate::types::{MatchesLookup, Matches, CompFile, JsonRoot};

pub struct ThreadPool {
    where_is_match: MatchesLookup,
    matches_hash: Matches,
    args: Cli,
}

impl ThreadPool {
    fn partition_file_combinations(&self) -> Vec<mpsc::Receiver<(PathBuf, PathBuf)>> {
        let mut senders = Vec::with_capacity(self.args.worker_threads);
        let mut receivers = Vec::with_capacity(self.args.worker_threads);

        for _ in 0..self.args.worker_threads {
            let (tx, rx) = mpsc::channel();
            senders.push(tx);
            receivers.push(rx);
        }

        let mut sender_index = 0;
        for i in 0..self.args.files.len() {
            for j in i..self.args.files.len() {
                let two_files = (self.args.files[i].clone(), self.args.files[j].clone());
                senders[sender_index].send(two_files).unwrap();

                if sender_index == senders.len() - 1 {
                    sender_index = 0;
                } else {
                    sender_index += 1;
                }
            }
        }

        receivers
    }

    pub fn run_and_get_results(&mut self) -> JsonRoot {
        let (tx, rx) = mpsc::channel();

        for two_file_rx in self.partition_file_combinations() {
            let tx = tx.clone();
            let args = self.args.clone();
            thread::spawn(move || {
                let comp = comparison_lambda(&args);
                for (f1, f2) in two_file_rx {
                    if let Some(two_files) = CompFile::from_files(&f1, &f2) {
                        get_matches_from_2_files(&args, &tx, &comp, two_files);
                    }
                }
            });
        }

        drop(tx);

        for matches in rx {
            update_matches(matches, (&mut self.where_is_match, &mut self.matches_hash));
        }

        JsonRoot::from(&self.matches_hash)
    }
}

impl From<Cli> for ThreadPool {
    fn from(item: Cli) -> Self {
        Self {
            where_is_match: MatchesLookup(HashMap::new()),
            matches_hash: Matches(HashMap::new()),
            args: item,
        }
    }
}
