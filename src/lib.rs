mod matrix;
mod pattern;
mod ray;
mod scene;
mod tuple;

pub use matrix::{Matrix2x2, Matrix3x3, Matrix4x4};
pub use pattern::{
    checkers_pattern, gradient_pattern, ring_pattern, stripe_pattern, test_pattern, Pattern,
};
pub use ray::{
    glass_sphere, glass_spheret, plane, planem, planet, planetm, schlick, sphere, spherem, spheret, spheretm, Comps,
    Intersection, Intersections, Material, PointLight, Ray, Shape,
};
pub use scene::{view_transform, Camera, World};
pub use tuple::{black, pt, v, white, Canvas, Color, Tuple};

pub const EPSILON: f64 = 0.00001;

pub fn equal(a: f64, b: f64) -> bool {
    f64::abs(a - b) < EPSILON
}
