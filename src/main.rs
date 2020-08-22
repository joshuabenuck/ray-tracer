use std::ops::{Add, Sub};

const EPSILON: f64 = 0.00001;

#[derive(Debug, Clone)]
struct Tuple {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl Tuple {
    fn point(x: f64, y: f64, z: f64) -> Tuple {
        Tuple { x, y, z, w: 1.0 }
    }

    fn vector(x: f64, y: f64, z: f64) -> Tuple {
        Tuple { x, y, z, w: 0.0 }
    }

    fn is_point(&self) -> bool {
        self.w == 1.0
    }

    fn is_vector(&self) -> bool {
        self.w == 0.0
    }
}

impl From<(f64, f64, f64, f64)> for Tuple {
    fn from(value: (f64, f64, f64, f64)) -> Tuple {
        Tuple {
            x: value.0,
            y: value.1,
            z: value.2,
            w: value.3,
        }
    }
}

fn equal(a: f64, b: f64) -> bool {
    f64::abs(a - b) < EPSILON
}

impl PartialEq for Tuple {
    fn eq(&self, other: &Tuple) -> bool {
        let a = self;
        let b = other;
        equal(a.x, b.x) && equal(a.y, b.y) && equal(a.z, b.z) && equal(a.w, b.w)
    }
}

impl Add for Tuple {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl Sub for Tuple {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

fn main() {
    println!("Hello, world!");
}

#[test]
fn tuple_is_point() {
    let a: Tuple = (4.3, -4.2, 3.1, 1.0).into();
    assert_eq!(4.3, a.x);
    assert_eq!(-4.2, a.y);
    assert_eq!(3.1, a.z);
    assert_eq!(1.0, a.w);
    assert_eq!(true, a.is_point());
    assert_eq!(false, a.is_vector());
}

#[test]
fn tuple_is_vector() {
    let a: Tuple = (4.3, -4.2, 3.1, 0.0).into();
    assert_eq!(4.3, a.x);
    assert_eq!(-4.2, a.y);
    assert_eq!(3.1, a.z);
    assert_eq!(0.0, a.w);
    assert_eq!(false, a.is_point());
    assert_eq!(true, a.is_vector());
}

#[test]
fn tuple_point() {
    let p = Tuple::point(4.0, -4.0, 3.0);
    let t = (4.0, -4.0, 3.0, 1.0).into();
    assert_eq!(p, t);
}

#[test]
fn tuple_vector() {
    let v = Tuple::vector(4.0, -4.0, 3.0);
    let t = (4.0, -4.0, 3.0, 0.0).into();
    assert_eq!(v, t);
}

#[test]
fn tuple_addition() {
    let a1: Tuple = (3.0, -2.0, 5.0, 1.0).into();
    let a2: Tuple = (-2.0, 3.0, 1.0, 0.0).into();
    let a3: Tuple = (3.0, -2.0, 5.0, 1.0).into();
    assert_eq!(a1 + a2, (1.0, 1.0, 6.0, 1.0).into());
    let a4 = a3.clone() + a3;
    assert_eq!(false, a4.is_point());
    assert_eq!(false, a4.is_vector());
}

#[test]
fn tuple_subtraction() {
    let p1 = Tuple::point(3.0, 2.0, 1.0);
    let p2 = Tuple::point(5.0, 6.0, 7.0);
    assert_eq!(p1 - p2, Tuple::vector(-2.0, -4.0, -6.0));
}
