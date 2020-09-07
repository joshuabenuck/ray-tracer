use crate::{Intersection, Props, Ray, Shape, Tuple, EPSILON};
use std::any::Any;

#[derive(PartialEq)]
pub struct Triangle {
    props: Props,
    p1: Tuple,
    p2: Tuple,
    p3: Tuple,
    e1: Tuple,
    e2: Tuple,
    normal: Tuple,
}

impl Triangle {
    pub fn new(p1: Tuple, p2: Tuple, p3: Tuple) -> Triangle {
        let e1 = p2 - p1;
        let e2 = p3 - p1;
        let normal = (e2 * e1).normalize();
        Triangle {
            props: Props::default(),
            p1,
            p2,
            p3,
            e1,
            e2,
            normal,
        }
    }
}

impl Shape for Triangle {
    fn local_intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>> {
        let dir_cross_e2 = ray.direction * self.e2;
        let determinant = self.e1.dot(&dir_cross_e2);
        if determinant.abs() < EPSILON {
            return Vec::new();
        }

        let f = 1.0 / determinant;

        let p1_to_origin = ray.origin - self.p1;
        let u = f * p1_to_origin.dot(&dir_cross_e2);
        if u < 0.0 || u > 1.0 {
            return Vec::new();
        }

        let origin_cross_e1 = p1_to_origin * self.e1;
        let v = f * ray.direction.dot(&origin_cross_e1);
        if v < 0.0 || (u + v) > 1.0 {
            return Vec::new();
        }

        let t = f * self.e2.dot(&origin_cross_e1);
        vec![Intersection::new(t, self)]
    }

    fn local_normal_at(&self, _local_point: Tuple) -> Tuple {
        self.normal
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

impl<'a> From<Triangle> for Box<dyn Shape + 'a> {
    fn from(value: Triangle) -> Box<dyn Shape + 'a> {
        Box::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{pt, v};

    #[test]
    fn triangle_create() {
        // constructing a triangle
        let p1 = pt(0.0, 1.0, 0.0);
        let p2 = pt(-1.0, 0.0, 0.0);
        let p3 = pt(1.0, 0.0, 0.0);
        let t = Triangle::new(p1, p2, p3);

        assert_eq!(t.p1, p1);
        assert_eq!(t.p2, p2);
        assert_eq!(t.p3, p3);
        assert_eq!(t.p3, p3);
        assert_eq!(t.e1, v(-1.0, -1.0, 0.0));
        assert_eq!(t.e2, v(1.0, -1.0, 0.0));
        assert_eq!(t.normal, v(0.0, 0.0, -1.0));
    }

    #[test]
    fn triangle_normal_at() {
        // finding the normal on a triangle
        let t = Triangle::new(pt(0.0, 1.0, 0.0), pt(-1.0, 0.0, 0.0), pt(1.0, 0.0, 0.0));
        let n1 = t.local_normal_at(pt(0.0, 0.5, 0.0));
        let n2 = t.local_normal_at(pt(-0.5, 0.75, 0.0));
        let n3 = t.local_normal_at(pt(0.5, 0.25, 0.0));

        assert_eq!(n1, t.normal);
        assert_eq!(n2, t.normal);
        assert_eq!(n3, t.normal);
    }

    #[test]
    fn ray_intersect_triangle() {
        // intersecting a ray parllel to the triangle
        let t = Triangle::new(pt(0.0, 1.0, 0.0), pt(-1.0, 0.0, 0.0), pt(1.0, 0.0, 0.0));
        let r = Ray::new(pt(0.0, -1.0, -2.0), v(0.0, 1.0, 0.0));
        let xs = t.local_intersect(&r);
        assert_eq!(xs.len(), 0);

        // a ray misses the p1-p3 edge
        let r = Ray::new(pt(1.0, 1.0, -2.0), v(0.0, 0.0, 1.0));
        let xs = t.local_intersect(&r);
        assert_eq!(xs.len(), 0);

        // a ray misses the p1-p2 edge
        let r = Ray::new(pt(-1.0, 1.0, -2.0), v(0.0, 0.0, 1.0));
        let xs = t.local_intersect(&r);
        assert_eq!(xs.len(), 0);

        // a ray misses the p2-p3 edge
        let r = Ray::new(pt(0.0, -1.0, -2.0), v(0.0, 0.0, 1.0));
        let xs = t.local_intersect(&r);
        assert_eq!(xs.len(), 0);

        // a ray strikes a triangle
        let r = Ray::new(pt(0.0, 0.5, -2.0), v(0.0, 0.0, 1.0));
        let xs = t.local_intersect(&r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 2.0);
    }
}
