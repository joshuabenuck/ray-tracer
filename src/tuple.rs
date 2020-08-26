use std::cmp::{max, min};
use std::ops::{Add, Div, Mul, Neg, Sub};

const EPSILON: f64 = 0.00001;

#[derive(Debug, Clone, Copy)]
pub struct Tuple {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Tuple {
    pub fn point(x: f64, y: f64, z: f64) -> Tuple {
        Tuple { x, y, z, w: 1.0 }
    }

    pub fn vector(x: f64, y: f64, z: f64) -> Tuple {
        Tuple { x, y, z, w: 0.0 }
    }

    pub fn is_point(&self) -> bool {
        self.w == 1.0
    }

    pub fn is_vector(&self) -> bool {
        self.w == 0.0
    }

    /// Distance represented by a vector (called magnitude or length)
    /// It's how far you would travel in a straight line if you were to walk
    /// from one end of the vector to another.
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    pub fn normalize(self) -> Self {
        let magnitude = self.magnitude();
        Self {
            x: self.x / magnitude,
            y: self.y / magnitude,
            z: self.z / magnitude,
            w: self.w / magnitude,
        }
    }

    /// Dot product (or scalar product or inner product)
    /// Takes two vectors and returns a scalar value.
    /// Used when intersecting rays with objects.
    /// The smaller the dot product, the larger the angle between the vectors.
    /// A dot product of 1 means vectors are identical.
    /// -1 means they point in opposite directions.
    /// If two vectors are unit vectors, the dot product is the cosine of the
    /// angle between them.
    /// For more info: http://betterexplained.com/articles/vector-calculus-understanding-the-dot-product
    pub fn dot(&self, other: &Tuple) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn reflect(&self, normal: &Tuple) -> Tuple {
        *self - *normal * 2.0 * self.dot(normal)
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
    type Output = Tuple;

    fn add(self, other: Tuple) -> Tuple {
        Tuple {
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

impl Neg for Tuple {
    type Output = Tuple;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

/// Scalar multiplication - what point lies X times farther in the direction of the vector?
/// In t * 3.5, the 3.5 here is a scalar value because multiplying by it scales the vector
/// (changes its length uniformly).
impl Mul<f64> for Tuple {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

/// Cross product
/// TODO: Consider making Vector its own type since certain methods are only value on vectors.
/// Note: Four diensional cross product is significantly more complicated than this three diemnsional version.
///
/// If the order of the operands are changed, the direction of the resulting vector changes.
/// Returns a new vector that is perpendicular to both the original vectors.
///
/// X-axis cross Y-axis is Z axis, but Y-axis cross X-axis is -Z axis.
///
/// Cross products are primarily used when working with view transformations.
impl Mul for Tuple {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let a = self;
        let b = rhs;
        Tuple {
            x: a.y * b.z - a.z * b.y,
            y: a.z * b.x - a.x * b.z,
            z: a.x * b.y - a.y * b.x,
            w: a.w,
        }
    }
}

impl Div<f64> for Tuple {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w / rhs,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    red: f64,
    green: f64,
    blue: f64,
}

impl Color {
    pub fn new(red: f64, green: f64, blue: f64) -> Color {
        Color { red, green, blue }
    }
}

impl From<Color> for (usize, usize, usize) {
    fn from(color: Color) -> (usize, usize, usize) {
        let r = (color.red * 255.0).round() as isize;
        let g = (color.green * 255.0).round() as isize;
        let b = (color.blue * 255.0).round() as isize;
        (
            min(255, max(0, r)) as usize,
            min(255, max(0, g)) as usize,
            min(255, max(0, b)) as usize,
        )
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Color) -> bool {
        let a = self;
        let b = other;
        equal(a.red, b.red) && equal(a.green, b.green) && equal(a.blue, b.blue)
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color {
            red: self.red + other.red,
            green: self.green + other.green,
            blue: self.blue + other.blue,
        }
    }
}

impl Sub for Color {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            red: self.red - other.red,
            green: self.green - other.green,
            blue: self.blue - other.blue,
        }
    }
}

impl Mul<f64> for Color {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Self {
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
        }
    }
}

/// Hadamard product (or Schur product)
/// Multiply corresponding components of each color to form a new color.
impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let a = self;
        let b = rhs;
        Self {
            red: a.red * b.red,
            green: a.green * b.green,
            blue: a.blue * b.blue,
        }
    }
}

pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Canvas {
        let black = Color::new(0.0, 0.0, 0.0);
        Canvas {
            width,
            height,
            pixels: vec![black; width * height],
        }
    }

