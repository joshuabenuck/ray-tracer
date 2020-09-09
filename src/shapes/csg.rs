use crate::{Intersection, Props, Ray, Shape, Tuple};
use std::any::Any;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Op {
    Union,
    Intersection,
    Difference,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Hit {
    Left,
    Right,
}

pub struct Csg {
    props: Props,
    op: Op,
    left: Box<dyn Shape>,
    right: Box<dyn Shape>,
}

impl Csg {
    pub fn new(op: Op, left: Box<dyn Shape>, right: Box<dyn Shape>) -> Csg {
        Csg {
            props: Props::default(),
            op,
            left,
            right,
        }
    }

    pub fn union(left: Box<dyn Shape>, right: Box<dyn Shape>) -> Csg {
        Csg {
            props: Props::default(),
            op: Op::Union,
            left,
            right,
        }
    }

    fn intersection_allowed(op: Op, hit: Hit, inl: bool, inr: bool) -> bool {
        match op {
            Op::Union => (hit == Hit::Left && !inr) || (hit == Hit::Right && !inl),
            Op::Intersection => (hit == Hit::Left && inr) || (hit == Hit::Right && inl),
            Op::Difference => (hit == Hit::Left && !inr) || (hit == Hit::Right && inl),
        }
    }

    fn filter_intersections<'a>(&self, xs: Vec<Intersection<'a>>) -> Vec<Intersection<'a>> {
        // begin outside of both children
        let mut inl = false;
        let mut inr = false;

        // prepare a list to receive the filtered intersections
        let mut result = Vec::new();

        for i in xs {
            // if i.object is part of the left child then hit is Hit::Left
            let hit = if self.left.includes(i.object) {
                Hit::Left
            } else {
                Hit::Right
            };

            if Self::intersection_allowed(self.op, hit, inl, inr) {
                result.push(i.clone());
            }

            // depending on which object was hit, toggle inl or inr
            if hit == Hit::Left {
                inl = !inl;
            } else {
                inr = !inr;
            }
        }
        result
    }
}

impl Shape for Csg {
    fn local_intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>> {
        let mut left_xs = self.left.intersect(&ray);
        let mut right_xs = self.right.intersect(&ray);
        left_xs.append(&mut right_xs);
        let mut xs = left_xs;
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        self.filter_intersections(xs)
    }

    fn local_normal_at(&self, _local_point: Tuple, _i: &Intersection) -> Tuple {
        unreachable!()
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
        if let Some(other) = other.downcast_ref::<Csg>() {
            return &self.left == &other.left && &self.right == &other.right;
        }
        false
    }

    fn includes(&self, other: &dyn Shape) -> bool {
        self.left.includes(other) || self.right.includes(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{id, pt, v, Cube, Intersection, Ray, Sphere};
    use std::any::TypeId;

    #[test]
    fn csg_create() {
        let s1 = Sphere::new();
        let s2 = Cube::new();
        let c = Csg::union(s1.shape(), s2.shape());
        assert_eq!(c.left.as_any().type_id(), TypeId::of::<Sphere>());
        assert_eq!(c.right.as_any().type_id(), TypeId::of::<Cube>());
    }

    #[test]
    fn csg_ops() {
        let mut scenarios = vec![
            // op       hit        inl   inr   result
            (Op::Union, Hit::Left, true, true, false),
            (Op::Union, Hit::Left, true, false, true),
            (Op::Union, Hit::Left, false, true, false),
            (Op::Union, Hit::Left, false, false, true),
            (Op::Union, Hit::Right, true, true, false),
            (Op::Union, Hit::Right, true, false, false),
            (Op::Union, Hit::Right, false, true, true),
            (Op::Union, Hit::Right, false, false, true),
            (Op::Intersection, Hit::Left, true, true, true),
            (Op::Intersection, Hit::Left, true, false, false),
            (Op::Intersection, Hit::Left, false, true, true),
            (Op::Intersection, Hit::Left, false, false, false),
            (Op::Intersection, Hit::Right, true, true, true),
            (Op::Intersection, Hit::Right, true, false, true),
            (Op::Intersection, Hit::Right, false, true, false),
            (Op::Intersection, Hit::Right, false, false, false),
            (Op::Difference, Hit::Left, true, true, false),
            (Op::Difference, Hit::Left, true, false, true),
            (Op::Difference, Hit::Left, false, true, false),
            (Op::Difference, Hit::Left, false, false, true),
            (Op::Difference, Hit::Right, true, true, true),
            (Op::Difference, Hit::Right, true, false, true),
            (Op::Difference, Hit::Right, false, true, false),
            (Op::Difference, Hit::Right, false, false, false),
        ];
        for (op, hit, inl, inr, result) in scenarios.drain(..) {
            assert_eq!(Csg::intersection_allowed(op, hit, inl, inr), result);
        }
    }

    #[test]
    fn intersection_filter() {
        // filtering a list of intersections
        let mut scenarios = vec![
            (Op::Union, 0, 3),
            (Op::Intersection, 1, 2),
            (Op::Difference, 0, 1),
        ];
        for (op, x0, x1) in scenarios.drain(..) {
            let s1 = Sphere::new();
            let s2 = Cube::new();
            let csg = Csg::new(op, s1.shape(), s2.shape());
            let xs = vec![
                Intersection::new(1.0, &*csg.left),
                Intersection::new(2.0, &*csg.right),
                Intersection::new(3.0, &*csg.left),
                Intersection::new(4.0, &*csg.right),
            ];
            let result = csg.filter_intersections(xs.clone());
            assert_eq!(result.len(), 2);
            assert_eq!(result[0], xs[x0]);
            assert_eq!(result[1], xs[x1]);
        }
    }

    #[test]
    fn ray_misses_csg() {
        // a ray misses a CSG object
        let c = Csg::union(Sphere::new().shape(), Cube::new().shape());
        let r = Ray::new(pt(0.0, 2.0, -5.0), v(0.0, 0.0, 1.0));
        let xs = c.local_intersect(&r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn ray_hits_csg() {
        // a ray hits a CSG object
        let s1 = Sphere::new();
        let s2 = Sphere::new().transform(id().translate(0.0, 0.0, 0.5));
        let c = Csg::union(s1.shape(), s2.shape());
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let xs = c.local_intersect(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 4.0);
        assert_eq!(xs[0].object, &*c.left);
        assert_eq!(xs[1].t, 6.5);
        assert_eq!(xs[1].object, &*c.right);
    }
}
