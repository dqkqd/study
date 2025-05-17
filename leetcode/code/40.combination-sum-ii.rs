fn back(current: &[i32], mut target: i32, candidates: &[i32]) -> Vec<Vec<i32>> {
    if target == 0 {
        vec![current.to_vec()]
    } else if target < 0 || candidates.is_empty() {
        vec![]
    } else {
        let mut out = vec![];
        let p = candidates
            .iter()
            .position(|p| p > &candidates[0])
            .unwrap_or(candidates.len());

        out.extend_from_slice(&back(current, target, &candidates[p..]));
        let mut current = current.to_vec();
        for _ in 0..p {
            current.push(candidates[0]);
            target -= candidates[0];
            out.extend_from_slice(&back(&current, target, &candidates[p..]));
        }
        out
    }
}

impl Solution {
    pub fn combination_sum2(candidates: Vec<i32>, target: i32) -> Vec<Vec<i32>> {
        let mut candidates = candidates;
        candidates.sort();
        let mut ans = vec![];
        back(&mut ans, target, &candidates)
    }
}
