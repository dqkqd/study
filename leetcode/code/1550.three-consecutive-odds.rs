impl Solution {
    pub fn three_consecutive_odds(arr: Vec<i32>) -> bool {
        arr.windows(3).any(|v| v.iter().all(|x| x % 2 == 1))
    }
}
