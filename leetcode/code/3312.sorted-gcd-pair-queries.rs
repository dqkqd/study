impl Solution {
    pub fn gcd_values(nums: Vec<i32>, queries: Vec<i64>) -> Vec<i32> {
        let max = *nums.iter().max().unwrap() as usize;
        let mut count = vec![0; max + 1];
        for n in nums {
            count[n as usize] += 1;
        }

        for n in 1..max + 1 {
            for factor in 2..max + 1 {
                let m = n * factor;
                if m > max {
                    break;
                }
                count[n] += count[m];
            }
        }

        let mut pair_count: Vec<i64> = count.into_iter().map(|n| n * (n - 1) / 2).collect();

        for n in (1..max + 1).rev() {
            for factor in 2..max + 1 {
                let m = n * factor;
                if m > max {
                    break;
                }
                pair_count[n] -= pair_count[m];
            }
        }

        let mut prefix_sum = vec![0; max + 1];
        for i in 1..max + 1 {
            prefix_sum[i] = prefix_sum[i - 1] + pair_count[i];
        }

        queries
            .into_iter()
            .map(|q| {
                let index = prefix_sum.partition_point(|x| x <= &q);
                index.min(max) as i32
            })
            .collect()
    }
}
