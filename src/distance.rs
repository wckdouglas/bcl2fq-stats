pub fn hamming_distance(barcode_a: &[u8], barcode_b: &[u8]) -> Result<u8, String> {
    let mut score = 0;
    if barcode_a.len() == barcode_b.len() {
        for it in barcode_a.iter().zip(barcode_b.iter()) {
            let (a, b) = it;
            if a != b {
                score += 1;
            }
        }
        Ok(score)
    } else {
        Err("Barcode lengths are not the same".to_string())
    }
}
