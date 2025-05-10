use std::collections::HashMap;

impl Solution {
    pub fn three_sum(nums: Vec<i32>) -> Vec<Vec<i32>> {
        let mut count = HashMap::new();
        for x in nums {
            *count.entry(x).or_insert(0) += 1;
        }
        let mut res = vec![];

        // all equal
        if count.get(&0).is_some_and(|v| v >= &3) {
            res.push(vec![0, 0, 0]);
        }

        // two equal
        for (k, v) in count.iter() {
            if k != &0 && v >= &2 && count.contains_key(&(-2 * k)) {
                res.push(vec![*k, *k, -2 * k]);
            }
        }

        // three distinct
        for x in count.keys() {
            for y in count.keys() {
                let z = -x - y;
                if x < y && y < &z && count.contains_key(&z) {
                    res.push(vec![*x, *y, z]);
                }
            }
        }

        res
    }
}
