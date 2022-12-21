use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use itertools::Itertools;
use serde::Serialize;

/// A structure to easily move parameters from one place to another.
#[derive(Clone, Debug)]
pub struct CompFile {
    pub file: PathBuf,
    pub lines: Vec<String>,
    pub start: usize,
}

/// A matching block.
///
/// Points to a single block of lines in some file.
#[derive(Hash, PartialEq, Eq, Clone, Debug, Serialize)]
pub struct Match {
    pub file: PathBuf,
    pub line: usize,
    pub size: usize,
}

/// A bunch of Matches.
///
/// Consists of an original match that is deemed similar to a list of other matches. The match that
/// is the key is arbitrarily chosen and is fungible.
pub struct Matches(pub HashMap<Match, Vec<Match>>);

/// A lookup that points some arbitrary match to the key match.
///
/// Used to check which key some match belongs to, in order to insert into `Matches`.
pub struct MatchesLookup(pub HashMap<Match, Match>);

/// A bunch of matches that are deemed similar.
///
/// Basically, this is just the `Matches` structure flattened into a list.
#[derive(Serialize)]
pub struct FlattenedMatch(pub Vec<Match>);

#[derive(Serialize)]
pub struct FlattenedMatches(pub Vec<FlattenedMatch>);

impl FlattenedMatch {
    fn from_kv_matches(initial_match: Match, mut other_matches: Vec<Match>) -> Self {
        other_matches.insert(0, initial_match);
        FlattenedMatch(other_matches)
    }
}

impl fmt::Display for FlattenedMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "=== MATCH ===\n{}\n",
            self.0
                .iter()
                .group_by(|item| &item.file)
                .into_iter()
                .map(|(key, group)| {
                    let group: Vec<&Match> = group.collect();
                    format!(
                        "File: {:?}\nLines: {:?}\nSize: {}",
                        key,
                        group.iter().map(|x| x.line).collect::<Vec<usize>>(),
                        group[0].size,
                    )
                })
                .collect::<Vec<String>>()
                .join("\n---\n")
        )
    }
}

impl FlattenedMatches {
    pub fn from_matches(m: Matches) -> Self {
        FlattenedMatches(
            m.0.into_iter()
                .map(|(k, v)| FlattenedMatch::from_kv_matches(k, v))
                .collect(),
        )
    }

    pub fn unique_matches(&self) -> usize {
        self.0.len()
    }

    pub fn json(&self) -> String {
        serde_json::to_string(&self.0).unwrap_or("[]".to_owned())
    }
}

impl fmt::Display for FlattenedMatches {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

pub type ComparisonFn = Box<dyn Fn(&String, &String) -> bool>;
pub type FileCache = HashMap<PathBuf, Vec<String>>;

impl Match {
    pub fn from_compfiles(f1: &CompFile, f2: &CompFile, block_length: usize) -> (Self, Self) {
        (
            Match {
                file: f1.file.clone(),
                line: f1.start + 1,
                size: block_length,
            },
            Match {
                file: f2.file.clone(),
                line: f2.start + 1,
                size: block_length,
            },
        )
    }
}

fn get_lines_from_file(file: &PathBuf) -> std::io::Result<Vec<String>> {
    Ok(std::fs::read_to_string(file)?
        .split('\n')
        .map(|line| line.trim().to_owned())
        .collect::<Vec<String>>())
}

impl CompFile {
    pub fn current_line(&self) -> &String {
        &self.lines[self.start]
    }

    pub fn from_files(f1: &PathBuf, f2: &PathBuf, cache: &mut FileCache) -> Option<(Self, Self)> {
        match (cache.get(f1), cache.get(f2)) {
            (Some(lines1), Some(lines2)) => Some((
                CompFile {
                    file: f1.clone(),
                    lines: lines1.clone(),
                    start: 0,
                },
                CompFile {
                    file: f2.clone(),
                    lines: lines2.clone(),
                    start: 0,
                },
            )),
            (None, Some(_)) => CompFile::from_files(f2, f1, cache),
            (Some(_), None) | (None, None) => {
                if let Ok(lines) = get_lines_from_file(f2) {
                    cache.insert(f2.clone(), lines);
                    CompFile::from_files(f1, f2, cache)
                } else {
                    None
                }
            }
        }
    }
}
