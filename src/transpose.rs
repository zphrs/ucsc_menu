pub fn transposed<T>(initial: Vec<Vec<T>>) -> Vec<Vec<T>> {
    // uses mem::swap to avoid cloning
    let mut transposed = Vec::with_capacity(initial[0].len());
    for _ in 0..initial[0].len() {
        transposed.push(Vec::with_capacity(initial.len()));
    }
    for row in initial {
        for (i, cell) in row.into_iter().enumerate() {
            transposed[i].push(cell);
        }
    }
    transposed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transposed() {
        let initial = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let after = transposed(initial);
        assert_eq!(after, vec![vec![1, 4, 7], vec![2, 5, 8], vec![3, 6, 9]]);
    }
}
