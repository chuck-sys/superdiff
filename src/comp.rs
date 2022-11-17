use std::collections::{HashMap, HashSet};
use indicatif::{ProgressBar, ProgressStyle};
use crate::cli::Cli;

#[derive(Hash, PartialEq, Eq, PartialOrd, Clone)]
pub struct Block {
    pub start: usize,
    pub size: usize,
}

impl Ord for Block {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.start, self.size).cmp(&(other.start, other.size))
    }
}

pub type BlockMap = HashMap<Block, Vec<usize>>;
pub type ComparisonFn = Box<dyn Fn(&String, &String) -> bool>;

const INSERTION_COST: usize = 1;
const DELETION_COST: usize = 1;
const SUBSTITUTION_COST: usize = 1;

/// Print similar/duplicate code blocks.
pub fn print_blocks(args: &Cli, blocks: &BlockMap, original_lines: &Vec<String>) {
    let mut keys = blocks.keys().collect::<Vec<&Block>>();
    keys.sort();
    for k in keys {
        let v = blocks.get(k).unwrap();
        let (i, l) = (k.start, k.size);

        if args.verbose == 0 {
            println!("{}, {}: {:?}", i, l, v);
        } else {
            println!("Line {} length {}: {:?}", i, l, v);
        }

        if args.verbose >= 2 {
            println!("{}\n", original_lines[i - 1 .. i + l - 1].join("\n"));
        }
    }
}

pub fn print_ending_status(args: &Cli, blocks: &BlockMap) {
    if args.verbose > 0 {
        let num_duped_blocks = blocks.values()
            .fold(0usize, |acc, item| acc + item.len());
        let total_dupes = blocks.len() + num_duped_blocks;
        println!("{} unique blocks with duplicates found, {} total duplicates",
            blocks.len(),
            total_dupes);
    }
}

/// Compute the edit distance of 2 strings, with shortcut.
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
pub fn levenshtein_distance(x: &String, y: &String, threshold: usize) -> usize {
    let (x, y): (Vec<char>, Vec<char>) = (x.chars().collect(), y.chars().collect());
    let (m, n) = (x.len(), y.len());
    let mut d = vec![0usize; (m + 1) * (n + 1)];
    let size = m + 1;

    if threshold >= m + n {
        return threshold;
    }

    for i in 1..(m + 1) {
        d[i + 0 * size] = i;
    }

    for j in 1..(n + 1) {
        d[0 + j * size] = j;
    }

    for j in 1..(n + 1) {
        let mut has_changed_row = false;

        for i in 1..(m + 1) {
            let sub_cost = if x[i - 1] == y[j - 1] { 0 } else { SUBSTITUTION_COST };
            d[i + j * size] = std::cmp::min(
                d[(i - 1) + j * size] + INSERTION_COST,
                std::cmp::min(
                    d[i + (j - 1) * size] + DELETION_COST,
                    d[(i - 1) + (j - 1) * size] + sub_cost));

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

/// Create a comparison function based on the given threshold.
///
/// If the threshold is 0, we use string comparison. If not, we use Levenshtein distance.
pub fn comparison_lambda(args: &Cli) -> ComparisonFn {
    let threshold = args.lev_threshold.clone();
    if threshold == 0 {
        Box::new(move |x, y| x == y)
    } else {
        Box::new(move |x, y| levenshtein_distance(x, y, threshold) <= threshold)
    }
}

/// Find block length of the code blocks.
///
/// Stops comparison when we reach the end, or if the original index hits the occurrance index.
/// This stops code blocks from "eating" the other code block (i.e. no nested overlapping blocks
/// that are similar).
pub fn get_block_length(
    original_index: usize,
    occurrance_index: usize,
    lines: &Vec<String>,
    comp: &ComparisonFn) -> usize {

    let mut block_length = 1;

    loop {
        let i = original_index + block_length;
        let j = occurrance_index + block_length;

        if i == occurrance_index {
            return block_length;
        }

        if j >= lines.len() {
            return block_length;
        }

        if comp(&lines[i], &lines[j]) {
            block_length += 1;
        } else {
            return block_length;
        }
    }
}

/// Remove code blocks that appear multiple times.
///
/// For instance, if we have 3 copies of a code block, there would be 2 keys that refer to it. The
/// first one has 2 indices (pointing to the last 2 copies) and the second one has 1 index
/// (pointing to the last copy). This function removes all of them except for the first instance of
/// the key.
pub fn remove_duplicate_blocks(blocks: BlockMap) -> BlockMap {
    let mut keys = blocks.keys().collect::<Vec<&Block>>();
    let mut ret = HashMap::new();
    let mut bad_keys = HashSet::new();
    keys.sort();

    for k in keys.iter().rev() {
        match blocks.get(k) {
            Some(linenos) => {
                for j in linenos {
                    let alt_index = Block { start: *j, size: k.size };
                    if blocks.contains_key(&alt_index) {
                        bad_keys.insert(alt_index);
                    }
                }
            },
            None => {
                continue;
            }
        }
    }

    for k in keys {
        if !bad_keys.contains(&k) {
            ret.insert(k.clone(), blocks.get(&k).unwrap().clone());
        }
    }

    ret
}

/// Compare lines.
pub fn global_compare_lines(args: &Cli, lines: &Vec<String>) -> BlockMap {
    let mut bm: BlockMap = HashMap::new();
    let bar = ProgressBar::new((lines.len() - 1).try_into().unwrap());
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"
            ).unwrap());
    let comp = comparison_lambda(args);
    let mut i = 0;

    while i < lines.len() - 1 {
        if lines[i].len() < args.line_threshold {
            i += 1;
            continue;
        }

        let mut j = i + 1;
        let mut max_block_length = 1;

        while j < lines.len() {
            if comp(&lines[i], &lines[j]) {
                let block_length = get_block_length(i, j, lines, &comp);

                if block_length < args.block_threshold {
                    j += block_length;
                    continue;
                }

                // Use 1-based indexing because that's how we usually count line numbers
                // Just remember to subtract 1 when interfacing with the lines array.
                let k = Block { start: i + 1, size: block_length };
                if bm.contains_key(&k) {
                    bm.get_mut(&k).unwrap().push(j + 1);
                } else {
                    bm.insert(k, vec![j + 1]);
                }

                j += block_length;
                max_block_length = std::cmp::max(max_block_length, block_length);
            } else {
                j += 1;
            }
        }

        // Skip smaller code blocks as an optimization. This removes the possibility of blocks
        // within other blocks, with the downside that genuine smaller "sub-"code blocks won't
        // be found.
        i += max_block_length;
        bar.inc(max_block_length as u64);
    }

    remove_duplicate_blocks(bm)
}

#[cfg(test)]
mod tests {
    use crate::comp::levenshtein_distance;

    macro_rules! check_lev {
        ( $a:literal, $b:literal, $t:literal ) => {
            {
                check_lev!($a, $b, $t, $t);
            }
        };

        ( $a:literal, $b:literal, $t:literal, $e:literal ) => {
            {
                let dist = levenshtein_distance(&$a.to_string(), &$b.to_string(), $t);
                assert_eq!(
                    dist,
                    $e,
                    "levenshtein_distance({}, {}, {}) = {}, expected {}",
                    $a, $b, $t, dist, $e);
            }
        }
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
