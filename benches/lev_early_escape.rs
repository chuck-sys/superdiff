use criterion::{black_box, criterion_group, criterion_main, Criterion};

const INSERTION_COST: usize = 1;
const DELETION_COST: usize = 1;
const SUBSTITUTION_COST: usize = 1;

fn levenshtein_distance_no_upper(x: &String, y: &String, threshold: usize) -> usize {
    let (x, y): (Vec<char>, Vec<char>) = (x.chars().collect(), y.chars().collect());
    let (m, n) = (x.len(), y.len());
    let mut d = vec![0usize; (m + 1) * (n + 1)];
    let size = m + 1;

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

fn levenshtein_distance_yes_upper(x: &String, y: &String, threshold: usize) -> usize {
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

fn criterion_lev_check(c: &mut Criterion) {
    let apple = "apple".to_string();
    let orange = "orange".to_string();

    c.bench_function("lev no upper limit", |b| b.iter(|| levenshtein_distance_no_upper(black_box(&apple), black_box(&orange), black_box(11usize))));
    c.bench_function("lev has upper limit", |b| b.iter(|| levenshtein_distance_yes_upper(black_box(&apple), black_box(&orange), black_box(11usize))));
}

criterion_group!(benches, criterion_lev_check);
criterion_main!(benches);
