use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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
#[derive(Hash, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
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

#[derive(Serialize, Clone, Deserialize, PartialEq, Eq, Debug)]
pub struct JsonFileInfo {
    pub count_blocks: usize,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct JsonBlockInfo {
    pub starting_line: usize,
    pub block_length: usize,
}

#[derive(Serialize, Deserialize, Eq, Debug)]
pub struct JsonMatch {
    pub files: HashMap<PathBuf, JsonFileInfo>,
    pub blocks: HashMap<PathBuf, Vec<JsonBlockInfo>>,
}

#[derive(Serialize, Deserialize, Eq, Debug)]
pub struct JsonRoot {
    pub version: String,
    pub files: HashMap<PathBuf, JsonFileInfo>,
    pub matches: Vec<JsonMatch>,
}

impl From<Match> for JsonBlockInfo {
    fn from(m: Match) -> Self {
        Self {
            starting_line: m.line,
            block_length: m.size,
        }
    }
}

impl PartialEq for JsonMatch {
    fn eq(&self, other: &Self) -> bool {
        if self.files != other.files || self.blocks.len() != other.blocks.len() {
            return false;
        }

        for (k, v) in &self.blocks {
            match other.blocks.get(k) {
                Some(other_v) => {
                    let (a_info, b_info): (HashSet<_>, HashSet<_>) =
                        (v.iter().collect(), other_v.iter().collect());
                    if a_info != b_info {
                        return false;
                    }
                }
                None => {
                    return false;
                }
            }
        }

        true
    }
}

impl From<(Match, Vec<Match>)> for JsonMatch {
    fn from((initial_match, other_matches): (Match, Vec<Match>)) -> Self {
        let mut blocks = HashMap::new();
        let mut files = HashMap::new();
        files.insert(initial_match.file.clone(), JsonFileInfo { count_blocks: 1 });
        blocks.insert(
            initial_match.file.clone(),
            vec![JsonBlockInfo::from(initial_match)],
        );

        for m in other_matches {
            let f = m.clone().file;
            files
                .entry(f.clone())
                .and_modify(|info| info.count_blocks += 1)
                .or_insert(JsonFileInfo { count_blocks: 1 });
            blocks
                .entry(f)
                .and_modify(|v| v.push(JsonBlockInfo::from(m.clone())))
                .or_insert(vec![JsonBlockInfo::from(m)]);
        }

        Self { files, blocks }
    }
}

impl fmt::Display for JsonMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "=== MATCH ===\n{}\n",
            self.blocks
                .iter()
                .map(|(filename, infos)| format!(
                    "File: {}\nLines: {:?}\nSize: {}",
                    filename.display(),
                    infos
                        .iter()
                        .map(|info| info.starting_line)
                        .collect::<Vec<usize>>(),
                    infos[0].block_length,
                ))
                .collect::<Vec<String>>()
                .join("\n---\n")
        )
    }
}

impl From<Matches> for JsonRoot {
    fn from(m: Matches) -> Self {
        let version = clap::crate_version!().to_owned();
        let matches: Vec<JsonMatch> = m.0.into_iter().map(JsonMatch::from).collect();
        let jm_files = matches.iter().map(|jm| jm.files.clone());
        let mut files: HashMap<PathBuf, JsonFileInfo> = HashMap::new();

        for jmf in jm_files {
            for (filename, info) in jmf {
                files
                    .entry(filename)
                    .and_modify(|v| v.count_blocks += info.count_blocks)
                    .or_insert(info);
            }
        }

        Self {
            version,
            files,
            matches,
        }
    }
}

impl JsonRoot {
    pub fn unique_matches(&self) -> usize {
        self.matches.len()
    }

    pub fn json(&self) -> String {
        serde_json::to_string(&self).unwrap_or("{}".to_owned())
    }
}

impl PartialEq for JsonRoot {
    fn eq(&self, other: &Self) -> bool {
        if self.matches.len() != other.matches.len() {
            return false;
        }

        for item in &self.matches {
            if !other.matches.contains(item) {
                return false;
            }
        }

        true
    }
}

impl fmt::Display for JsonRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.matches
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
            Self {
                file: f1.file.clone(),
                line: f1.start + 1,
                size: block_length,
            },
            Self {
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
                Self {
                    file: f1.clone(),
                    lines: lines1.clone(),
                    start: 0,
                },
                Self {
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
