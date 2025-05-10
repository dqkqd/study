impl Solution {
    pub fn get_longest_subsequence(words: Vec<String>, groups: Vec<i32>) -> Vec<String> {
        let mut index = vec![];
        let mut cur = -1;
        for (i, g) in groups.into_iter().enumerate() {
            if g != cur {
                index.push(i);
                cur = g;
            }
        }
        index
            .into_iter()
            .map(|u| words[u as usize].clone())
            .collect()
    }
}
