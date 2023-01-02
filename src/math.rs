/// Calculate factorial from [stop, n].
/// Equivalent to n! / (stop - 1)!
pub fn factorial(n: usize, stop: usize) -> usize {
    let mut x = 1;

    for f in stop..=n {
        x *= f;
    }

    x
}

/// Calculate nCk
pub fn combinations(n: usize, k: usize) -> usize {
    if n < k {
        0
    } else {
        factorial(n, n - k + 1) / factorial(k, 2)
    }
}

#[cfg(test)]
mod tests {
    use super::{combinations, factorial};

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(1, 1), 1);
        assert_eq!(factorial(2, 1), 2);
        assert_eq!(factorial(10, 1), 3628800);
        assert_eq!(factorial(6, 3), 360);
    }

    #[test]
    fn test_combinations() {
        assert_eq!(combinations(9, 2), 36);
    }
}
