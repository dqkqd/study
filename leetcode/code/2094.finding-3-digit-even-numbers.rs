use std::collections::HashMap;

impl Solution {
    pub fn find_even_numbers(digits: Vec<i32>) -> Vec<i32> {
        let mut count = HashMap::new();
        for x in digits {
            *count.entry(x).or_insert(0) += 1;
        }

        let mut ans = vec![];

        for x in (100..1000).step_by(2) {
            let a = x / 100;
            let b = x / 10 % 10;
            let c = x % 10;

            *count.entry(a).or_insert(0) -= 1;
            *count.entry(b).or_insert(0) -= 1;
            *count.entry(c).or_insert(0) -= 1;

            if count[&a] >= 0 && count[&b] >= 0 && count[&c] >= 0 {
                ans.push(x);
            }

            *count.get_mut(&a).unwrap() += 1;
            *count.get_mut(&b).unwrap() += 1;
            *count.get_mut(&c).unwrap() += 1;
        }

        ans.sort();
        ans
    }
}
