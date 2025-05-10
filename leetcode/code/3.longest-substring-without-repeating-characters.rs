use std::collections::BTreeSet;

impl Solution {
    pub fn length_of_longest_substring(s: String) -> i32 {
        let s: Vec<char> = s.chars().collect();
        let n = s.len();
        let mut ans = 0;
        let mut x = 0;

        let mut chars = BTreeSet::new();
        for y in 0..n {
            loop {
                if chars.contains(&s[y]) {
                    chars.remove(&s[x]);
                    x += 1
                } else {
                    break;
                }
            }
            chars.insert(s[y]);

            ans = ans.max(chars.len() as i32);
        }
        ans = ans.max(chars.len() as i32);

        ans
    }
}
