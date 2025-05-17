impl Solution {
    pub fn search(nums: Vec<i32>, target: i32) -> i32 {
        let max = nums.last().unwrap();
        let p = nums.partition_point(|x| x > &max);

        if let Ok(v) = nums[..p].binary_search(&target) {
            v as i32
        } else if let Ok(v) = nums[p..].binary_search(&target) {
            (v + p) as i32
        } else {
            -1
        }
    }
}
