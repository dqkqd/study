fn hamming_one(lhs: &str, rhs: &str) -> bool {
    if lhs.len() != rhs.len() {
        false
    } else {
        let mut c = 0;
        let mut iter = lhs.chars().zip(rhs.chars()).filter(|(x, y)| x != y);
        while iter.next().is_some() && c < 2 {
            c += 1;
        }
        c == 1
    }
}

impl Solution {
    pub fn get_words_in_longest_subsequence(words: Vec<String>, groups: Vec<i32>) -> Vec<String> {
        let n = words.len();
        let mut length = vec![1; n];

        let mut prev = vec![None; n];

        for i in 0..length.len() {
            for j in 0..i {
                if length[i] < length[j] + 1
                    && groups[i] != groups[j]
                    && hamming_one(&words[i], &words[j])
                {
                    length[i] = length[j] + 1;
                    prev[i] = Some(j);
                }
            }
        }

        let m = *length.iter().max().unwrap();
        let mut p = length.iter().position(|x| x == &m).unwrap();
        let mut ans = Vec::with_capacity(n);
        ans.push(p);
        while let Some(c) = prev[p] {
            p = c;
            ans.push(c);
        }

        ans.into_iter().rev().map(|i| words[i].clone()).collect()
    }
}
