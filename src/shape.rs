use crate::{equal, v, Intersection, Material, Matrix4x4, Ray, Tuple, EPSILON};
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

#[derive(PartialEq, Debug)]
pub struct Cube {
    props: Props,
}
#[derive(PartialEq, Debug)]
pub struct Cylinder {
    props: Props,
    min: f64,
    max: f64,
    closed: bool,
}

// pub struct Group<'a> {
//     props: Props,
//     children: Vec<Box<dyn Shape + 'a>>,
// }

#[derive(PartialEq)]
pub struct Triangle {
    props: Props,
    p1: Tuple,
    p2: Tuple,
    p3: Tuple,
}

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

#[derive(PartialEq)]
pub struct TestShape {
    props: Props,
}

impl TestShape {
    fn as_shape(&self) -> &dyn Shape {
        self
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

static mut SAVED_RAY: Option<Ray> = None;

pub fn test_shape() -> TestShape {
    TestShape {
        props: Props::default(),
    }
}

#[inline]
pub fn cube() -> Cube {
    cubetm(Matrix4x4::identity(), Material::new())
}

#[inline]
pub fn cubet(transform: Matrix4x4) -> Cube {
    cubetm(transform, Material::new())
}

#[inline]
pub fn cubem(material: Material) -> Cube {
    cubetm(Matrix4x4::identity(), material)
}

#[inline]
pub fn cubetm(transform: Matrix4x4, material: Material) -> Cube {
    Cube {
        props: Props {
            transform,
            material,
            ..Props::default()
        },
    }
}

#[inline]
pub fn cylinder(min: f64, max: f64, closed: bool) -> Cylinder {
    cylindertm(Matrix4x4::identity(), Material::new(), min, max, closed)
}

#[inline]
pub fn cylindert(transform: Matrix4x4, min: f64, max: f64, closed: bool) -> Cylinder {
    cylindertm(transform, Material::new(), min, max, closed)
}

#[inline]
pub fn cylinderm(material: Material, min: f64, max: f64, closed: bool) -> Cylinder {
    cylindertm(Matrix4x4::identity(), material, min, max, closed)
}

#[inline]
pub fn cylindertm(
    transform: Matrix4x4,
    material: Material,
    min: f64,
    max: f64,
    closed: bool,
) -> Cylinder {
    Cylinder {
        props: Props {
            transform,
            material,
            ..Props::default()
        },
        min,
        max,
        closed,
    }
}

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

#[inline]
pub fn triangle(p1: Tuple, p2: Tuple, p3: Tuple) -> Triangle {
    Triangle {
        props: Props::default(),
        p1,
        p2,
        p3,
    }
}

fn check_axis(origin: f64, direction: f64) -> (f64, f64) {
    let tmin_numerator = -1.0 - origin;
    let tmax_numerator = 1.0 - origin;

    let (mut tmin, mut tmax) = if direction.abs() >= EPSILON {
        (tmin_numerator / direction, tmax_numerator / direction)
    } else {
        (
            tmin_numerator * std::f64::INFINITY,
            tmax_numerator * std::f64::INFINITY,
        )
    };

    if tmin > tmax {
        std::mem::swap(&mut tmin, &mut tmax);
    }

    (tmin, tmax)
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

impl Shape for Cube {
    fn local_intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>> {
        // How to avoid always doing all of these computations?
        let (xtmin, xtmax) = check_axis(ray.origin.x, ray.direction.x);
        let (ytmin, ytmax) = check_axis(ray.origin.y, ray.direction.y);
        let (ztmin, ztmax) = check_axis(ray.origin.z, ray.direction.z);

        let tmin = xtmin.max(ytmin).max(ztmin);
        let tmax = xtmax.min(ytmax).min(ztmax);

        if tmin > tmax {
            return Vec::new();
        }

        vec![Intersection::new(tmin, self), Intersection::new(tmax, self)]
    }

    fn local_normal_at(&self, local_point: Tuple) -> Tuple {
        let maxc = local_point
            .x
            .abs()
            .max(local_point.y.abs())
            .max(local_point.z.abs());

        if maxc == local_point.x.abs() {
            v(local_point.x, 0.0, 0.0)
        } else if maxc == local_point.y.abs() {
            v(0.0, local_point.y, 0.0)
        } else {
            v(0.0, 0.0, local_point.z)
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

    fn shape_eq(&self, other: &dyn Any) -> bool {
        match other.downcast_ref::<Self>() {
            Some(_) => true,
            None => false,
        }
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

    fn local_normal_at(&self, local_point: Tuple) -> Tuple {
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

    fn shape_eq(&self, other: &dyn Any) -> bool {
        match other.downcast_ref::<Self>() {
            Some(_) => true,
            None => false,
        }
    }
}

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

impl<'a> From<Cube> for Box<dyn Shape + 'a> {
    fn from(value: Cube) -> Box<dyn Shape + 'a> {
        Box::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pt;
    use std::f64::consts::PI;

    #[test]
    fn shape_operations() {
        // the default transformation
        let mut s = test_shape();
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
        let mut s = test_shape();
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
        let s = test_shape();
        // assert!(s.parent == None);
    }

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

    #[test]
    fn shape_transformed_normal() {
        // computing the normal on a translated shape
        let mut s = test_shape();
        s.set_transform(Matrix4x4::translation(0.0, 1.0, 0.0));
        let n = s.normal_at(pt(0.0, 1.70711, -0.70711));
        assert_eq!(n, v(0.0, 0.70711, -0.70711));

        // computing the normal on a transformed shape
        let mut s = test_shape();
        s.set_transform(Matrix4x4::scaling(1.0, 0.5, 1.0) * Matrix4x4::rotation_z(PI / 5.0));
        let n = s.normal_at(pt(0.0, 2.0_f64 / 2.0, -2.0_f64 / 2.0));
        assert_eq!(n, v(0.0, 0.97014, -0.24254));
    }

    #[test]
    fn ray_intersection_with_cube() {
        // a ray intersects a cube
        fn test(origin: Tuple, direction: Tuple, t1: f64, t2: f64) {
            let c = cube();
            let r = Ray::new(origin, direction);
            let xs = c.local_intersect(&r);
            assert_eq!(xs.len(), 2);
            assert_eq!(xs[0].t, t1);
            assert_eq!(xs[1].t, t2);
        }
        // +x
        test(pt(5.0, 0.5, 0.0), v(-1.0, 0.0, 0.0), 4.0, 6.0);
        // -x
        test(pt(-5.0, 0.5, 0.0), v(1.0, 0.0, 0.0), 4.0, 6.0);
        // +y
        test(pt(0.5, 5.0, 0.0), v(0.0, -1.0, 0.0), 4.0, 6.0);
        // -y
        test(pt(0.5, -5.0, 0.0), v(0.0, 1.0, 0.0), 4.0, 6.0);
        // +z
        test(pt(0.5, 0.0, 5.0), v(0.0, 0.0, -1.0), 4.0, 6.0);
        // -z
        test(pt(0.5, 0.0, -5.0), v(0.0, 0.0, 1.0), 4.0, 6.0);
        // insidee
        test(pt(0.0, 0.5, 0.0), v(0.0, 0.0, 1.0), -1.0, 1.0);

        fn test_miss(origin: Tuple, direction: Tuple) {
            let c = cube();
            let r = Ray::new(origin, direction);
            let xs = c.local_intersect(&r);
            assert_eq!(xs.len(), 0);
        }
        test_miss(pt(-2.0, 0.0, 0.0), v(0.2673, 0.5345, 0.8018));
        test_miss(pt(0.0, -2.0, 0.0), v(0.8018, 0.2673, 0.5345));
        test_miss(pt(0.0, 0.0, -2.0), v(0.5345, 0.8018, 0.2673));
        test_miss(pt(2.0, 0.0, 2.0), v(0.0, 0.0, -1.0));
        test_miss(pt(0.0, 2.0, 2.0), v(0.0, -1.0, 0.0));
        test_miss(pt(2.0, 0.0, 2.0), v(-1.0, 0.0, 0.0));
    }

    #[test]
    fn ray_intersection_with_cylinder_misses() {
        // a ray misses a cylinder
        fn test(scenario: &str, origin: Tuple, direction: Tuple) {
            let cyl = cylinder(std::f64::NEG_INFINITY, std::f64::INFINITY, false);
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
            let cyl = cylinder(std::f64::NEG_INFINITY, std::f64::INFINITY, false);
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
            let cyl = cylinder(1.0, 2.0, false);
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
            let cyl = cylinder(1.0, 2.0, true);
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

    #[test]
    fn cube_normal_at() {
        // the normal of the surface of a cube
        fn test(point: Tuple, normal: Tuple) {
            let c = cube();
            let n = c.normal_at(point);
            assert_eq!(n, normal);
        }

        test(pt(1.0, 0.5, -0.8), v(1.0, 0.0, 0.0));
        test(pt(-1.0, -0.2, 0.9), v(-1.0, 0.0, 0.0));
        test(pt(-0.4, 1.0, -0.1), v(0.0, 1.0, 0.0));
        test(pt(0.3, -1.0, -0.7), v(0.0, -1.0, 0.0));
        test(pt(0.6, 0.3, 1.0), v(0.0, 0.0, 1.0));
        test(pt(0.4, 0.4, -1.0), v(0.0, 0.0, -1.0));
        // normal at cube's corners
        test(pt(1.0, 1.0, 1.0), v(1.0, 0.0, 0.0));
        test(pt(-1.0, -1.0, -1.0), v(-1.0, 0.0, 0.0));
    }

    #[test]
    fn cylinder_normal_at() {
        // normal vector on a cylinder
        fn test(point: Tuple, normal: Tuple) {
            let cyl = cylinder(std::f64::NEG_INFINITY, std::f64::INFINITY, false);
            let n = cyl.local_normal_at(point);
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
            let cyl = cylinder(1.0, 2.0, true);
            let n = cyl.local_normal_at(point);
            assert_eq!(n, normal);
        }

        test(pt(0.0, 1.0, 0.0), v(0.0, -1.0, 0.0));
        test(pt(0.5, 1.0, 0.0), v(0.0, -1.0, 0.0));
        test(pt(0.0, 1.0, 0.5), v(0.0, -1.0, 0.0));
        test(pt(0.0, 2.0, 0.0), v(0.0, 1.0, 0.0));
        test(pt(0.5, 2.0, 0.0), v(0.0, 1.0, 0.0));
        test(pt(0.0, 2.0, 0.5), v(0.0, 1.0, 0.0));
    }

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
