mod matrix;
mod ray;
mod scene;
mod tuple;

pub use matrix::{Matrix2x2, Matrix3x3, Matrix4x4};
pub use ray::{Comps, Intersection, Intersections, Material, PointLight, Ray, Sphere};
pub use scene::{view_transform, Camera, World};
pub use tuple::{pt, v, Canvas, Color, Tuple};

pub const EPSILON: f64 = 0.00001;
