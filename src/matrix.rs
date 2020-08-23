use crate::Tuple;
use std::ops::{Index, Mul};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Matrix2x2([[f64; 2]; 2]);

impl Matrix2x2 {
    fn determinant(&self) -> f64 {
        self.0[0][0] * self.0[1][1] - self.0[0][1] * self.0[1][0]
    }
}

impl Index<usize> for Matrix2x2 {
    type Output = [f64; 2];

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Matrix3x3([[f64; 3]; 3]);

impl Matrix3x3 {
    fn submatrix(&self, row: usize, column: usize) -> Matrix2x2 {
        let mut sub = [[0.0; 2]; 2];
        let mut sr = 0;
        for r in 0..3 {
            if row == r {
                continue;
            }
            let mut sc = 0;
            for c in 0..3 {
                if column == c {
                    continue;
                }
                sub[sr][sc] = self.0[r][c];
                sc += 1;
            }
            sr += 1;
        }
        Matrix2x2(sub)
    }

    // determinant of the submatrix
    fn minor(&self, row: usize, column: usize) -> f64 {
        let sub = self.submatrix(row, column);
        sub.determinant()
    }

    fn cofactor(&self, row: usize, column: usize) -> f64 {
        /*
        sign swapped as follows
        [+ - +]
        [- + -]
        [+ - +]
        or, more simply, if row + column is odd
        */
        let sign = if (row + column) % 2 == 0 { 1.0 } else { -1.0 };
        self.minor(row, column) * sign
    }

