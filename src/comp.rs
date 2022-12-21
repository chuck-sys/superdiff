use crate::cli::Cli;
use crate::types::{CompFile, Match, Matches, MatchesLookup, ComparisonFn, FileCache};
use crate::printer;

use itertools::Itertools;
use std::collections::HashMap;

const INSERTION_COST: usize = 1;
const DELETION_COST: usize = 1;
const SUBSTITUTION_COST: usize = 1;

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
    (mut where_is_match, mut matches_hash): (MatchesLookup, Matches),
    comp: &ComparisonFn,
    (mut f1, mut f2): (CompFile, CompFile),
) -> (MatchesLookup, Matches) {
    f1.start = 0;

    while f1.start < f1.lines.len() {
        printer::now_comparing(args, &f1, &f2);

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

                let (original_block, matching_block) =
                    Match::from_compfiles(&f1, &f2, block_length);
                let k = where_is_match
                    .0
                    .get(&original_block)
                    .unwrap_or(&original_block);

                if let Some(v) = matches_hash.0.get_mut(k) {
                    v.push(matching_block.clone());
                } else {
                    matches_hash
                        .0
                        .insert(original_block.clone(), vec![matching_block.clone()]);
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

/// Get all groups of matches in the given files.
pub fn get_all_matches(args: &Cli) -> Matches {
    let mut filecache = FileCache::new();
    let mut where_is_match = MatchesLookup(HashMap::new());
    let mut matches_hash = Matches(HashMap::new());
    let comp = comparison_lambda(args);

    for (i, combo) in args
        .files
        .iter()
        .combinations_with_replacement(2)
        .enumerate()
    {
        if let Some((f1, f2)) = CompFile::from_files(combo[0], combo[1], &mut filecache) {
            (where_is_match, matches_hash) =
                get_matches_from_2_files(args, (where_is_match, matches_hash), &comp, (f1, f2));

                printer::done_comparison(args, i + 1);
        }
    }

    matches_hash
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
            let dist = levenshtein_distance($a, $b, $t);
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
        check_lev!("the same", "the same", 10, 0);
        check_lev!("kitten", "sitting", 3);
        check_lev!("train", "shine", 4);
        check_lev!("a", "aaa", 2);
        // Maximum threshold
        check_lev!("arst", "zxcv", 4);
        // Short circuit at the end
        check_lev!("ieanrstien", "            ", 5, 6);
        // Short circuit at the start
        check_lev!("arstarst", "zxcv", 100, 100);
        // A bit tight
        check_lev!("the same", "the same", 0);
    }
}
