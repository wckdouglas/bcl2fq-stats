use std::collections::HashMap;

/// Descending sorting a hashmap, and return a vector of tuple
pub fn sort_hashmap(input_hashmap: &HashMap<String, u64>) -> Result<Vec<(&String, &u64)>, String> {
    let mut input_vec = Vec::from_iter(input_hashmap);
    input_vec.sort_by(|(_, a), (_, b)| b.cmp(&a));
    Ok(input_vec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        vec![("AAA".to_string(), 1), ("ACA".to_string(), 2)], 
        vec![("ACA".to_string(), 2), ("AAA".to_string(), 1)]
    )]
    #[case(
        vec![("TTT".to_string(), 4),("AAA".to_string(), 1), ("ACA".to_string(), 2)], 
        vec![("TTT".to_string(), 4),("ACA".to_string(), 2), ("AAA".to_string(), 1)]
    )]
    #[case(
        vec![("TTT".to_string(), 2),("AAA".to_string(), 3), ("ACA".to_string(), 4)], 
        vec![("ACA".to_string(), 4),("AAA".to_string(), 3), ("TTT".to_string(), 2)]
    )]
    fn test_sort_hashmap(
        #[case] input_array: Vec<(String, u64)>,
        #[case] expected_array: Vec<(String, u64)>,
    ) {
        let input_hashmap: HashMap<String, u64> = HashMap::from_iter(input_array);
        let sorted = sort_hashmap(&input_hashmap).unwrap();
        let sorted_vec: Vec<(String, u64)> = sorted
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        assert_eq!(sorted_vec, expected_array);
    }
}
