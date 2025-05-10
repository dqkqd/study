impl Solution {
    pub fn change(amount: i32, coins: Vec<i32>) -> i32 {
        let mut dp = vec![0; amount as usize + 1];
        dp[0] = 1;

        for c in coins {
            for amt in c..amount + 1 {
                dp[amt as usize] += dp[(amt - c) as usize];
            }
        }
        dp[amount as usize]
    }
}
