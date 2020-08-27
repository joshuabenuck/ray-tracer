mod matrix;
mod ray;
mod scene;
mod tuple;

pub use matrix::{Matrix2x2, Matrix3x3, Matrix4x4};
pub use ray::{Intersection, Intersections, Material, PointLight, Ray, Sphere};
pub use scene::World;
pub use tuple::{Canvas, Color, Tuple};
