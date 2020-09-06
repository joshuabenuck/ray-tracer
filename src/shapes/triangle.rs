use crate::{Intersection, Props, Ray, Shape, Tuple};
use std::any::Any;

#[derive(PartialEq)]
pub struct Triangle {
    props: Props,
    p1: Tuple,
    p2: Tuple,
    p3: Tuple,
}

impl Triangle {
    pub fn new(p1: Tuple, p2: Tuple, p3: Tuple) -> Triangle {
        Triangle {
            props: Props::default(),
            p1,
            p2,
            p3,
        }
    }
}

impl Shape for Triangle {
    fn local_intersect(&'_ self, _ray: &Ray) -> Vec<Intersection<'_>> {
        Vec::new()
    }

    fn local_normal_at(&self, _local_point: Tuple) -> Tuple {
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
    // #[test]
    // fn triangle_normal_vector() {
    //     // finding the normal on a triangle
    //     let p1 = pt(0.0, 1.0, 0.0);
    //     let p2 = pt(-1.0, 0.0, 0.0);
    //     let p3 = pt(1.0, 0.0, 0.0);
    //     let t = triangle(p1, p2, p3);
    //     let n1 = t.local_normal_at(pt(0.0, 0.5, 0.0));
    //     let n2 = t.local_normal_at(pt(-0.5, 0.75, 0.0));
    //     let n3 = t.local_normal_at(pt(0.5, 0.25, 0.0));
    // }
}
