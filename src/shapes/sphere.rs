use crate::{pt, Intersection, Material, Matrix4x4, Props, Ray, Shape, Tuple};
use std::any::Any;

#[derive(PartialEq, Debug)]
pub struct Sphere {
    props: Props,
}

impl Sphere {
    pub fn new() -> Sphere {
        Sphere {
            props: Props::default(),
        }
    }

    pub fn glass() -> Sphere {
        let mut m = Material::new();
        m.transparency = 1.0;
        m.refractive_index = 1.5;
        Sphere::new().material(m)
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

impl Shape for Sphere {
    fn local_intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>> {
        let sphere_to_ray = ray.origin - pt(0.0, 0.0, 0.0);

        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * ray.direction.dot(&sphere_to_ray);
        let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;

        let discriminant = (b * b) - 4.0 * a * c;

        if discriminant < 0.0 {
            return Vec::new();
        }

        let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
        vec![Intersection::new(t1, self), Intersection::new(t2, self)]
    }

    fn local_normal_at(&self, local_point: Tuple) -> Tuple {
        local_point - pt(0.0, 0.0, 0.0)
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

impl<'a> From<Sphere> for Box<dyn Shape + 'a> {
    fn from(value: Sphere) -> Box<dyn Shape + 'a> {
        Box::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v;

    #[test]
    fn sphere_glass() {
        // a helper for producing a sphere with a glassy material
        let s = Sphere::glass().shape();
        assert_eq!(s.transform(), &Matrix4x4::identity());
        assert_eq!(s.material().transparency, 1.0);
        assert_eq!(s.material().refractive_index, 1.5);
    }

    #[test]
    fn ray_sphere_intersection() {
        // a ray intersects a sphere at two points
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let s = Sphere::new();
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 4.0);
        assert_eq!(xs[0].object, &Sphere::new() as &dyn Shape);
        assert_eq!(xs[1].t, 6.0);
        assert_eq!(xs[1].object, &Sphere::new() as &dyn Shape);

        // a ray intersects a sphere at a tangent
        let r = Ray::new(pt(0.0, 1.0, -5.0), v(0.0, 0.0, 1.0));
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 5.0);
        assert_eq!(xs[1].t, 5.0);

        // a ray misses a sphere
        let r = Ray::new(pt(0.0, 2.0, -5.0), v(0.0, 0.0, 1.0));
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 0);

        // a ray originates inside a sphere
        let r = Ray::new(pt(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0));
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, -1.0);
        assert_eq!(xs[1].t, 1.0);

        // a sphere is behind a ray
        let r = Ray::new(pt(0.0, 0.0, 5.0), v(0.0, 0.0, 1.0));
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, -6.0);
        assert_eq!(xs[1].t, -4.0);
    }

    #[test]
    fn ray_intersection_with_scaled_sphere() {
        // intersecting a scaled sphere with a ray
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let s = Sphere::new().transform(Matrix4x4::scaling(2.0, 2.0, 2.0));
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 3.0);
        assert_eq!(xs[1].t, 7.0);

        // intersecting a translated sphere with a ray
        let s = Sphere::new().transform(Matrix4x4::translation(5.0, 0.0, 0.0));
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn sphere_normal_at() {
        // the normal on a sphere at a point on the x axis
        let s = Sphere::new();
        let n = s.normal_at(pt(1.0, 0.0, 0.0));
        assert_eq!(n, v(1.0, 0.0, 0.0));

        // the normal on a sphere at a point on the y axis
        let n = s.normal_at(pt(0.0, 1.0, 0.0));
        assert_eq!(n, v(0.0, 1.0, 0.0));

        // the normal on a sphere at a point on the y axis
        let n = s.normal_at(pt(0.0, 0.0, 1.0));
        assert_eq!(n, v(0.0, 0.0, 1.0));

        // the normal on a sphere at a point on a nonaxial point
        let n = s.normal_at(pt(
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
        ));
        assert_eq!(
            n,
            v(
                3.0_f64.sqrt() / 3.0,
                3.0_f64.sqrt() / 3.0,
                3.0_f64.sqrt() / 3.0
            )
        );

        // the normal is a normalized vector
        assert_eq!(n, n.normalize());
    }
}
