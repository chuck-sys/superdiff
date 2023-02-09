use crate::cli::Cli;
use crate::printer;
use crate::types::{CompFile, ComparisonFn, Match, Matches, MatchesLookup};

use std::sync::mpsc;

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

/// Add or remove entries from lookup and matches based on given pair of matches.
///
/// There are 4 situations that we prepare for.
///
/// 1. **The pair of matches both exist in the same bucket.** We don't need to do anything.
/// 2. **The pair of matches both exist in different buckets.** We combine both buckets.
/// 3. **One of the pair of matches exist in a bucket.** We put the other one in that bucket.
/// 4. **None of the matches exist in any bucket.** We put one of them in a bucket labelled with
///    the other.
pub fn update_matches(
    (a, b): (Match, Match),
    (where_is_match, matches_hash): (&mut MatchesLookup, &mut Matches),
) {
    let mut where_is_match_to_insert = Vec::new();
    let (refa, refb) = (where_is_match.0.get(&a), where_is_match.0.get(&b));
    match (refa, refb, &a, &b) {
        (Some(refa), Some(refb), _, _) if refa != refb => {
            // Reassign all of refb references to refa
            let mut refb_v = matches_hash.0.remove(refb).unwrap();
            refb_v.push(refb.clone());
            for block in &refb_v {
                where_is_match_to_insert.push((block.clone(), refa.clone()));
            }

            // Append all of the refb into refa's bucket
            matches_hash
                .0
                .entry(refa.clone())
                .and_modify(|v| v.append(&mut refb_v));
        }
        (Some(refblock), None, _, b) | (None, Some(refblock), b, _) => {
            matches_hash
                .0
                .entry(refblock.clone())
                .and_modify(|v| v.push(b.clone()));

            where_is_match_to_insert.push((b.clone(), refblock.clone()));
        }
        (None, None, a, b) => {
            matches_hash.0.insert(b.clone(), vec![a.clone()]);

            where_is_match_to_insert.push((a.clone(), b.clone()));
            where_is_match_to_insert.push((b.clone(), b.clone()));
        }
        _ => {}
    }

    for (key, val) in where_is_match_to_insert {
        where_is_match.0.insert(key, val);
    }
}

pub fn get_matches_from_2_files(
    args: &Cli,
    tx: &mpsc::Sender<(Match, Match)>,
    comp: &ComparisonFn,
    (mut f1, mut f2): (CompFile, CompFile),
) {
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

                let matches = Match::from_compfiles(&f1, &f2, block_length);
                tx.send(matches).unwrap_or(());

                f2.start += block_length;
                max_block_length = std::cmp::max(max_block_length, block_length);
            } else {
                f2.start += 1;
            }
        }

        f1.start += max_block_length;
    }
}

/// Make a `Vec<char>`.
///
/// We use a preallocated `Vec` instead of `.collect()` to avoid allocation penalties.
fn to_char_vec(s: &str) -> Vec<char> {
    let mut v = Vec::with_capacity(s.len());

    for c in s.chars() {
        v.push(c);
    }

    v
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
#[allow(clippy::needless_range_loop)]
pub fn levenshtein_distance(x: &str, y: &str, threshold: usize) -> usize {
    let (x, y) = (to_char_vec(x), to_char_vec(y));
    let (m, n) = (x.len(), y.len());
    let mut d = vec![0usize; (m + 1) * (n + 1)];
    let size = m + 1;

    // Distance is at most the length of the longer string
    if threshold >= std::cmp::max(m, n) {
        return threshold;
    }

    // Distance is at least the absolute value of the difference in sizes of the two strings
    if threshold < m.abs_diff(n) {
        return threshold + 1;
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
    use super::levenshtein_distance;

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
        check_lev!("the same the same", "the same the same", 10, 0);
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
