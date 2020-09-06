mod intersection;
mod material;
mod matrix;
mod pattern;
mod ray;
mod scene;
mod shape;
mod tuple;

pub use intersection::{schlick, Comps, Intersection, Intersections};
pub use material::{lighting, m, Material};
pub use matrix::{id, Matrix2x2, Matrix3x3, Matrix4x4};
pub use pattern::{
    checkers_pattern, gradient_pattern, ring_pattern, stripe_pattern, stripe_patternt,
    test_pattern, Pattern,
};
pub use ray::{PointLight, Ray};
pub use scene::{view_transform, Camera, World};
pub use shape::{
    cube, cubem, cubet, cubetm, cylinder, cylinderm, cylindert, cylindertm, glass_sphere,
    glass_spheret, normal_at, plane, planem, planet, planetm, sphere, spherem, spheret, spheretm,
    test_shape, world_to_object, Cube, Cylinder, Shape, Sphere, Triangle,
};
pub use tuple::{black, pt, v, white, Canvas, Color, Tuple};

pub const EPSILON: f64 = 0.00001;

pub fn equal(a: f64, b: f64) -> bool {
    f64::abs(a - b) < EPSILON
}
