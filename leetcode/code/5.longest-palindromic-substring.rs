impl Solution {
    pub fn longest_palindrome(s: String) -> String {
        let chars: Vec<char> = s.chars().collect();

        let (mut lo, mut hi) = (0, 0);

        let mut span = |mut l: i32, mut r: i32| loop {
            if l < 0 || r >= chars.len() as i32 || chars[l as usize] != chars[r as usize] {
                break;
            }
            if (hi - lo + 1) <= (r - l + 1) {
                lo = l;
                hi = r;
            }
            l -= 1;
            r += 1;
        };
        for i in 0..chars.len() as i32 {
            span(i, i);
            span(i, i + 1);
        }

        chars[lo as usize..hi as usize + 1].iter().collect()
    }
}
