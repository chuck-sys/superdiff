use crate::cli::{Cli, ReportingMode};
use crate::math::combinations;

use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;

/// A structure to easily move parameters from one place to another.
#[derive(Clone, Debug)]
struct CompFile<'a> {
    file: PathBuf,
    lines: &'a Vec<String>,
    start: usize,
}

/// A matching block. Doesn't include the text.
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Match {
    pub file: PathBuf,
    pub line: usize,
    pub size: usize,
}

pub struct Matches(HashMap<Match, Vec<Match>>);
pub struct MatchesLookup(HashMap<Match, Match>);
pub struct FlattenedMatch(Vec<Match>);
pub struct FlattenedMatches(Vec<FlattenedMatch>);

impl FlattenedMatch {
    pub fn from_kv_matches(initial_match: Match, mut other_matches: Vec<Match>) -> Self {
        other_matches.insert(0, initial_match);
        FlattenedMatch(other_matches)
    }

    pub fn to_json_string(&self) -> String {
        format!(
            "[{}]",
            self.0
                .iter()
                .map(|x| x.to_json_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }

    pub fn to_friendly_string(&self) -> String {
        format!(
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

    pub fn to_json_string(&self) -> String {
        format!(
            "[{}]",
            self.0
                .iter()
                .map(|x| x.to_json_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }

    pub fn to_friendly_string(&self) -> String {
        self.0
            .iter()
            .map(|x| x.to_friendly_string())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

type ComparisonFn = Box<dyn Fn(&String, &String) -> bool>;

const INSERTION_COST: usize = 1;
const DELETION_COST: usize = 1;
const SUBSTITUTION_COST: usize = 1;

impl Match {
    pub fn to_json_string(&self) -> String {
        // Use 1-based indexing to count line numbers.
        format!(
            "{{ \"file\": \"{}\", \"line\": {}, \"size\": {} }}",
            self.file.display(),
            self.line + 1,
            self.size
        )
    }
}

impl<'a> CompFile<'a> {
    fn current_line(&'a self) -> &String {
        &self.lines[self.start]
    }
}

/// Create a comparison function based on the given threshold.
///
/// If the threshold is 0, we use string comparison. If not, we use Levenshtein distance.
pub fn comparison_lambda(args: &Cli) -> ComparisonFn {
    let threshold = args.lev_threshold;
    if threshold == 0 {
        Box::new(move |x, y| x == y)
    } else {
        Box::new(move |x, y| levenshtein_distance(x, y, threshold) <= threshold)
    }
}

/// Find block length of the matching code block.
///
/// Stops comparison when we reach the end of the file, or if the files are the same and the
/// original index hits the occurrance index. This stops code blocks from "eating" the other code
/// block (i.e. no nested overlapping blocks that are similar).
fn get_max_block_size(comp: &ComparisonFn, f1: &CompFile, f2: &CompFile) -> usize {
    let mut block_length = 1;

    loop {
        let i1 = f1.start + block_length;
        let i2 = f2.start + block_length;

        if f1.file == f2.file && i1 == f2.start {
            return block_length;
        }

        if i1 >= f1.lines.len() || i2 >= f2.lines.len() {
            return block_length;
        }

        if comp(&f1.lines[i1], &f2.lines[i2]) {
            block_length += 1;
        } else {
            return block_length;
        }
    }
}

fn get_matches_from_2_files(
    args: &Cli,
    mut where_is_match: MatchesLookup,
    mut matches_hash: Matches,
    comp: &ComparisonFn,
    mut f1: CompFile,
    mut f2: CompFile,
) -> (MatchesLookup, Matches) {
    f1.start = 0;

    while f1.start < f1.lines.len() {
        if args.verbose && args.reporting_mode == ReportingMode::Text {
            eprint!(
                "\rNow comparing {:?} and {:?} ({:>4}/{:>4})",
                &f1.file,
                &f2.file,
                f1.start,
                f1.lines.len()
            );
        }

        // Don't consider line lengths below the threshold
        if f1.current_line().len() < args.line_threshold {
            f1.start += 1;
            continue;
        }

        f2.start = if f1.file == f2.file { f1.start + 1 } else { 0 };
        let mut max_block_length = 1;

        while f2.start < f2.lines.len() {
            if comp(f1.current_line(), f2.current_line()) {
                let block_length = get_max_block_size(comp, &f1, &f2);

                if block_length < args.block_threshold {
                    f2.start += block_length;
                    continue;
                }

                let original_block = Match {
                    file: f1.file.clone(),
                    line: f1.start,
                    size: block_length,
                };
                let k = match where_is_match.0.get(&original_block) {
                    Some(x) => x,
                    None => &original_block,
                };
                let matching_block = Match {
                    file: f2.file.clone(),
                    line: f2.start,
                    size: block_length,
                };
                if matches_hash.0.contains_key(k) {
                    matches_hash
                        .0
                        .get_mut(k)
                        .unwrap()
                        .push(matching_block.clone());
                } else {
                    matches_hash
                        .0
                        .insert(original_block.clone(), vec![matching_block.clone()]);
                    where_is_match
                        .0
                        .insert(original_block.clone(), original_block.clone());
                }
                where_is_match.0.insert(matching_block, original_block);

                f2.start += block_length;
                max_block_length = std::cmp::max(max_block_length, block_length);
            } else {
                f2.start += 1;
            }
        }

        f1.start += max_block_length;
    }

    (where_is_match, matches_hash)
}

fn get_lines_from_file(file: &PathBuf) -> std::io::Result<Vec<String>> {
    Ok(std::fs::read_to_string(file)?
        .split('\n')
        .map(|line| line.trim().to_string())
        .collect::<Vec<String>>())
}

/// Load all files into memory at a time.
///
/// May not work well if you have an extremely large codebase.
///
/// FIXME Do analysis on which is faster: initially loading everything into memory or loading on
///       the go. Stick with the one that is more efficient.
fn get_all_file_contents(args: &Cli) -> HashMap<&PathBuf, Vec<String>> {
    let mut contents = HashMap::new();

    for f in &args.files {
        match get_lines_from_file(f) {
            Ok(lines) => {
                contents.insert(f, lines);
            }
            Err(e) => {
                println!("file read error ('{}'): {}", f.display(), e);
            }
        }
    }

    contents
}

/// Pair each file with the other and get groups of matches.
fn match_with_others(
    args: &Cli,
    comp: &ComparisonFn,
    contents: &HashMap<&PathBuf, Vec<String>>,
) -> Matches {
    let mut where_is_match = MatchesLookup(HashMap::new());
    let mut matches_hash = Matches(HashMap::new());
    let total_combinations = combinations(args.files.len(), 2) + args.files.len();

    for (i, combo) in args
        .files
        .iter()
        .combinations_with_replacement(2)
        .enumerate()
    {
        let lines1 = contents.get(&combo[0]);
        let lines2 = contents.get(&combo[1]);

        if let (Some(lines1), Some(lines2)) = (lines1, lines2) {
            let f1 = CompFile {
                file: combo[0].clone(),
                start: 0,
                lines: lines1,
            };
            let f2 = CompFile {
                file: combo[1].clone(),
                start: 0,
                lines: lines2,
            };

            (where_is_match, matches_hash) =
                get_matches_from_2_files(args, where_is_match, matches_hash, comp, f1, f2);

            if args.verbose && args.reporting_mode == ReportingMode::Text {
                eprintln!("...done {} out of {}", i + 1, total_combinations);
            }
        }
    }

    matches_hash
}

/// Get all groups of matches in the given files.
pub fn get_all_matches(args: &Cli) -> Matches {
    let comp = comparison_lambda(args);
    let contents = get_all_file_contents(args);

    match_with_others(args, &comp, &contents)
}

/// Compute the edit distance of 2 strings, with shortcuts.
///
/// Modified from wikipedia pseudocode for matrix approach (no recursion).
///
/// For strings x and y with length m and n respectively, we create an m+1 by n+1 matrix
/// (represented by 1d array) of costs where moving to the right constitutes as inserting a
/// character from y; moving down constitutes as deleting a character from y; moving diagonally
/// across constitutes as substituting a character from y into a.
///
/// We stop computing if we find that nothing of our current row is under the threshold, in which
/// case we would exit early.
///
/// We can also stop computing if we know that the threshold is greater than m + n, which is the
/// maximum.
///
/// This algorithm runs at a time complexity of O(mn).
pub fn levenshtein_distance(x: &str, y: &str, threshold: usize) -> usize {
    let (x, y): (Vec<char>, Vec<char>) = (x.chars().collect(), y.chars().collect());
    let (m, n) = (x.len(), y.len());
    let mut d = vec![0usize; (m + 1) * (n + 1)];
    let size = m + 1;

    if threshold >= m + n {
        return threshold;
    }

    for i in 1..(m + 1) {
        d[i] = i;
    }

    for j in 1..(n + 1) {
        d[j * size] = j;
    }

    for j in 1..(n + 1) {
        let mut has_changed_row = false;

        for i in 1..(m + 1) {
            let sub_cost = if x[i - 1] == y[j - 1] {
                0
            } else {
                SUBSTITUTION_COST
            };
            d[i + j * size] = std::cmp::min(
                d[(i - 1) + j * size] + INSERTION_COST,
                std::cmp::min(
                    d[i + (j - 1) * size] + DELETION_COST,
                    d[(i - 1) + (j - 1) * size] + sub_cost,
                ),
            );

            if d[i + j * size] <= threshold {
                has_changed_row = true;
            }
        }

        // Guarantee to not pass the threshold check
        if !has_changed_row {
            return threshold + 1;
        }
    }

    d[m + n * size]
}

#[cfg(test)]
mod tests {
    use crate::comp::levenshtein_distance;

    macro_rules! check_lev {
        ( $a:literal, $b:literal, $t:literal ) => {{
            check_lev!($a, $b, $t, $t);
        }};

        ( $a:literal, $b:literal, $t:literal, $e:literal ) => {{
            let dist = levenshtein_distance(&$a.to_string(), &$b.to_string(), $t);
            assert_eq!(
                dist, $e,
                "levenshtein_distance({}, {}, {}) = {}, expected {}",
                $a, $b, $t, dist, $e
            );
        }};
    }

    #[test]
    fn test_lev_distance() {
        // Normal use of function
        check_lev!("kitten", "sitting", 3);
        check_lev!("train", "shine", 4);
        check_lev!("a", "aaa", 2);
        // Maximum threshold
        check_lev!("arst", "zxcv", 4);
        // Short circuit at the end
        check_lev!("ieanrstien", "            ", 5, 6);
        // Short circuit at the start
        check_lev!("arstarst", "zxcv", 100, 100);
    }
}