    fn xy(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, color: Color) {
        // TODO: Test
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = self.xy(x, y);
        self.pixels[idx] = color;
    }

    pub fn pixel_at(&self, x: usize, y: usize) -> Color {
        let idx = self.xy(x, y);
        self.pixels[idx]
    }

    pub fn to_ppm(&self) -> String {
        let mut ppm = format!("P3\n{} {}\n255\n", self.width, self.height);
        let width = self.width;
        for row in self.pixels.chunks(width) {
            let mut i = 0;
            for pixel in row.iter() {
                let (red, green, blue) = Into::<(usize, usize, usize)>::into(*pixel);
                let mut append = |number: usize| {
                    let text = number.to_string();
                    let len = text.len();
                    if i == 0 {
                    } else if len + i + 1 > 70 {
                        ppm += "\n";
                        i = 0;
                    } else {
                        ppm += " ";
                        i += 1;
                    }
                    ppm += &text;
                    i += len;
                };
                append(red);
                append(green);
                append(blue);
            }
            ppm += "\n";
        }
        ppm
    }
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
    let a4 = a3 + a3;
    assert_eq!(false, a4.is_point());
    assert_eq!(false, a4.is_vector());
}

#[test]
fn tuple_subtraction() {
    let p1 = Tuple::point(3.0, 2.0, 1.0);
    let p2 = Tuple::point(5.0, 6.0, 7.0);
    assert_eq!(p1 - p2, Tuple::vector(-2.0, -4.0, -6.0));

    let p = Tuple::point(3.0, 2.0, 1.0);
    let v = Tuple::vector(5.0, 6.0, 7.0);
    assert_eq!(p - v, Tuple::point(-2.0, -4.0, -6.0));

    let v1 = Tuple::vector(3.0, 2.0, 1.0);
    let v2 = Tuple::vector(5.0, 6.0, 7.0);
    assert_eq!(v1 - v2, Tuple::vector(-2.0, -4.0, -6.0));

    let p = Tuple::point(3.0, 2.0, 1.0);
    let v = Tuple::vector(5.0, 6.0, 7.0);
    let invalid = v - p;
    assert_eq!(false, invalid.is_point());
    assert_eq!(false, invalid.is_vector());
}

#[test]
fn tuple_negation() {
    let zero = Tuple::vector(0.0, 0.0, 0.0);
    let v = Tuple::vector(1.0, -2.0, 3.0);
    assert_eq!(zero - v, Tuple::vector(-1.0, 2.0, -3.0));

    let a: Tuple = (1.0, -2.0, 3.0, -4.0).into();
    assert_eq!(-a, (-1.0, 2.0, -3.0, 4.0).into())
}

#[test]
fn tuple_multiplication() {
    let a: Tuple = (1.0, -2.0, 3.0, -4.0).into();
    assert_eq!(a * 3.5, (3.5, -7.0, 10.5, -14.0).into());

    let a: Tuple = (1.0, -2.0, 3.0, -4.0).into();
    assert_eq!(a * 0.5, (0.5, -1.0, 1.5, -2.0).into());
}

#[test]
fn tuple_division() {
    let a: Tuple = (1.0, -2.0, 3.0, -4.0).into();
    assert_eq!(a / 2.0, (0.5, -1.0, 1.5, -2.0).into());
}

#[test]
fn vector_magnitude() {
    let v = Tuple::vector(1.0, 0.0, 0.0);
    assert_eq!(v.magnitude(), 1.0);

    let v = Tuple::vector(0.0, 1.0, 0.0);
    assert_eq!(v.magnitude(), 1.0);

    let v = Tuple::vector(0.0, 0.0, 1.0);
    assert_eq!(v.magnitude(), 1.0);

    let v = Tuple::vector(1.0, 2.0, 3.0);
    assert_eq!(v.magnitude(), (14.0_f64).sqrt());

    let v = Tuple::vector(-1.0, -2.0, -3.0);
    assert_eq!(v.magnitude(), (14.0_f64).sqrt());
}

#[test]
fn vector_normalization() {
    let v = Tuple::vector(4.0, 0.0, 0.0);
    assert_eq!(v.normalize(), Tuple::vector(1.0, 0.0, 0.0));

    let v = Tuple::vector(1.0, 2.0, 3.0);
    // 1/14.sqrt, 2/14.sqrt, 3/14.sqrt
    assert_eq!(v.normalize(), Tuple::vector(0.26726, 0.53452, 0.80178));
}

