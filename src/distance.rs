/// Hamming distance function to group undetermined barcodes into known ones
///
/// # Arguments
/// - barcode_a: the first barcode to be compared
/// - barcode_b: the second barcode to be compared (same length as barcode_a)
///
/// # Returns
/// - hamming distance
pub fn hamming_distance(barcode_a: &[u8], barcode_b: &[u8]) -> Result<u8, String> {
    let mut score: u8 = 0; // u8 should be enough for barcode distance given barcodes are not long
    if barcode_a.len() == barcode_b.len() {
        for it in barcode_a.iter().zip(barcode_b.iter()) {
            let (a, b): (&u8, &u8) = it;
            if a != b {
                score += 1;
            }
        }
        Ok(score)
    } else {
        Err("Barcode lengths are not the same".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    #[should_panic(expected = "Barcode lengths are not the same")]
    fn test_hamming_distance_fail() {
        let str_a = "ACTGCGG".as_bytes();
        let str_b = "ACTGCG".as_bytes();
        hamming_distance(&str_a, &str_b).unwrap();
    }

    #[rstest]
    #[case("ACTG", "ACTG", 0)]
    #[case("ACTG", "ATTG", 1)]
    #[case("ACTG", "ATGG", 2)]
    #[case("ACTG", "ATCT", 3)]
    #[case("ACTG", "GACT", 4)]
    fn test_hamming_distance(#[case] str_a: &str, #[case] str_b: &str, #[case] expected_score: u8) {
        let str_a_bytes = str_a.as_bytes();
        let str_b_bytes = str_b.as_bytes();
        let score = hamming_distance(&str_a_bytes, &str_b_bytes).unwrap();
        assert_eq!(score, expected_score);
    }
}
