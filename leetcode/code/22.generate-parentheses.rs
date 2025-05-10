use std::collections::VecDeque;

impl Solution {
    pub fn generate_parenthesis(n: i32) -> Vec<String> {
        let mut queue = VecDeque::new();
        queue.push_back(("".to_string(), 0));
        while let Some((s, sum)) = queue.pop_front() {
            if s.len() == 2 * n as usize {
                queue.push_back((s, sum));
                break;
            }
            let remaining = 2 * n as usize - s.len();
            if sum > 0 {
                queue.push_back((s.clone() + ")", sum - 1));
            }
            if sum < remaining {
                queue.push_back((s + "(", sum + 1));
            }
        }
        queue.into_iter().map(|v| v.0).collect()
    }
}
