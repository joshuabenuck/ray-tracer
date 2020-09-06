use crate::{Intersection, Material, Matrix4x4, Ray, Tuple};
use std::any::Any;
use std::fmt::Debug;

#[derive(Clone)]
pub struct Props {
    pub transform: Matrix4x4,
    pub material: Material,
    // pub parent: Option<Box<dyn Shape>>,
}

impl Default for Props {
    fn default() -> Props {
        Props {
            transform: Matrix4x4::identity(),
            material: Material::new(),
            // parent: None,
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

// pub struct Group<'a> {
//     props: Props,
//     children: Vec<Box<dyn Shape + 'a>>,
// }

pub trait Shape {
    fn as_any(&self) -> &dyn Any;
    fn shape_eq(&self, other: &dyn Any) -> bool;
    fn intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>> {
        let ray = ray.transform(self.transform().inverse().unwrap());
        self.local_intersect(&ray)
    }
    fn local_intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>>;
    fn normal_at(&self, world_point: Tuple) -> Tuple {
        let object_point = self.world_to_object(world_point);
        let object_normal = self.local_normal_at(object_point);
        self.normal_to_world(object_normal)
    }
    fn world_to_object(&self, point: Tuple) -> Tuple {
        let point = point;
        let shape = self;
        // if let Some(parent) = &shape.parent {
        //     point = world_to_object(&parent, point);
        // }
        shape.transform().inverse().unwrap() * point
    }
    fn normal_to_world(&self, normal: Tuple) -> Tuple {
        let shape = self;
        let mut normal = shape.transform().inverse().unwrap().transpose() * normal;
        normal.w = 0.0;
        normal = normal.normalize();

        // if let Some(parent) = &shape.parent {
        //     normal = normal_to_world(&parent, normal);
        // }

        normal
    }
    fn local_normal_at(&self, local_point: Tuple) -> Tuple;
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

// impl PartialEq for Shape {
//     fn eq(&self, other: &Self) -> bool {
//         self.transform == other.transform
//             && self.material == other.material
//             && self.form == other.form
//     }
// }

// #[inline]
// pub fn group<'a>() -> Group<'a> {
//     Group {
//         props: Props::default(),
//         children: Vec::new(),
//     }
// }

// pub fn add_child<'a>(group: &'a mut Group, child: Box<dyn Shape + 'a>) {
//     // child.parent = Some(group.clone());
//     // group.children.push(child);
// }

// pub fn children<'a>(group: &'a Group) -> &'a Vec<Box<dyn Shape + 'a>> {
//     &group.children
// }

// impl<'a> Shape for Group<'a> {
//     fn local_intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>> {
//         let mut xs: Vec<Intersection> = self
//             .children
//             .iter()
//             .flat_map(|c| c.intersect(&ray))
//             .collect();
//         xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
//         xs
//     }

//     fn local_normal_at(&self, _local_point: Tuple) -> Tuple {
//         unreachable!()
//     }

//     fn common(&self) -> &Props {
//         &self.props
//     }

//     fn common_mut(&mut self) -> &mut Props {
//         &mut self.props
//     }
// }

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

        fn local_normal_at(&self, local_point: Tuple) -> Tuple {
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

        fn shape_eq(&self, other: &dyn Any) -> bool {
            match other.downcast_ref::<Self>() {
                Some(_) => true,
                None => false,
            }
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

    // #[test]
    // fn group_create() {
    //     // creating a new group
    //     let g = group();
    //     assert_eq!(g.transform(), &Matrix4x4::identity());
    //     assert_eq!(g.children.len(), 0);
    // }

    // #[test]
    // fn group_add_children() {
    //     // adding a child to a group
    //     let mut g = group();
    //     let s = test_shape();
    //     add_child(&mut g, Box::new(s));
    //     let children = children(&g);
    //     assert_eq!(children.len(), 1);
    //     // assert!(children.iter().any(|e| e == &s));
    //     // let s = s;
    //     // assert!(s.parent.is_some());
    //     // assert_eq!(s.parent.as_ref().unwrap(), &g);
    // }

    #[test]
    fn shape_parent() {
        // a shape has a parent attribute
        let s = TestShape::new();
        // assert!(s.parent == None);
    }

    #[test]
    fn shape_transformed_normal() {
        // computing the normal on a translated shape
        let mut s = TestShape::new();
        s.set_transform(Matrix4x4::translation(0.0, 1.0, 0.0));
        let n = s.normal_at(pt(0.0, 1.70711, -0.70711));
        assert_eq!(n, v(0.0, 0.70711, -0.70711));

        // computing the normal on a transformed shape
        let mut s = TestShape::new();
        s.set_transform(Matrix4x4::scaling(1.0, 0.5, 1.0) * Matrix4x4::rotation_z(PI / 5.0));
        let n = s.normal_at(pt(0.0, 2.0_f64 / 2.0, -2.0_f64 / 2.0));
        assert_eq!(n, v(0.0, 0.97014, -0.24254));
    }

    // #[test]
    // fn ray_intersection_with_empty_group() {
    //     // intersecting a ray with an empty group
    //     let g = group();
    //     let r = Ray::new(pt(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0));
    //     let xs = g.local_intersect(&r);
    //     assert_eq!(xs.len(), 0);
    // }

    #[test]
    fn ray_intersection_with_non_empty_group() {
        // intersecting a ray with a non-empty group
        // let g = group();
        // let s1: Box<dyn Shape> = Sphere::new().into();
        // let s2: Box<dyn Shape> = Sphere::new().transform(Matrix4x4::translation(0.0, 0.0, -3.0)).into();
        // let s3: Box<dyn Shape> = Sphere::new().transform(Matrix4x4::translation(5.0, 0.0, 0.0)).into();
        // add_child(&mut g, s1.clone());
        // add_child(&mut g, s2.clone());
        // add_child(&mut g, s3.clone());
        // let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        // let xs = g.local_intersect(&r);
        // assert_eq!(children(&g).len(), 3);
        // assert_eq!(xs.len(), 4);
        // assert_eq!(xs[0].object, &*s2);
        // assert_eq!(xs[1].object, &*s2);
        // assert_eq!(xs[2].object, &*s1);
        // assert_eq!(xs[3].object, &*s1);
    }

    // #[test]
    // fn ray_intersection_with_transformed_group() {
    //     // intersecting a transformed group
    //     let mut g = group();
    //     g.set_transform(Matrix4x4::scaling(2.0, 2.0, 2.0));
    //     // let s = Sphere::new().transform(Matrix4x4::translation(5.0, 0.0, 0.0));
    //     // add_child(&g, s.clone());
    //     let r = Ray::new(pt(10.0, 0.0, -10.0), v(0.0, 0.0, 1.0));
    //     let xs = g.intersect(&r);
    //     assert_eq!(xs.len(), 2);
    // }

    // #[test]
    // fn world_to_object_space() {
    //     // converting a point from world to object space
    //     let mut g1 = group();
    //     g1.set_transform(Matrix4x4::rotation_y(PI / 2.0));
    //     let mut g2 = group();
    //     g2.set_transform(Matrix4x4::scaling(2.0, 2.0, 2.0));
    //     let s = Sphere::new().transform(Matrix4x4::translation(5.0, 0.0, 0.0));
    //     // add_child(&g2, s.clone());
    //     // add_child(&g1, g2);
    //     let p = world_to_object(&s, pt(-2.0, 0.0, -10.0));
    //     assert_eq!(p, pt(0.0, 0.0, -1.0));
    // }

    // #[test]
    // fn normal_from_object_to_world_space() {
    //     // converting a normal from object to world space
    //     let mut g1 = group();
    //     g1.set_transform(Matrix4x4::rotation_y(PI / 2.0));
    //     let mut g2 = group();
    //     g2.set_transform(Matrix4x4::scaling(1.0, 2.0, 3.0));
    //     let s = Sphere::new().transform(Matrix4x4::translation(5.0, 0.0, 0.0));
    //     // add_child(&g2, s.clone());
    //     // add_child(&g1, g2);
    //     let n = normal_to_world(
    //         &s,
    //         v(
    //             3.0_f64.sqrt() / 3.0,
    //             3.0_f64.sqrt() / 3.0,
    //             3.0_f64.sqrt() / 3.0,
    //         ),
    //     );
    //     assert_eq!(n, v(0.28571, 0.42857, -0.85714));
    // }

    // #[test]
    // fn normal_on_child_object() {
    //     // finding the normal on a child object
    //     let mut g1 = group();
    //     g1.set_transform(Matrix4x4::rotation_y(PI / 2.0));
    //     let mut g2 = group();
    //     g2.set_transform(Matrix4x4::scaling(1.0, 2.0, 3.0));
    //     let s = Sphere::new().transform(Matrix4x4::translation(5.0, 0.0, 0.0));
    //     // add_child(&g2, s.clone());
    //     // add_child(&g1, g2);
    //     let n = normal_at(&s, pt(1.7321, 1.1547, -5.5774));
    //     assert_eq!(n, v(0.2857, 0.42854, -0.85716));
    // }
}
