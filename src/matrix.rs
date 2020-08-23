use crate::Tuple;
use std::ops::{Index, Mul};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Matrix([[f64; 4]; 4]);

impl Matrix {
    fn identity() -> Matrix {
        Matrix([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Used when translating normal vectors between object space and world space
    fn transpose(&self) -> Matrix {
        let a = self.0;
        let mut ta = [[0.0; 4]; 4];
        for row in 0..4 {
            for col in 0..4 {
                ta[row][col] = a[col][row];
            }
        }
        Matrix(ta)
    }
}

impl Index<usize> for Matrix {
    type Output = [f64; 4];

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

// Matrix multiplication computes the dot product of every row-column combination
impl Mul for Matrix {
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
        Matrix(m)
    }
}

impl Mul<Tuple> for Matrix {
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
        let m = Matrix([
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

        let m = [[-3.0, 5.0], [1.0, -2.0]];
        assert_eq!(m[0][0], -3.0);
        assert_eq!(m[0][1], 5.0);
        assert_eq!(m[1][0], 1.0);
        assert_eq!(m[1][1], -2.0);

        let m = [[-3.0, 5.0, 0.0], [1.0, -2.0, -7.0], [0.0, 1.0, 1.0]];
        assert_eq!(m[0][0], -3.0);
        assert_eq!(m[1][1], -2.0);
        assert_eq!(m[2][2], 1.0);
    }

    #[test]
    fn matrix_compare() {
        let a = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ];
        let b = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ];
        assert_eq!(a, b);
        let b = [
            [2.0, 3.0, 4.0, 5.0],
            [6.0, 7.0, 8.0, 9.0],
            [8.0, 7.0, 6.0, 5.0],
            [4.0, 3.0, 2.0, 1.0],
        ];
        assert_ne!(a, b);
    }

    #[test]
    fn matrix_multiply() {
        let a = Matrix([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ]);
        let b = Matrix([
            [-2.0, 1.0, 2.0, 3.0],
            [3.0, 2.0, 1.0, -1.0],
            [4.0, 3.0, 6.0, 5.0],
            [1.0, 2.0, 7.0, 8.0],
        ]);
        let ab = Matrix([
            [20.0, 22.0, 50.0, 48.0],
            [44.0, 54.0, 114.0, 108.0],
            [40.0, 58.0, 110.0, 102.0],
            [16.0, 26.0, 46.0, 42.0],
        ]);
        assert_eq!(a * b, ab);

        let a = Matrix([
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
        let a = Matrix([
            [0.0, 1.0, 2.0, 4.0],
            [1.0, 2.0, 4.0, 8.0],
            [2.0, 4.0, 8.0, 16.0],
            [4.0, 8.0, 16.0, 32.0],
        ]);

        assert_eq!(a * Matrix::identity(), a);

        let a: Tuple = (1.0, 2.0, 3.0, 4.0).into();
        assert_eq!(Matrix::identity() * a, a);
    }

    #[test]
    fn matrix_transpose() {
        let a = Matrix([
            [0.0, 9.0, 3.0, 0.0],
            [9.0, 8.0, 0.0, 8.0],
            [1.0, 8.0, 5.0, 3.0],
            [0.0, 0.0, 5.0, 8.0],
        ]);
        let ta = Matrix([
            [0.0, 9.0, 1.0, 0.0],
            [9.0, 8.0, 8.0, 0.0],
            [3.0, 0.0, 5.0, 5.0],
            [0.0, 8.0, 3.0, 8.0],
        ]);
        assert_eq!(a.transpose(), ta);
        assert_eq!(Matrix::identity().transpose(), Matrix::identity());
    }

    #[test]
    fn matrix_determinant() {
        let a = [[1.0, 5.0], [-3.0, 2.0]];
        // Not sure this will be needed elsewhere.
        let determinant = a[0][0] * a[1][1] - a[0][1] * a[1][0];
        assert_eq!(determinant, 17.0);
    }
}
