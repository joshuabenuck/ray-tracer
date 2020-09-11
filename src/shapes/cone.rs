use crate::{equal, v, Intersection, Material, Matrix4x4, Props, Ray, Shape, Tuple, EPSILON};
use std::any::Any;
use std::f64::{INFINITY, NEG_INFINITY};

#[derive(PartialEq, Debug)]
pub struct Cone {
    props: Props,
    min: f64,
    max: f64,
    closed: bool,
}

impl Cone {
    pub fn new(min: f64, max: f64, closed: bool) -> Cone {
        Cone {
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

impl Default for Cone {
    fn default() -> Self {
        Cone::new(NEG_INFINITY, INFINITY, false)
    }
}

// a helper function to reduce duplication
// checks to see if the intersection at t is within a radius
// of y (the radius of the cone) from the x axis
fn check_cap(ray: &Ray, t: f64, y: f64) -> bool {
    let x = ray.origin.x + t * ray.direction.x;
    let z = ray.origin.z + t * ray.direction.z;
    x.powi(2) + z.powi(2) <= y.powi(2)
}

fn intersect_caps<'a>(cone: &'a Cone, ray: &Ray, xs: &mut Vec<Intersection<'a>>) {
    let cone = cone;
    // caps only matter if the cone is closed and might
    // possibly be interesected by the ray.
    if !cone.closed || equal(ray.direction.y.abs(), 0.0) {
        return;
    }

    // check for an intersection with the lower end cap by intersecting
    // the ray with the plane at y = min
    let t = (cone.min - ray.origin.y) / ray.direction.y;
    if check_cap(ray, t, cone.min.abs()) {
        xs.push(Intersection::new(t, cone));
    }

    // check for an intersection with the upper end cap by intersecting
    // the ray with the plane at y = max
    let t = (cone.max - ray.origin.y) / ray.direction.y;
    if check_cap(ray, t, cone.max.abs()) {
        xs.push(Intersection::new(t, cone));
    }
}

impl Shape for Cone {
    fn local_intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>> {
        let a = ray.direction.x.powi(2) - ray.direction.y.powi(2) + ray.direction.z.powi(2);
        let b = 2.0 * ray.origin.x * ray.direction.x - 2.0 * ray.origin.y * ray.direction.y
            + 2.0 * ray.origin.z * ray.direction.z;
        let c = ray.origin.x.powi(2) - ray.origin.y.powi(2) + ray.origin.z.powi(2);

        // ray misses cone
        if equal(a.abs(), 0.0) {
            if equal(b.abs(), 0.0) {
                return Vec::new();
            } else {
                let t = -c / (2.0 * b);
                let mut xs = vec![Intersection::new(t, self)];
                intersect_caps(self, ray, &mut xs);
                return xs;
            }
        }

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

        let mut y = dist.sqrt();
        if local_point.y > 0.0 {
            y = -y;
        }
        v(local_point.x, y, local_point.z)
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
    fn ray_intersection_with_cone_hits() {
        // intersecting a cone with a ray
        fn test_hit(scenario: &str, origin: Tuple, direction: Tuple, t0: f64, t1: f64) {
            let shape = Cone::default();
            let direction = direction.normalize();
            let r = Ray::new(origin, direction);
            let xs = shape.local_intersect(&r);
            assert_eq!(xs.len(), 2, "{}", scenario);
            assert_eq!(equal(xs[0].t, t0), true, "{}", scenario);
            assert_eq!(equal(xs[1].t, t1), true, "{}", scenario);
        }
        test_hit("one", pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0), 5.0, 5.0);
        test_hit(
            "two",
            pt(0.0, 0.0, -5.0),
            v(1.0, 1.0, 1.0),
            8.66025,
            8.66025,
        );
        test_hit(
            "three",
            pt(1.0, 1.0, -5.0),
            v(-0.5, -1.0, 1.0),
            4.55006,
            49.44994,
        );
    }

    #[test]
    fn ray_intersection_with_cone_one_half() {
        // intersecting a cone with a ray parallel to one of its halves
        let shape = Cone::new(NEG_INFINITY, INFINITY, false);
        let direction = v(0.0, 1.0, 1.0).normalize();
        let r = Ray::new(pt(0.0, 0.0, -1.0), direction);
        let xs = shape.local_intersect(&r);
        assert_eq!(xs.len(), 1);
        assert!(equal(xs[0].t, 0.35355));
    }

    #[test]
    fn ray_intersection_with_cone_end_caps() {
        // intersecting the caps of a closed cone
        fn test(scenario: &str, origin: Tuple, direction: Tuple, count: usize) {
            let cone = Cone::new(-0.5, 0.5, true);
            let direction = direction.normalize();
            let r = Ray::new(origin, direction);
            let xs = cone.intersect(&r);
            assert_eq!(xs.len(), count, "{}", scenario);
        }

        test("1", pt(0.0, 0.0, -5.0), v(0.0, 1.0, 0.0), 0);
        test("2", pt(0.0, 0.0, -0.25), v(0.0, 1.0, 1.0), 2);
        test("3", pt(0.0, 0.0, -0.25), v(0.0, 1.0, 0.0), 4);
    }

    #[test]
    fn cone_normal_at() {
        // normal vector on a cylinder
        fn test(point: Tuple, normal: Tuple) {
            let shape = Cone::default();
            let n = shape.local_normal_at(point, &Intersection::new(0.0, &shape));
            assert_eq!(n, normal);
        }
        test(pt(0.0, 0.0, 0.0), v(0.0, 0.0, 0.0));
        test(pt(1.0, 1.0, 1.0), v(1.0, -2.0_f64.sqrt(), 1.0));
        test(pt(-1.0, -1.0, 0.0), v(-1.0, 1.0, 0.0));
    }

    // #[test]
    // fn cylinder_end_cap_normal_at() {
    //     // the normal vector on a cylinder's end caps
    //     fn test(point: Tuple, normal: Tuple) {
    //         let cyl = Cylinder::new(1.0, 2.0, true);
    //         let n = cyl.local_normal_at(point, &Intersection::new(0.0, &cyl));
    //         assert_eq!(n, normal);
    //     }

    //     test(pt(0.0, 1.0, 0.0), v(0.0, -1.0, 0.0));
    //     test(pt(0.5, 1.0, 0.0), v(0.0, -1.0, 0.0));
    //     test(pt(0.0, 1.0, 0.5), v(0.0, -1.0, 0.0));
    //     test(pt(0.0, 2.0, 0.0), v(0.0, 1.0, 0.0));
    //     test(pt(0.5, 2.0, 0.0), v(0.0, 1.0, 0.0));
    //     test(pt(0.0, 2.0, 0.5), v(0.0, 1.0, 0.0));
    // }
}
