use crate::{Intersection, Props, Ray, Shape, Tuple, EPSILON};
use std::any::Any;

#[derive(PartialEq, Debug)]
pub enum Normal {
    Default(Tuple),
    Smooth(Tuple, Tuple, Tuple),
}

#[derive(PartialEq, Debug)]
pub struct Triangle {
    props: Props,
    pub p1: Tuple,
    pub p2: Tuple,
    pub p3: Tuple,
    pub e1: Tuple,
    pub e2: Tuple,
    pub normal: Normal,
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
            normal: Normal::Default(normal),
        }
    }

    pub fn smooth(p1: Tuple, p2: Tuple, p3: Tuple, n1: Tuple, n2: Tuple, n3: Tuple) -> Triangle {
        let e1 = p2 - p1;
        let e2 = p3 - p1;
        Triangle {
            props: Props::default(),
            p1,
            p2,
            p3,
            normal: Normal::Smooth(n1, n2, n3),
            e1,
            e2,
        }
    }

    pub fn shape(self) -> Box<dyn Shape> {
        Box::new(self)
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
        vec![Intersection::with_uv(t, self, u, v)]
    }

    fn local_normal_at(&self, _local_point: Tuple, i: &Intersection) -> Tuple {
        match self.normal {
            Normal::Default(normal) => normal,
            Normal::Smooth(n1, n2, n3) => {
                if let Some((u, v)) = i.uv {
                    n2 * u + n3 * v + n1 * (1.0 - u - v)
                } else {
                    panic!("uv not set on intersection!")
                }
            }
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

impl<'a> From<Triangle> for Box<dyn Shape + 'a> {
    fn from(value: Triangle) -> Box<dyn Shape + 'a> {
        Box::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{equal, pt, v};

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
        if let Normal::Default(normal) = t.normal {
            assert_eq!(normal, v(0.0, 0.0, -1.0));
        } else {
            panic!("wrong normal type");
        }
    }

    #[test]
    fn triangle_normal_at() {
        // finding the normal on a triangle
        let t = Triangle::new(pt(0.0, 1.0, 0.0), pt(-1.0, 0.0, 0.0), pt(1.0, 0.0, 0.0));
        let i = Intersection::new(0.0, &t);
        let n1 = t.local_normal_at(pt(0.0, 0.5, 0.0), &i);
        let n2 = t.local_normal_at(pt(-0.5, 0.75, 0.0), &i);
        let n3 = t.local_normal_at(pt(0.5, 0.25, 0.0), &i);

        if let Normal::Default(normal) = t.normal {
            assert_eq!(n1, normal);
            assert_eq!(n2, normal);
            assert_eq!(n3, normal);
        } else {
            panic!("wrong normal type");
        }
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

    #[test]
    fn smooth_triangles() {
        let p1 = pt(0.0, 1.0, 0.0);
        let p2 = pt(-1.0, 0.0, 0.0);
        let p3 = pt(1.0, 0.0, 0.0);
        let n1 = v(0.0, 1.0, 0.0);
        let n2 = v(-1.0, 0.0, 0.0);
        let n3 = v(1.0, 0.0, 0.0);
        let tri = Triangle::smooth(p1, p2, p3, n1, n2, n3);

        // constructing a smooth triangle
        assert_eq!(tri.p1, p1);
        assert_eq!(tri.p2, p2);
        assert_eq!(tri.p3, p3);
        assert_eq!(tri.normal, Normal::Smooth(n1, n2, n3));

        // an intersection can encapsulate u and v
        let s = Triangle::new(pt(0.0, 1.0, 0.0), pt(-1.0, 0.0, 0.0), pt(1.0, 0.0, 0.0));
        let i = Intersection::with_uv(3.5, &s, 0.2, 0.4);
        let (iu, iv) = i.uv.unwrap();
        assert_eq!(iu, 0.2);
        assert_eq!(iv, 0.4);

        // an intersection with a smooth triangle stores u, v
        let r = Ray::new(pt(-0.2, 0.3, -2.0), v(0.0, 0.0, 1.0));
        let xs = tri.local_intersect(&r);
        assert_eq!(xs.len(), 1);
        let (iu, iv) = xs[0].uv.unwrap();
        assert!(equal(iu, 0.45));
        assert!(equal(iv, 0.25));

        // a smooth triangle uses u, v to interpolate the normal
        let i = Intersection::with_uv(1.0, &tri, 0.45, 0.25);
        let n = tri.normal_at(pt(0.0, 0.0, 0.0), &i);
        assert_eq!(n, v(-0.5547, 0.83205, 0.0));

        // preparing the normal on a smooth triangle
        let i = Intersection::with_uv(1.0, &tri, 0.45, 0.25);
        let xs = vec![i.clone()];
        let comps = i.prepare_computations(&r, &xs);
        assert_eq!(comps.normalv, v(-0.5547, 0.83205, 0.0));
    }
}
