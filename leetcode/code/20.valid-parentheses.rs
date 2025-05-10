impl Solution {
    pub fn is_valid(s: String) -> bool {
        let mut stack = vec![];
        for c in s.chars() {
            match c {
                c @ ('{' | '[' | '(') => stack.push(c),
                '}' => {
                    let v = stack.pop();
                    if !matches!(v, Some('{')) {
                        return false;
                    }
                }
                ']' => {
                    let v = stack.pop();
                    if !matches!(v, Some('[')) {
                        return false;
                    }
                }
                ')' => {
                    let v = stack.pop();
                    if !matches!(v, Some('(')) {
                        return false;
                    }
                }
                _ => unreachable!(),
            }
        }

        stack.is_empty()
    }
}
