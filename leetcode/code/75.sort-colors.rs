impl Solution {
    pub fn sort_colors(nums: &mut Vec<i32>) {
        let n = nums.len();
        let mut i0 = 0;
        let mut i2 = n - 1;
        let mut i = 0;
        while i < nums.len() && i0 < i2 && i <= i2 {
            match nums[i] {
                0 => {
                    nums.swap(i, i0);
                    i0 += 1;
                    i += 1;
                }
                1 => {
                    i += 1;
                }
                2 => {
                    nums.swap(i, i2);
                    i2 -= 1;
                }
                _ => unreachable!(),
            }
        }
    }
}
