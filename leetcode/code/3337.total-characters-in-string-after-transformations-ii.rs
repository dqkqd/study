use std::ops::Mul;

const MOD: i64 = 1_000_000_007;

#[derive(Debug, Clone)]
struct Matrix<const H: usize, const W: usize> {
    data: [[i64; W]; H],
}

type SquareMatrix<const N: usize> = Matrix<N, N>;

impl<const H: usize, const W: usize> Matrix<H, W> {
    fn new(v: Vec<Vec<i32>>) -> Self {
        let mut data = [[0; W]; H];
        for (h, row) in v.iter().enumerate() {
            for (w, e) in row.iter().enumerate() {
                data[h][w] = *e as i64;
            }
        }
        Matrix { data }
    }

    fn zero() -> Self {
        Self::new(Vec::new())
    }

    fn one() -> Self {
        let mut mat = Self::zero();
        for x in 0..W {
            mat.data[x][x] = 1;
        }
        mat
    }

    fn sum(&self) -> i32 {
        self.data.iter().flatten().fold(0, |acc, x| (acc + x) % MOD) as i32
    }
}

impl<const N: usize> SquareMatrix<N> {
    fn pow(mut self, mut t: i32) -> Self {
        let mut res = Self::one();
        while t > 0 {
            if t & 1 == 1 {
                res = &res * &self;
            }
            self = &self * &self;
            t >>= 1;
        }
        res
    }
}

impl<const M: usize, const N: usize, const P: usize> Mul<&Matrix<N, P>> for &Matrix<M, N> {
    type Output = Matrix<M, P>;

    fn mul(self, rhs: &Matrix<N, P>) -> Self::Output {
        let mut output = Matrix::<M, P>::zero();
        for m in 0..M {
            for p in 0..P {
                for n in 0..N {
                    output.data[m][p] += self.data[m][n] * rhs.data[n][p];
                    output.data[m][p] %= MOD;
                }
            }
        }
        output
    }
}

impl Solution {
    pub fn length_after_transformations(s: String, t: i32, nums: Vec<i32>) -> i32 {
        let mut mat = SquareMatrix::<26>::zero();
        for (col, n) in nums.iter().enumerate() {
            for row in (col + 1)..(col + 1 + *n as usize) {
                mat.data[row % 26][col] = 1;
            }
        }

        let mat = mat.pow(t);

        let mut count = [0; 26];
        for c in s.chars() {
            count[(c as usize) - ('a' as usize)] += 1;
        }
        let count = count.into_iter().map(|v| vec![v]).collect();
        let one = Matrix::<26, 1>::new(count);
        let mul = &mat * &one;

        mul.sum()
    }
}
