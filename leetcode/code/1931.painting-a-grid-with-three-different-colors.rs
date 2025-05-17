const MOD: i64 = 1_000_000_007;

impl Solution {
    pub fn color_the_grid(m: i32, n: i32) -> i32 {
        let total = (3_f64).powi(m) as usize;
        let mut possible_cols = Vec::with_capacity(total);
        for x in 0..total {
            let mut prev = None;
            let mut good = true;
            let mut y = x;
            for _ in 0..m {
                if let Some(p) = prev {
                    if p == y % 3 {
                        good = false;
                        break;
                    }
                }
                prev = Some(y % 3);
                y /= 3;
            }
            if good {
                possible_cols.push(x);
            }
        }

        let mut data = vec![vec![false; possible_cols.len()]; possible_cols.len()];
        for (i, x) in possible_cols.iter().enumerate() {
            for (j, y) in possible_cols.iter().enumerate() {
                let mut x = *x;
                let mut y = *y;
                let mut good = true;
                for _ in 0..m {
                    if x % 3 == y % 3 {
                        good = false;
                        break;
                    }
                    x /= 3;
                    y /= 3;
                }
                if good {
                    data[i][j] = true;
                }
            }
        }

        let mut dp = vec![vec![0; possible_cols.len()]; 2];
        for x in 0..possible_cols.len() {
            dp[0][x] = 1;
        }
        for _ in 2..=(n as usize) {
            for y in 0..possible_cols.len() {
                dp[1][y] = 0;
                for z in 0..possible_cols.len() {
                    if data[y][z] {
                        dp[1][y] += dp[0][z];
                        dp[1][y] %= MOD;
                    }
                }
            }
            dp.swap(0, 1);
        }

        dp[0].iter().fold(0, |acc, x| (acc + x) % MOD) as i32
    }
}
