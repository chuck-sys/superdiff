use std::collections::{HashMap, HashSet};
use indicatif::ProgressBar;
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

pub fn levenshtein_distance(x: &String, y: &String) -> usize {
    0
}

pub fn comparison_lambda(args: &Cli) -> ComparisonFn {
    let threshold = args.lev_threshold.clone();
    if threshold == 0 {
        Box::new(move |x, y| x == y)
    } else {
        Box::new(move |x, y| levenshtein_distance(x, y) <= threshold)
    }
}

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

pub fn global_compare_lines(args: &Cli, lines: &Vec<String>) -> BlockMap {
    let mut bm: BlockMap = HashMap::new();
    let bar = ProgressBar::new((lines.len() - 1).try_into().unwrap());
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
