impl Solution {
    pub fn letter_combinations(digits: String) -> Vec<String> {
        if digits.is_empty() {
            return vec![];
        }
        let mut ans: Vec<String> = vec!["".into()];
        for c in digits.chars() {
            let arr = match c {
                '2' => "abc",
                '3' => "def",
                '4' => "ghi",
                '5' => "jkl",
                '6' => "mno",
                '7' => "pqrs",
                '8' => "tuv",
                '9' => "wxyz",
                _ => unreachable!(),
            };
            let mut new = vec![];
            for c in arr.chars() {
                for x in ans.iter() {
                    new.push(format!("{}{}", x, c));
                }
            }
            std::mem::swap(&mut new, &mut ans);
        }
        ans
    }
}