#[test]
fn vector_dot_product() {
    let a = Tuple::vector(1.0, 2.0, 3.0);
    let b = Tuple::vector(2.0, 3.0, 4.0);
    assert_eq!(a.dot(&b), 20.0);
}

#[test]
fn vector_cross_product() {
    let a = Tuple::vector(1.0, 2.0, 3.0);
    let b = Tuple::vector(2.0, 3.0, 4.0);
    assert_eq!(a * b, Tuple::vector(-1.0, 2.0, -1.0));
    assert_eq!(b * a, Tuple::vector(1.0, -2.0, 1.0));
}

#[test]
fn color() {
    let c = Color::new(-0.5, 0.4, 1.7);
    assert_eq!(c.red, -0.5);
    assert_eq!(c.green, 0.4);
    assert_eq!(c.blue, 1.7);
}

#[test]
fn color_ops() {
    let c1 = Color::new(0.9, 0.6, 0.75);
    let c2 = Color::new(0.7, 0.1, 0.25);
    assert_eq!(c1 + c2, Color::new(1.6, 0.7, 1.0));
    assert_eq!(c1 - c2, Color::new(0.2, 0.5, 0.5));

    let c = Color::new(0.2, 0.3, 0.4);
    assert_eq!(c * 2.0, Color::new(0.4, 0.6, 0.8));

    let c1 = Color::new(1.0, 0.2, 0.4);
    let c2 = Color::new(0.9, 1.0, 0.1);
    assert_eq!(c1 * c2, Color::new(0.9, 0.2, 0.04));
}

#[test]
fn canvas_create() {
    let mut c = Canvas::new(10, 20);
    assert_eq!(c.width, 10);
    assert_eq!(c.height, 20);
    let black = Color::new(0.0, 0.0, 0.0);
    for pixel in &c.pixels {
        assert_eq!(pixel, &black);
    }

    let red = Color::new(1.0, 0.0, 0.0);
    c.write_pixel(2, 3, red);
    assert_eq!(c.pixel_at(2, 3), red);
}

#[test]
fn canvs_to_ppm() {
    let c = Canvas::new(5, 3);
    let ppm = c.to_ppm();
    let first_three = ["P3", "5 3", "255"];
    assert_eq!(ppm.split("\n").take(3).collect::<Vec<&str>>(), first_three);

    let mut c = Canvas::new(5, 3);
    let c1 = Color::new(1.5, 0.0, 0.0);
    let c2 = Color::new(0.0, 0.5, 0.0);
    let c3 = Color::new(-0.5, 0.0, 1.0);
    c.write_pixel(0, 0, c1);
    c.write_pixel(2, 1, c2);
    c.write_pixel(4, 2, c3);
    let four_to_six = [
        "255 0 0 0 0 0 0 0 0 0 0 0 0 0 0",
        "0 0 0 0 0 0 0 128 0 0 0 0 0 0 0",
        "0 0 0 0 0 0 0 0 0 0 0 0 0 0 255",
    ];
    let ppm = c.to_ppm();
    assert_eq!(
        ppm.split("\n").skip(3).take(3).collect::<Vec<&str>>(),
        four_to_six
    );

    let mut c = Canvas::new(10, 2);
    c.pixels = vec![Color::new(1.0, 0.8, 0.6); c.width * c.height];
    let four_to_seven = [
        "255 204 153 255 204 153 255 204 153 255 204 153 255 204 153 255 204",
        "153 255 204 153 255 204 153 255 204 153 255 204 153",
        "255 204 153 255 204 153 255 204 153 255 204 153 255 204 153 255 204",
        "153 255 204 153 255 204 153 255 204 153 255 204 153",
    ];
    let ppm = c.to_ppm();
    assert_eq!(
        ppm.split("\n").skip(3).take(4).collect::<Vec<&str>>(),
        four_to_seven
    );

    let c = Canvas::new(5, 3);
    let ppm = c.to_ppm();
    assert_eq!(ppm.as_bytes()[ppm.len() - 1], '\n' as u8);
}

#[test]
fn vector_reflection() {
    // reflecting a vector approaching at 45 degrees
    let v = Tuple::vector(1.0, -1.0, 0.0);
    let n = Tuple::vector(0.0, 1.0, 0.0);
    let r = v.reflect(&n);
    assert_eq!(r, Tuple::vector(1.0, 1.0, 0.0));

    // reflecting a vector off a slanted surface
    let v = Tuple::vector(0.0, -1.0, 0.0);
    let n = Tuple::vector(2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0, 0.0);
    let r = v.reflect(&n);
    assert_eq!(r, Tuple::vector(1.0, 0.0, 0.0));
}
