impl Solution {
    pub fn is_zero_array(nums: Vec<i32>, queries: Vec<Vec<i32>>) -> bool {
        let mut range = vec![0; nums.len() + 1];
        for q in queries {
            range[q[0] as usize] += 1;
            range[q[1] as usize + 1] -= 1;
        }

        for i in 1..range.len() {
            range[i] += range[i - 1]
        }
        nums.into_iter().zip(range).all(|(n, p)| n <= p)
    }
}
