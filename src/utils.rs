use std::collections::HashMap;

pub fn sort_hashmap(input_hashmap: &HashMap<String, u64>) -> Result<Vec<(&String, &u64)>, String> {
    let mut input_vec = Vec::from_iter(input_hashmap);
    input_vec.sort_by(|(_, a), (_, b)| b.cmp(&a));
    Ok(input_vec)
}
