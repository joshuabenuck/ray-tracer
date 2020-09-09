use crate::{equal, v, Intersection, Material, Matrix4x4, Props, Ray, Shape, Tuple, EPSILON};
use std::any::Any;
use std::f64::{INFINITY, NEG_INFINITY};

#[derive(PartialEq, Debug)]
pub struct Cylinder {
    props: Props,
    min: f64,
    max: f64,
    closed: bool,
}

impl Cylinder {
    pub fn new(min: f64, max: f64, closed: bool) -> Cylinder {
        Cylinder {
            props: Props::default(),
            min,
            max,
            closed,
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

impl Default for Cylinder {
    fn default() -> Self {
        Cylinder::new(NEG_INFINITY, INFINITY, false)
    }
}

// a helper function to reduce duplication
// checks to see if the intersection at t is within a radius
// of 1 (the radius of the cylinders) from the x axis
fn check_cap(ray: &Ray, t: f64) -> bool {
    let x = ray.origin.x + t * ray.direction.x;
    let z = ray.origin.z + t * ray.direction.z;
    x.powi(2) + z.powi(2) <= 1.0
}

fn intersect_caps<'a>(cyl: &'a Cylinder, ray: &Ray, xs: &mut Vec<Intersection<'a>>) {
    let cyl = cyl;
    // caps only matter if the cylinder is closed and might
    // possibly be interesected by the ray.
    if !cyl.closed || equal(ray.direction.y, 0.0) {
        return;
    }

    // check for an intersection with the lower end cap by intersecting
    // the ray with the plane at y = min
    let t = (cyl.min - ray.origin.y) / ray.direction.y;
    if check_cap(ray, t) {
        xs.push(Intersection::new(t, cyl));
    }

    // check for an intersection with the upper end cap by intersecting
    // the ray with the plane at y = max
    let t = (cyl.max - ray.origin.y) / ray.direction.y;
    if check_cap(ray, t) {
        xs.push(Intersection::new(t, cyl));
    }
}

impl Shape for Cylinder {
    fn local_intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>> {
        let a = ray.direction.x.powi(2) + ray.direction.z.powi(2);

        // ray is parallel to the y axis
        if equal(a, 0.0) {
            let mut xs = Vec::new();
            intersect_caps(self, ray, &mut xs);
            return xs;
        }

        let b = 2.0 * ray.origin.x * ray.direction.x + 2.0 * ray.origin.z * ray.direction.z;
        let c = ray.origin.x.powi(2) + ray.origin.z.powi(2) - 1.0;

        let disc = b.powi(2) - 4.0 * a * c;

        // ray does not intersect the cylinder
        if disc < 0.0 {
            return Vec::new();
        }

        let mut t0 = (-b - disc.sqrt()) / (2.0 * a);
        let mut t1 = (-b + disc.sqrt()) / (2.0 * a);
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }

        let mut xs = Vec::new();

        let y0 = ray.origin.y + t0 * ray.direction.y;
        if self.min < y0 && y0 < self.max {
            xs.push(Intersection::new(t0, self));
        }

        let y1 = ray.origin.y + t1 * ray.direction.y;
        if self.min < y1 && y1 < self.max {
            xs.push(Intersection::new(t1, self));
        }
        intersect_caps(&self, ray, &mut xs);
        xs
    }

    fn local_normal_at(&self, local_point: Tuple, _i: &Intersection) -> Tuple {
        // compute the square of the distance from the y axis
        let dist = local_point.x.powi(2) + local_point.z.powi(2);

        if dist < 1.0 && local_point.y >= self.max - EPSILON {
            return v(0.0, 1.0, 0.0);
        } else if dist < 1.0 && local_point.y <= self.min + EPSILON {
            return v(0.0, -1.0, 0.0);
        }
        v(local_point.x, 0.0, local_point.z)
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

    fn includes(&self, other: &dyn Shape) -> bool {
        self as &dyn Shape == other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pt;

    #[test]
    fn ray_intersection_with_cylinder_misses() {
        // a ray misses a cylinder
        fn test(scenario: &str, origin: Tuple, direction: Tuple) {
            let cyl = Cylinder::default();
            let direction = direction.normalize();
            let r = Ray::new(origin, direction);
            let xs = cyl.local_intersect(&r);
            assert_eq!(xs.len(), 0, "{}", scenario);
        }
        test("one", pt(1.0, 0.0, 0.0), v(0.0, 1.0, 0.0));
        test("two", pt(0.0, 0.0, 0.0), v(0.0, 1.0, 0.0));
        test("three", pt(0.0, 0.0, -5.0), v(1.0, 1.0, 1.0));
    }

    #[test]
    fn ray_intersection_with_cylinder_hits() {
        // a ray strikes a cylinder
        fn test_hit(scenario: &str, origin: Tuple, direction: Tuple, t0: f64, t1: f64) {
            let cyl = Cylinder::default();
            let direction = direction.normalize();
            let r = Ray::new(origin, direction);
            let xs = cyl.local_intersect(&r);
            assert_eq!(xs.len(), 2, "{}", scenario);
            assert_eq!(equal(xs[0].t, t0), true, "{}", scenario);
            assert_eq!(equal(xs[1].t, t1), true, "{}", scenario);
        }
        test_hit("one", pt(1.0, 0.0, -5.0), v(0.0, 0.0, 1.0), 5.0, 5.0);
        test_hit("two", pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0), 4.0, 6.0);
        test_hit(
            "three",
            pt(0.5, 0.0, -5.0),
            v(0.1, 1.0, 1.0),
            6.80798,
            7.08872,
        );
    }

    #[test]
    fn ray_intersection_with_cylinder_constrained() {
        // intersecting a constrained cylinder
        fn test(scenario: &str, origin: Tuple, direction: Tuple, count: usize) {
            let cyl = Cylinder::new(1.0, 2.0, false);
            let direction = direction.normalize();
            let r = Ray::new(origin, direction);
            let xs = cyl.local_intersect(&r);
            assert_eq!(xs.len(), count, "{}", scenario);
        }

        test("1", pt(0.0, 1.5, 0.0), v(0.1, 1.0, 0.0), 0);
        test("2", pt(0.0, 3.0, -5.0), v(0.0, 0.0, 1.0), 0);
        test("3", pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0), 0);
        test("4", pt(0.0, 2.0, -5.0), v(0.0, 0.0, 1.0), 0);
        test("5", pt(0.0, 1.0, -5.0), v(0.0, 0.0, 1.0), 0);
        test("6", pt(0.0, 1.5, -2.0), v(0.0, 0.0, 1.0), 2);
    }

    #[test]
    fn ray_intersection_with_closed_cylinder() {
        // intersecting the caps of a closed cylinder
        fn test(scenario: &str, origin: Tuple, direction: Tuple, count: usize) {
            let cyl = Cylinder::new(1.0, 2.0, true);
            let direction = direction.normalize();
            let r = Ray::new(origin, direction);
            let xs = cyl.intersect(&r);
            assert_eq!(xs.len(), count, "{}", scenario);
        }

        test("1", pt(0.0, 3.0, 0.0), v(0.0, -1.0, 0.0), 2);
        test("2", pt(0.0, 3.0, -2.0), v(0.0, -1.0, 2.0), 2);
        // corner case
        test("3", pt(0.0, 4.0, -2.0), v(0.0, -1.0, 1.0), 2);
        test("4", pt(0.0, 0.0, -2.0), v(0.0, 1.0, 2.0), 2);
        // corner case
        test("5", pt(0.0, -1.0, -2.0), v(0.0, 1.0, 1.0), 2);
    }

    #[test]
    fn cylinder_normal_at() {
        // normal vector on a cylinder
        fn test(point: Tuple, normal: Tuple) {
            let cyl = Cylinder::default();
            let n = cyl.local_normal_at(point, &Intersection::new(0.0, &cyl));
            assert_eq!(n, normal);
        }
        test(pt(1.0, 0.0, 0.0), v(1.0, 0.0, 0.0));
        test(pt(0.0, 5.0, -1.0), v(0.0, 0.0, -1.0));
        test(pt(0.0, -2.0, 1.0), v(0.0, 0.0, 1.0));
        test(pt(-1.0, 1.0, 0.0), v(-1.0, 0.0, 0.0));
    }

    #[test]
    fn cylinder_end_cap_normal_at() {
        // the normal vector on a cylinder's end caps
        fn test(point: Tuple, normal: Tuple) {
            let cyl = Cylinder::new(1.0, 2.0, true);
            let n = cyl.local_normal_at(point, &Intersection::new(0.0, &cyl));
            assert_eq!(n, normal);
        }

        test(pt(0.0, 1.0, 0.0), v(0.0, -1.0, 0.0));
        test(pt(0.5, 1.0, 0.0), v(0.0, -1.0, 0.0));
        test(pt(0.0, 1.0, 0.5), v(0.0, -1.0, 0.0));
        test(pt(0.0, 2.0, 0.0), v(0.0, 1.0, 0.0));
        test(pt(0.5, 2.0, 0.0), v(0.0, 1.0, 0.0));
        test(pt(0.0, 2.0, 0.5), v(0.0, 1.0, 0.0));
    }
}
