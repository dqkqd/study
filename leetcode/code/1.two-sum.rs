impl Solution {
    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
        let mut nums: Vec<(usize, i32)> = nums.into_iter().enumerate().collect();
        nums.sort_by_key(|k| k.1);

        let item = nums
            .iter()
            .find_map(|k| {
                let item = target - k.1;
                match nums.binary_search_by(|x| x.1.cmp(&item)) {
                    Ok(i) => Some((k.0 as i32, nums[i].0 as i32)),
                    Err(_) => None,
                }
            })
            .unwrap();

        vec![item.0 as i32, item.1 as i32]
    }
}
