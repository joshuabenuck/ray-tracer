use crate::{v, Intersection, Material, Matrix4x4, Props, Ray, Shape, Tuple, EPSILON};
use std::any::Any;

#[derive(PartialEq, Debug)]
pub struct Cube {
    props: Props,
}

impl Cube {
    pub fn new() -> Cube {
        Cube {
            props: Props::default(),
        }
    }

    pub fn transform(mut self, transform: Matrix4x4) -> Self {
        self.props.transform = transform;
        self
    }

    pub fn material(mut self, material: Material) -> Self {
        self.props.material = material;
        self
    }

    pub fn shape(self) -> Box<dyn Shape> {
        Box::new(self)
    }
}

fn check_axis(origin: f64, direction: f64) -> (f64, f64) {
    let tmin_numerator = -1.0 - origin;
    let tmax_numerator = 1.0 - origin;

    let (mut tmin, mut tmax) = if direction.abs() >= EPSILON {
        (tmin_numerator / direction, tmax_numerator / direction)
    } else {
        (
            tmin_numerator * std::f64::INFINITY,
            tmax_numerator * std::f64::INFINITY,
        )
    };

    if tmin > tmax {
        std::mem::swap(&mut tmin, &mut tmax);
    }

    (tmin, tmax)
}

impl Shape for Cube {
    fn local_intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>> {
        // How to avoid always doing all of these computations?
        let (xtmin, xtmax) = check_axis(ray.origin.x, ray.direction.x);
        let (ytmin, ytmax) = check_axis(ray.origin.y, ray.direction.y);
        let (ztmin, ztmax) = check_axis(ray.origin.z, ray.direction.z);

        let tmin = xtmin.max(ytmin).max(ztmin);
        let tmax = xtmax.min(ytmax).min(ztmax);

        if tmin > tmax {
            return Vec::new();
        }

        vec![Intersection::new(tmin, self), Intersection::new(tmax, self)]
    }

    fn local_normal_at(&self, local_point: Tuple, _i: &Intersection) -> Tuple {
        let maxc = local_point
            .x
            .abs()
            .max(local_point.y.abs())
            .max(local_point.z.abs());

        if maxc == local_point.x.abs() {
            v(local_point.x, 0.0, 0.0)
        } else if maxc == local_point.y.abs() {
            v(0.0, local_point.y, 0.0)
        } else {
            v(0.0, 0.0, local_point.z)
        }
    }

    fn common(&self) -> &Props {
        &self.props
    }

    fn common_mut(&mut self) -> &mut Props {
        &mut self.props
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn shape_eq(&self, other: &dyn Any) -> bool {
        match other.downcast_ref::<Self>() {
            Some(_) => true,
            None => false,
        }
    }
}

impl<'a> From<Cube> for Box<dyn Shape + 'a> {
    fn from(value: Cube) -> Box<dyn Shape + 'a> {
        Box::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pt;

    #[test]
    fn ray_intersection_with_cube() {
        // a ray intersects a cube
        fn test(origin: Tuple, direction: Tuple, t1: f64, t2: f64) {
            let c = Cube::new();
            let r = Ray::new(origin, direction);
            let xs = c.local_intersect(&r);
            assert_eq!(xs.len(), 2);
            assert_eq!(xs[0].t, t1);
            assert_eq!(xs[1].t, t2);
        }
        // +x
        test(pt(5.0, 0.5, 0.0), v(-1.0, 0.0, 0.0), 4.0, 6.0);
        // -x
        test(pt(-5.0, 0.5, 0.0), v(1.0, 0.0, 0.0), 4.0, 6.0);
        // +y
        test(pt(0.5, 5.0, 0.0), v(0.0, -1.0, 0.0), 4.0, 6.0);
        // -y
        test(pt(0.5, -5.0, 0.0), v(0.0, 1.0, 0.0), 4.0, 6.0);
        // +z
        test(pt(0.5, 0.0, 5.0), v(0.0, 0.0, -1.0), 4.0, 6.0);
        // -z
        test(pt(0.5, 0.0, -5.0), v(0.0, 0.0, 1.0), 4.0, 6.0);
        // insidee
        test(pt(0.0, 0.5, 0.0), v(0.0, 0.0, 1.0), -1.0, 1.0);

        fn test_miss(origin: Tuple, direction: Tuple) {
            let c = Cube::new();
            let r = Ray::new(origin, direction);
            let xs = c.local_intersect(&r);
            assert_eq!(xs.len(), 0);
        }
        test_miss(pt(-2.0, 0.0, 0.0), v(0.2673, 0.5345, 0.8018));
        test_miss(pt(0.0, -2.0, 0.0), v(0.8018, 0.2673, 0.5345));
        test_miss(pt(0.0, 0.0, -2.0), v(0.5345, 0.8018, 0.2673));
        test_miss(pt(2.0, 0.0, 2.0), v(0.0, 0.0, -1.0));
        test_miss(pt(0.0, 2.0, 2.0), v(0.0, -1.0, 0.0));
        test_miss(pt(2.0, 0.0, 2.0), v(-1.0, 0.0, 0.0));
    }

    #[test]
    fn cube_normal_at() {
        // the normal of the surface of a cube
        fn test(point: Tuple, normal: Tuple) {
            let c = Cube::new();
            let n = c.normal_at(point, &Intersection::new(0.0, &c));
            assert_eq!(n, normal);
        }

        test(pt(1.0, 0.5, -0.8), v(1.0, 0.0, 0.0));
        test(pt(-1.0, -0.2, 0.9), v(-1.0, 0.0, 0.0));
        test(pt(-0.4, 1.0, -0.1), v(0.0, 1.0, 0.0));
        test(pt(0.3, -1.0, -0.7), v(0.0, -1.0, 0.0));
        test(pt(0.6, 0.3, 1.0), v(0.0, 0.0, 1.0));
        test(pt(0.4, 0.4, -1.0), v(0.0, 0.0, -1.0));
        // normal at cube's corners
        test(pt(1.0, 1.0, 1.0), v(1.0, 0.0, 0.0));
        test(pt(-1.0, -1.0, -1.0), v(-1.0, 0.0, 0.0));
    }
}
