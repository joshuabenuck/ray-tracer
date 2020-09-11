use crate::{Intersection, Material, Matrix4x4, Ray, Tuple};
use std::any::Any;
use std::fmt::Debug;

pub struct Props {
    pub transform: Matrix4x4,
    pub material: Material,
    pub parent_transforms: Vec<Matrix4x4>,
    pub shadow: bool,
}

impl Default for Props {
    fn default() -> Props {
        Props {
            transform: Matrix4x4::identity(),
            material: Material::new(),
            parent_transforms: Vec::new(),
            shadow: true,
        }
    }
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        self.transform == other.transform && self.material == other.material
    }
}

impl Debug for Props {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Props {{ }}")
    }
}

pub trait Shape {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn shape_eq(&self, other: &dyn Any) -> bool;
    fn intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>> {
        let ray = ray.transform(self.transform().inverse().unwrap());
        self.local_intersect(&ray)
    }
    fn local_intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>>;
    fn normal_at(&self, world_point: Tuple, i: &Intersection) -> Tuple {
        let object_point = self.world_to_object(world_point);
        let object_normal = self.local_normal_at(object_point, i);
        self.normal_to_world(object_normal)
    }
    fn world_to_object(&self, point: Tuple) -> Tuple {
        let mut point = point;
        for transform in self.parent_transforms() {
            point = transform.inverse().unwrap() * point;
        }
        self.transform().inverse().unwrap() * point
    }
    fn normal_to_world(&self, normal: Tuple) -> Tuple {
        let shape = self;
        fn compute_normal(transform: &Matrix4x4, normal: Tuple) -> Tuple {
            let mut normal = transform.inverse().unwrap().transpose() * normal;
            normal.w = 0.0;
            normal = normal.normalize();
            normal
        }
        let mut normal = compute_normal(shape.transform(), normal);
        for transform in self.parent_transforms().iter().rev() {
            normal = compute_normal(transform, normal);
        }

        normal
    }
    fn local_normal_at(&self, local_point: Tuple, i: &Intersection) -> Tuple;
    fn common(&self) -> &Props;
    fn common_mut(&mut self) -> &mut Props;
    fn transform(&self) -> &Matrix4x4 {
        &self.common().transform
    }
    fn transform_mut(&mut self) -> &mut Matrix4x4 {
        &mut self.common_mut().transform
    }
    fn set_transform(&mut self, transform: Matrix4x4) {
        self.common_mut().transform = transform;
    }
    fn material(&self) -> &Material {
        &self.common().material
    }
    fn material_mut(&mut self) -> &mut Material {
        &mut self.common_mut().material
    }
    fn set_material(&mut self, material: Material) {
        self.common_mut().material = material;
    }
    fn parent_transforms(&self) -> &Vec<Matrix4x4> {
        &self.common().parent_transforms
    }
    fn set_parent_transforms(&mut self, parent_transforms: Vec<Matrix4x4>) {
        self.common_mut().parent_transforms = parent_transforms;
    }
    fn refresh_parents(&mut self) {}
    fn includes(&self, other: &dyn Shape) -> bool;
    fn shadow(&self) -> bool {
        self.common().shadow
    }
    fn set_shadow(&mut self, shadow: bool) {
        self.common_mut().shadow = shadow;
    }
}

impl Debug for dyn Shape + '_ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Shape {{ {:?} {:?} }}",
            self.transform(),
            self.material()
        )
    }
}

impl PartialEq for dyn Shape + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.common() == other.common() && self.shape_eq(other.as_any())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{pt, v};
    use std::f64::consts::PI;

    static mut SAVED_RAY: Option<Ray> = None;

    #[derive(PartialEq)]
    pub struct TestShape {
        props: Props,
    }

    impl TestShape {
        fn new() -> TestShape {
            TestShape {
                props: Props::default(),
            }
        }
    }

    impl Shape for TestShape {
        fn local_intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>> {
            unsafe {
                SAVED_RAY = Some(*ray);
            }
            Vec::new()
        }

        fn local_normal_at(&self, local_point: Tuple, _i: &Intersection) -> Tuple {
            v(local_point.x, local_point.y, local_point.z)
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

    #[test]
    fn shape_operations() {
        // the default transformation
        let mut s = TestShape::new();
        assert_eq!(s.transform(), &Matrix4x4::identity());

        // assigning a transformation
        let t = Matrix4x4::translation(2.0, 3.0, 4.0);
        s.set_transform(t);
        assert_eq!(s.transform(), &t);

        // the default material
        assert_eq!(s.material(), &Material::new());

        // assigning a material
        let mut m = Material::new();
        m.ambient = 1.0;
        s.set_material(m);
        assert_eq!(s.material(), &m);

        // intersecting a scaled shape with a ray
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let mut s = TestShape::new();
        s.set_transform(Matrix4x4::scaling(2.0, 2.0, 2.0));
        unsafe {
            SAVED_RAY = None;
            s.intersect(&r);
            assert_eq!(SAVED_RAY.unwrap().origin, pt(0.0, 0.0, -2.5));
            assert_eq!(SAVED_RAY.unwrap().direction, v(0.0, 0.0, 0.5));
        };

        // intersecting a translated shape with a ray
        s.set_transform(Matrix4x4::translation(5.0, 0.0, 0.0));
        unsafe {
            SAVED_RAY = None;
            s.intersect(&r);
            assert_eq!(SAVED_RAY.unwrap().origin, pt(-5.0, 0.0, -5.0));
            assert_eq!(SAVED_RAY.unwrap().direction, v(0.0, 0.0, 1.0));
        };
    }

    #[test]
    fn shape_parent() {
        // a shape has a parent attribute
        let s = TestShape::new();
        assert!(s.parent_transforms().len() == 0);
    }

    #[test]
    fn shape_transformed_normal() {
        // computing the normal on a translated shape
        let mut s = TestShape::new();
        s.set_transform(Matrix4x4::translation(0.0, 1.0, 0.0));
        let n = s.normal_at(pt(0.0, 1.70711, -0.70711), &Intersection::new(0.0, &s));
        assert_eq!(n, v(0.0, 0.70711, -0.70711));

        // computing the normal on a transformed shape
        let mut s = TestShape::new();
        s.set_transform(Matrix4x4::scaling(1.0, 0.5, 1.0) * Matrix4x4::rotation_z(PI / 5.0));
        let n = s.normal_at(
            pt(0.0, 2.0_f64 / 2.0, -2.0_f64 / 2.0),
            &Intersection::new(0.0, &s),
        );
        assert_eq!(n, v(0.0, 0.97014, -0.24254));
    }
}
