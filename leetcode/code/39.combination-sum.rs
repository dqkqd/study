#[derive(Clone)]
struct Data {
    solution: Vec<i32>,
    sum: i32,
}

fn backtrack(data: &Data, target: i32, candidates: &[i32]) -> Vec<Data> {
    if data.sum == target {
        vec![data.clone()]
    } else if data.sum > target || candidates.is_empty() {
        vec![]
    } else {
        let needed = target - data.sum;
        let value = candidates[0];
        let mut data = data.clone();
        let mut output = vec![];

        output.extend_from_slice(&backtrack(&data, target, &candidates[1..]));
        for _ in 0..needed / value {
            data.solution.push(value);
            data.sum += value;
            output.extend_from_slice(&backtrack(&data, target, &candidates[1..]));
        }
        output
    }
}

impl Solution {
    pub fn combination_sum(candidates: Vec<i32>, target: i32) -> Vec<Vec<i32>> {
        backtrack(
            &Data {
                solution: vec![],
                sum: 0,
            },
            target,
            &candidates,
        )
        .into_iter()
        .map(|v| v.solution)
        .collect()
    }
}
