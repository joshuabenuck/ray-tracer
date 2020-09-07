use crate::{v, Intersection, Material, Matrix4x4, Props, Ray, Shape, Tuple, EPSILON};
use std::any::Any;

#[derive(PartialEq, Debug)]
pub struct Plane {
    props: Props,
}

impl Plane {
    pub fn new() -> Plane {
        Plane {
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

impl Shape for Plane {
    fn local_intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>> {
        if ray.direction.y.abs() < EPSILON {
            Vec::new()
        } else {
            let t = -ray.origin.y / ray.direction.y;
            vec![Intersection::new(t, self)]
        }
    }

    fn local_normal_at(&self, _local_point: Tuple) -> Tuple {
        v(0.0, 1.0, 0.0)
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

impl<'a> From<Plane> for Box<dyn Shape + 'a> {
    fn from(value: Plane) -> Box<dyn Shape + 'a> {
        Box::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pt;

    #[test]
    fn ray_plane_intersection() {
        // intersect with a ray parallel to the plane
        let p = Plane::new();
        let r = Ray::new(pt(0.0, 10.0, 0.0), v(0.0, 0.0, 1.0));
        let xs = p.local_intersect(&r);
        assert_eq!(xs.len(), 0);

        // intersect with a coplanar ray
        let r = Ray::new(pt(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0));
        let xs = p.local_intersect(&r);
        assert_eq!(xs.len(), 0);

        // a ray intersecting a plane from above
        let r = Ray::new(pt(0.0, 1.0, 0.0), v(0.0, -1.0, 0.0));
        let xs = p.local_intersect(&r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 1.0);
        assert_eq!(xs[0].object, &p as &dyn Shape);

        // a ray intersection a plane from below
        let r = Ray::new(pt(0.0, -1.0, 0.0), v(0.0, 1.0, 0.0));
        let xs = p.local_intersect(&r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 1.0);
        assert_eq!(xs[0].object, &p as &dyn Shape);
    }

    #[test]
    fn plane_normal_at() {
        // the normal of a plane is constant everywhere
        let p = Plane::new();
        let n1 = p.normal_at(pt(0.0, 0.0, 0.0));
        let n2 = p.normal_at(pt(10.0, 0.0, -10.0));
        let n3 = p.normal_at(pt(-5.0, 0.0, 150.0));
        assert_eq!(n1, v(0.0, 1.0, 0.0));
        assert_eq!(n2, v(0.0, 1.0, 0.0));
        assert_eq!(n3, v(0.0, 1.0, 0.0));
    }
}
