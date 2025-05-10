impl Solution {
    pub fn reverse(x: i32) -> i32 {
        let to_vec = |mut v: i32| -> Vec<i32> {
            let mut data = vec![];
            while v != 0 {
                data.push((v % 10).abs());
                v /= 10;
            }
            data.into_iter().rev().collect()
        };
        let to_num = |v: Vec<i32>, neg: bool| -> i32 {
            let mut data = 0;
            for u in v {
                data *= 10;
                data += u;
            }
            if neg {
                -data
            } else {
                data
            }
        };
        let le = |v1: &[i32], v2: &[i32]| -> bool {
            if v1.len() > v2.len() {
                false
            } else if v1.len() < v2.len() {
                true
            } else {
                for (lhs, rhs) in v1.iter().zip(v2.iter()) {
                    if lhs > rhs {
                        return false;
                    } else if lhs < rhs {
                        break;
                    }
                }
                true
            }
        };

        let rev: Vec<i32> = to_vec(x).into_iter().rev().collect();
        let max = to_vec(i32::MAX);
        let min = to_vec(i32::MIN);

        if x >= 0 && le(&rev, &max) {
            to_num(rev, false)
        } else if x < 0 && le(&rev, &min) {
            to_num(rev, true)
        } else {
            0
        }
    }
}
