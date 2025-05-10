impl Solution {
    pub fn length_after_transformations(s: String, t: i32) -> i32 {
        let m = 1_000_000_007;

        let mut dp = vec![0; t as usize + 27];
        for i in 0..26 {
            dp[i] = 1;
        }
        for i in 26..t as usize + 27 {
            dp[i] = dp[i - 26] + dp[i - 25];
            dp[i] %= m;
        }
        let mut ans = 0;
        for c in s.chars() {
            let c = (c as usize) - ('a' as usize);
            ans += dp[t as usize + c];
            ans %= m;
        }
        ans
    }
}