    fn determinant(&self) -> f64 {
        let mut determinant = 0.0;
        let mut c = 0;
        for col in &self.0[0] {
            determinant += col * self.cofactor(0, c);
            c += 1;
        }
        determinant
    }
}

impl Index<usize> for Matrix3x3 {
    type Output = [f64; 3];

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Matrix4x4([[f64; 4]; 4]);

impl Matrix4x4 {
    fn identity() -> Matrix4x4 {
        Matrix4x4([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Used when translating normal vectors between object space and world space
    fn transpose(&self) -> Matrix4x4 {
        let a = self.0;
        let mut ta = [[0.0; 4]; 4];
        for row in 0..4 {
            for col in 0..4 {
                ta[row][col] = a[col][row];
            }
        }
        Matrix4x4(ta)
    }

    fn submatrix(&self, row: usize, column: usize) -> Matrix3x3 {
        let mut sub = [[0.0; 3]; 3];
        let mut sr = 0;
        for r in 0..4 {
            if row == r {
                continue;
            }
            let mut sc = 0;
            for c in 0..4 {
                if column == c {
                    continue;
                }
                sub[sr][sc] = self.0[r][c];
                sc += 1;
            }
            sr += 1;
        }
        Matrix3x3(sub)
    }

    fn cofactor(&self, row: usize, column: usize) -> f64 {
        /*
        sign swapped as follows
        [+ - +]
        [- + -]
        [+ - +]
        or, more simply, if row + column is odd
        */
        let sign = if (row + column) % 2 == 0 { 1.0 } else { -1.0 };
        self.minor(row, column) * sign
    }

    fn determinant(&self) -> f64 {
        let mut determinant = 0.0;
        let mut c = 0;
        for col in &self.0[0] {
            determinant += col * self.cofactor(0, c);
            c += 1;
        }
        determinant
    }

    fn is_invertiable(&self) -> bool {
        self.determinant() != 0.0
    }

    fn inverse(&self) -> Option<Matrix4x4> {
        let determinant = self.determinant();
        if determinant == 0.0 {
            return None;
        }
        let mut cofactors = [[0.0; 4]; 4];
        for row in 0..4 {
            for col in 0..4 {
                cofactors[row][col] = self.cofactor(row, col);
            }
        }
        let mut inverse = [[0.0; 4]; 4];
        for row in 0..4 {
            for col in 0..4 {
                // perform transpose
                inverse[col][row] = cofactors[row][col] / determinant;
            }
        }
        Some(Matrix4x4(inverse))
    }

    // determinant of the submatrix
    fn minor(&self, row: usize, column: usize) -> f64 {
        let sub = self.submatrix(row, column);
        sub.determinant()
    }
}

const EPSILON: f64 = 0.00001;

fn equal(a: f64, b: f64) -> bool {
    f64::abs(a - b) < EPSILON
}

impl PartialEq for Matrix4x4 {
    fn eq(&self, other: &Matrix4x4) -> bool {
        let a = self;
        let b = other;
        for row in 0..4 {
            for col in 0..4 {
                if !equal(a[row][col], b[row][col]) {
                    return false;
                }
            }
        }
        true
    }
}

impl Index<usize> for Matrix4x4 {
    type Output = [f64; 4];

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

// Matrix multiplication computes the dot product of every row-column combination
impl Mul for Matrix4x4 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let a = self.0;
        let b = rhs.0;
        let mut m = [[0.0; 4]; 4];
        let max = a.len();
        for r in 0..max {
            for c in 0..max {
                m[r][c] =
                    a[r][0] * b[0][c] + a[r][1] * b[1][c] + a[r][2] * b[2][c] + a[r][3] * b[3][c];
            }
        }
        Matrix4x4(m)
    }
}

impl Mul<Tuple> for Matrix4x4 {
    type Output = Tuple;

    fn mul(self, rhs: Tuple) -> Tuple {
        let a = self.0;
        let b = [rhs.x, rhs.y, rhs.z, rhs.w];
        let mut m = [0.0; 4];
        let max = a.len();
        for r in 0..max {
            m[r] = a[r][0] * b[0] + a[r][1] * b[1] + a[r][2] * b[2] + a[r][3] * b[3];
        }
        (m[0], m[1], m[2], m[3]).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_create() {
        let m = Matrix4x4([
            [1.0, 2.0, 3.0, 4.0],
            [5.5, 6.5, 7.5, 8.5],
            [9.0, 10.0, 11.0, 12.0],
            [13.5, 14.5, 15.5, 16.5],
        ]);
        assert_eq!(m[0][0], 1.0);
        assert_eq!(m[0][3], 4.0);
        assert_eq!(m[1][0], 5.5);
        assert_eq!(m[1][2], 7.5);
        assert_eq!(m[2][2], 11.0);
        assert_eq!(m[3][0], 13.5);
        assert_eq!(m[3][2], 15.5);

        let m = Matrix2x2([[-3.0, 5.0], [1.0, -2.0]]);
        assert_eq!(m[0][0], -3.0);
        assert_eq!(m[0][1], 5.0);
        assert_eq!(m[1][0], 1.0);
        assert_eq!(m[1][1], -2.0);

        let m = Matrix3x3([[-3.0, 5.0, 0.0], [1.0, -2.0, -7.0], [0.0, 1.0, 1.0]]);
        assert_eq!(m[0][0], -3.0);
        assert_eq!(m[1][1], -2.0);
        assert_eq!(m[2][2], 1.0);
    }

    #[test]
    fn matrix_compare() {
        let a = Matrix4x4([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ]);
        let b = Matrix4x4([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ]);
        assert_eq!(a, b);
        let b = Matrix4x4([
            [2.0, 3.0, 4.0, 5.0],
            [6.0, 7.0, 8.0, 9.0],
            [8.0, 7.0, 6.0, 5.0],
            [4.0, 3.0, 2.0, 1.0],
        ]);
        assert_ne!(a, b);
    }

    #[test]
    fn matrix_multiply() {
        let a = Matrix4x4([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ]);
        let b = Matrix4x4([
            [-2.0, 1.0, 2.0, 3.0],
            [3.0, 2.0, 1.0, -1.0],
            [4.0, 3.0, 6.0, 5.0],
            [1.0, 2.0, 7.0, 8.0],
        ]);
        let ab = Matrix4x4([
            [20.0, 22.0, 50.0, 48.0],
            [44.0, 54.0, 114.0, 108.0],
            [40.0, 58.0, 110.0, 102.0],
            [16.0, 26.0, 46.0, 42.0],
        ]);
        assert_eq!(a * b, ab);

        let a = Matrix4x4([
            [1.0, 2.0, 3.0, 4.0],
            [2.0, 4.0, 4.0, 2.0],
            [8.0, 6.0, 4.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        let b: Tuple = (1.0, 2.0, 3.0, 1.0).into();
        assert_eq!(a * b, (18.0, 24.0, 33.0, 1.0).into());
    }

    #[test]
    fn matrix_identity() {
        let a = Matrix4x4([
            [0.0, 1.0, 2.0, 4.0],
            [1.0, 2.0, 4.0, 8.0],
            [2.0, 4.0, 8.0, 16.0],
            [4.0, 8.0, 16.0, 32.0],
        ]);

        assert_eq!(a * Matrix4x4::identity(), a);

        let a: Tuple = (1.0, 2.0, 3.0, 4.0).into();
        assert_eq!(Matrix4x4::identity() * a, a);
    }

    #[test]
    fn matrix_transpose() {
        let a = Matrix4x4([
            [0.0, 9.0, 3.0, 0.0],
            [9.0, 8.0, 0.0, 8.0],
            [1.0, 8.0, 5.0, 3.0],
            [0.0, 0.0, 5.0, 8.0],
        ]);
        let ta = Matrix4x4([
            [0.0, 9.0, 1.0, 0.0],
            [9.0, 8.0, 8.0, 0.0],
            [3.0, 0.0, 5.0, 5.0],
            [0.0, 8.0, 3.0, 8.0],
        ]);
        assert_eq!(a.transpose(), ta);
        assert_eq!(Matrix4x4::identity().transpose(), Matrix4x4::identity());
    }

    #[test]
    fn matrix_determinant() {
        let a = Matrix2x2([[1.0, 5.0], [-3.0, 2.0]]);
        assert_eq!(a.determinant(), 17.0);
    }

    #[test]
    fn matrix_submatrix() {
        let a = Matrix3x3([[1.0, 5.0, 0.0], [-3.0, 2.0, 7.0], [0.0, 6.0, -3.0]]);
        let sub_a = Matrix2x2([[-3.0, 2.0], [0.0, 6.0]]);
        assert_eq!(a.submatrix(0, 2), sub_a);

        let a = Matrix4x4([
            [-6.0, 1.0, 1.0, 6.0],
            [-8.0, 5.0, 8.0, 6.0],
            [-1.0, 0.0, 8.0, 2.0],
            [-7.0, 1.0, -1.0, 1.0],
        ]);
        let sub_a = Matrix3x3([[-6.0, 1.0, 6.0], [-8.0, 8.0, 6.0], [-7.0, -1.0, 1.0]]);
        assert_eq!(a.submatrix(2, 1), sub_a);
    }

    #[test]
    fn matrix_minor() {
        let a = Matrix3x3([[3.0, 5.0, 0.0], [2.0, -1.0, -7.0], [6.0, -1.0, 5.0]]);
        let b = a.submatrix(1, 0);
        assert_eq!(b.determinant(), 25.0);
        assert_eq!(a.minor(1, 0), 25.0);
    }

    #[test]
    fn matrix_cofactor() {
        let a = Matrix3x3([[3.0, 5.0, 0.0], [2.0, -1.0, -7.0], [6.0, -1.0, 5.0]]);
        assert_eq!(a.minor(0, 0), -12.0);
        assert_eq!(a.cofactor(0, 0), -12.0);
        assert_eq!(a.minor(1, 0), 25.0);
        assert_eq!(a.cofactor(1, 0), -25.0);
    }

    #[test]
    fn matrix_determinant_for_larger() {
        let a = Matrix3x3([[1.0, 2.0, 6.0], [-5.0, 8.0, -4.0], [2.0, 6.0, 4.0]]);
        assert_eq!(a.cofactor(0, 0), 56.0);
        assert_eq!(a.cofactor(0, 1), 12.0);
        assert_eq!(a.cofactor(0, 2), -46.0);
        assert_eq!(a.determinant(), -196.0);

        let a = Matrix4x4([
            [-2.0, -8.0, 3.0, 5.0],
            [-3.0, 1.0, 7.0, 3.0],
            [1.0, 2.0, -9.0, 6.0],
            [-6.0, 7.0, 7.0, -9.0],
        ]);
        assert_eq!(a.cofactor(0, 0), 690.0);
        assert_eq!(a.cofactor(0, 1), 447.0);
        assert_eq!(a.cofactor(0, 2), 210.0);
        assert_eq!(a.cofactor(0, 3), 51.0);
        assert_eq!(a.determinant(), -4071.0);
    }

    #[test]
    fn matrix_invertible() {
        let a = Matrix4x4([
            [6.0, 4.0, 4.0, 4.0],
            [5.0, 5.0, 7.0, 6.0],
            [4.0, -9.0, 3.0, -7.0],
            [9.0, 1.0, 7.0, -6.0],
        ]);
        assert_eq!(a.determinant(), -2120.0);
        assert_eq!(a.is_invertiable(), true);

        let a = Matrix4x4([
            [-4.0, 2.0, -2.0, -3.0],
            [9.0, 6.0, 2.0, 6.0],
            [0.0, -5.0, 1.0, -5.0],
            [0.0, 0.0, 0.0, 0.0],
        ]);
        assert_eq!(a.determinant(), 0.0);
        assert_eq!(a.is_invertiable(), false);
    }

    #[test]
    fn matrix_invert() {
        let a = Matrix4x4([
            [-5.0, 2.0, 6.0, -8.0],
            [1.0, -5.0, 1.0, 8.0],
            [7.0, 7.0, -6.0, -7.0],
            [1.0, -3.0, 7.0, 4.0],
        ]);
        let b = a.inverse().expect("unexpected failure inverting matrix");
        assert_eq!(a.determinant(), 532.0);
        assert_eq!(a.cofactor(2, 3), -160.0);
        assert_eq!(b[3][2], -160.0 / 532.0);
        assert_eq!(a.cofactor(3, 2), 105.0);
        assert_eq!(b[2][3], 105.0 / 532.0);
        let inverse = Matrix4x4([
            [0.21805, 0.45113, 0.24060, -0.04511],
            [-0.80827, -1.45677, -0.44361, 0.52068],
            [-0.07895, -0.22368, -0.05263, 0.19737],
            [-0.52256, -0.81391, -0.30075, 0.30639],
        ]);
        assert_eq!(b, inverse);
    }

    #[test]
    fn matrix_invert_additional_coverage() {
        let a = Matrix4x4([
            [8.0, -5.0, 9.0, 2.0],
            [7.0, 5.0, 6.0, 1.0],
            [-6.0, 0.0, 9.0, 6.0],
            [-3.0, 0.0, -9.0, -4.0],
        ]);
        let inverse_a = Matrix4x4([
            [-0.15385, -0.15385, -0.28205, -0.53846],
            [-0.07692, 0.12308, 0.02564, 0.03077],
            [0.35897, 0.35897, 0.43590, 0.92308],
            [-0.69231, -0.69231, -0.76923, -1.92308],
        ]);
        assert_eq!(a.inverse().unwrap(), inverse_a);

        let a = Matrix4x4([
            [9.0, 3.0, 0.0, 9.0],
            [-5.0, -2.0, -6.0, -3.0],
            [-4.0, 9.0, 6.0, 4.0],
            [-7.0, 6.0, 6.0, 2.0],
        ]);
        let inverse_a = Matrix4x4([
            [-0.04074, -0.07778, 0.14444, -0.22222],
            [-0.07778, 0.03333, 0.36667, -0.33333],
            [-0.02901, -0.14630, -0.10926, 0.12963],
            [0.17778, 0.06667, -0.26667, 0.33333],
        ]);
        assert_eq!(a.inverse().unwrap(), inverse_a);
    }

    #[test]
    fn matrix_inverse_multiply() {
        let a = Matrix4x4([
            [3.0, -9.0, 7.0, -3.0],
            [3.0, -8.0, 2.0, -9.0],
            [-4.0, 4.0, 4.0, 1.0],
            [-6.0, 5.0, -1.0, 1.0],
        ]);
        let b = Matrix4x4([
            [8.0, 2.0, 2.0, 2.0],
            [3.0, -1.0, 7.0, 0.0],
            [7.0, 0.0, 5.0, 4.0],
            [6.0, -2.0, 0.0, 5.0],
        ]);
        let c = a * b;
        assert_eq!(c * b.inverse().unwrap(), a);
    }
}
