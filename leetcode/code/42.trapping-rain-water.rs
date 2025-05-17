impl Solution {
    pub fn trap(height: Vec<i32>) -> i32 {
        let max = height.iter().max().unwrap();
        let max_pos = height.iter().position(|h| h == max).unwrap();

        let mut water = 0;
        let mut max = height[0];
        for i in 1..max_pos {
            if height[i] > max {
                max = height[i];
            } else {
                water += max - height[i];
            }
        }
        let mut max = *height.last().unwrap();
        for i in (max_pos + 1..height.len()).rev() {
            if height[i] > max {
                max = height[i];
            } else {
                water += max - height[i];
            }
        }

        water
    }
}
