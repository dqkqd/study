impl Solution {
    pub fn max_area(height: Vec<i32>) -> i32 {
        let mut height_pos: Vec<Option<(i32, i32)>> = vec![None; 10_001];
        for (i, h) in height.iter().enumerate() {
            let i = i as i32;
            let pos = height_pos.get_mut(*h as usize).unwrap();
            match pos {
                Some(pos) => {
                    pos.0 = pos.0.min(i);
                    pos.1 = pos.1.max(i);
                }
                None => {
                    *pos = Some((i, i));
                }
            }
        }

        let mut ans = 0;
        let mut cur: Option<(i32, i32)> = None;
        for (h, pos) in height_pos.iter().enumerate().rev() {
            if let Some(pos) = pos {
                match &cur {
                    Some(c) => {
                        let m = (c.0.min(pos.0), c.1.max(pos.1));
                        ans = ans.max(h as i32 * (m.1 - m.0));
                        cur = Some(m);
                    }
                    None => {
                        ans = ans.max(h as i32 * (pos.1 - pos.0));
                        cur = Some(pos.clone());
                    }
                }
            }
        }

        ans
    }
}
