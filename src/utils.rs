pub fn split_by_utf8_delimiter(data: Vec<u8>, delimiter: &[u8]) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    let mut start = 0;

    for (i, window) in data.windows(delimiter.len()).enumerate() {
        if window == delimiter {
            result.push(data[start..i].to_vec());
            start = i + delimiter.len();
        }
    }

    if start < data.len() {
        result.push(data[start..].to_vec());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_by_utf8_delimiter() {
        let data = vec![1, 2, 3, 4, 5, 6, 2, 3, 6, 7, 9];
        // [[1], [4, 5, 6], [6, 7, 9]]
        assert_eq!(
            split_by_utf8_delimiter(data, &[2, 3]),
            vec![vec![1], vec![4, 5, 6], vec![6, 7, 9]]
        );
    }
}
