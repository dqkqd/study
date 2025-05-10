use std::collections::VecDeque;

impl Solution {
    pub fn oranges_rotting(grid: Vec<Vec<i32>>) -> i32 {
        let mut grid = grid;
        let mut count = vec![vec![0; grid[0].len()]; grid.len()];
        let mut rottens = VecDeque::new();

        for (i, r) in grid.iter().enumerate() {
            for (j, c) in r.iter().enumerate() {
                if c == &2 {
                    rottens.push_back((i, j));
                }
            }
        }

        while let Some(pos) = rottens.pop_front() {
            for dx in [-1, 0, 1] {
                for dy in [-1, 0, 1] {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    if dx != 0 && dy != 0 {
                        continue;
                    }
                    let x = pos.0 as i32 + dx;
                    let y = pos.1 as i32 + dy;
                    if x < 0 || x >= grid.len() as i32 {
                        continue;
                    }
                    if y < 0 || y >= grid[0].len() as i32 {
                        continue;
                    }
                    let x = x as usize;
                    let y = y as usize;
                    if grid[x][y] == 1 {
                        grid[x][y] = 2;
                        count[x][y] = count[pos.0][pos.1] + 1;
                        rottens.push_back((x, y));
                    }
                }
            }
        }

        if grid.into_iter().flatten().any(|x| x == 1) {
            -1
        } else {
            count.into_iter().flatten().max().unwrap_or_default() as i32
        }
    }
}
