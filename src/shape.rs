use crate::{equal, pt, v, Intersection, Material, Matrix4x4, Ray, Tuple, EPSILON};
use std::{cell::RefCell, fmt::Debug, rc::Rc};

#[derive(Clone, PartialEq)]
pub enum ShapeForm {
    Sphere,
    Plane,
    Cube,
    Cylinder(f64, f64, bool),
    Group(Vec<Rc<RefCell<Shape>>>),
    Test,
}

impl Debug for ShapeForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("shape form");
        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub struct Shape {
    pub transform: Matrix4x4,
    pub material: Material,
    pub form: ShapeForm,
    pub parent: Option<Rc<RefCell<Shape>>>,
}

impl Debug for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("shape");
        Ok(())
    }
}

static mut SAVED_RAY: Option<Ray> = None;

pub fn test_shape() -> Rc<RefCell<Shape>> {
    Rc::new(RefCell::new(Shape {
        transform: Matrix4x4::identity(),
        material: Material::new(),
        form: ShapeForm::Test,
        parent: None,
    }))
}

#[inline]
pub fn plane() -> Rc<RefCell<Shape>> {
    Rc::new(RefCell::new(Shape {
        transform: Matrix4x4::identity(),
        material: Material::new(),
        form: ShapeForm::Plane,
        parent: None,
    }))
}

#[inline]
pub fn planet(transform: Matrix4x4) -> Rc<RefCell<Shape>> {
    planetm(transform, Material::new())
}

#[inline]
pub fn planem(material: Material) -> Rc<RefCell<Shape>> {
    planetm(Matrix4x4::identity(), material)
}

#[inline]
pub fn planetm(transform: Matrix4x4, material: Material) -> Rc<RefCell<Shape>> {
    Rc::new(RefCell::new(Shape {
        material,
        transform,
        form: ShapeForm::Plane,
        parent: None,
    }))
}

#[inline]
pub fn sphere() -> Rc<RefCell<Shape>> {
    spheretm(Matrix4x4::identity(), Material::new())
}

pub fn spheretm(transform: Matrix4x4, material: Material) -> Rc<RefCell<Shape>> {
    Rc::new(RefCell::new(Shape {
        transform,
        material,
        form: ShapeForm::Sphere,
        parent: None,
    }))
}

#[inline]
pub fn spheret(transform: Matrix4x4) -> Rc<RefCell<Shape>> {
    spheretm(transform, Material::new())
}

#[inline]
pub fn spherem(material: Material) -> Rc<RefCell<Shape>> {
    spheretm(Matrix4x4::identity(), material)
}

#[inline]
pub fn glass_sphere() -> Rc<RefCell<Shape>> {
    let mut m = Material::new();
    m.transparency = 1.0;
    m.refractive_index = 1.5;
    spherem(m)
}

#[inline]
pub fn glass_spheret(transform: Matrix4x4) -> Rc<RefCell<Shape>> {
    let mut m = Material::new();
    m.transparency = 1.0;
    m.refractive_index = 1.5;
    spheretm(transform, m)
}

#[inline]
pub fn cube() -> Rc<RefCell<Shape>> {
    cubetm(Matrix4x4::identity(), Material::new())
}

#[inline]
pub fn cubet(transform: Matrix4x4) -> Rc<RefCell<Shape>> {
    cubetm(transform, Material::new())
}

#[inline]
pub fn cubem(material: Material) -> Rc<RefCell<Shape>> {
    cubetm(Matrix4x4::identity(), material)
}

#[inline]
pub fn cubetm(transform: Matrix4x4, material: Material) -> Rc<RefCell<Shape>> {
    Rc::new(RefCell::new(Shape {
        transform,
        material,
        form: ShapeForm::Cube,
        parent: None,
    }))
}

#[inline]
pub fn cylinder(min: f64, max: f64, closed: bool) -> Rc<RefCell<Shape>> {
    cylindertm(Matrix4x4::identity(), Material::new(), min, max, closed)
}

#[inline]
pub fn cylindert(transform: Matrix4x4, min: f64, max: f64, closed: bool) -> Rc<RefCell<Shape>> {
    cylindertm(transform, Material::new(), min, max, closed)
}

#[inline]
pub fn cylinderm(material: Material, min: f64, max: f64, closed: bool) -> Rc<RefCell<Shape>> {
    cylindertm(Matrix4x4::identity(), material, min, max, closed)
}

#[inline]
pub fn cylindertm(
    transform: Matrix4x4,
    material: Material,
    min: f64,
    max: f64,
    closed: bool,
) -> Rc<RefCell<Shape>> {
    Rc::new(RefCell::new(Shape {
        transform,
        material,
        form: ShapeForm::Cylinder(min, max, closed),
        parent: None,
    }))
}

#[inline]
pub fn group() -> Rc<RefCell<Shape>> {
    Rc::new(RefCell::new(Shape {
        transform: Matrix4x4::identity(),
        material: Material::new(),
        form: ShapeForm::Group(Vec::new()),
        parent: None,
    }))
}

pub fn add_child(group: &mut Rc<RefCell<Shape>>, child: &mut Rc<RefCell<Shape>>) {
    let mut c = child.borrow_mut();
    c.parent = Some(group.clone());
    if let ShapeForm::Group(children) = &mut group.borrow_mut().form {
        children.push(child.clone());
        return;
    }
    panic!("add_child may only be called on a group!");
}

pub fn children(group: &Rc<RefCell<Shape>>) -> Vec<Rc<RefCell<Shape>>> {
    if let ShapeForm::Group(children) = &group.borrow().form {
        return children.clone();
    }
    panic!("children may only be called on a group!");
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

fn intersect_caps(cyl: &Rc<RefCell<Shape>>, ray: &Ray, xs: &mut Vec<Intersection>) {
    match cyl.borrow().form {
        ShapeForm::Cylinder(max, min, closed) => {
            // caps only matter if the cylinder is closed and might
            // possibly be interesected by the ray.
            if !closed || equal(ray.direction.y, 0.0) {
                return;
            }

            // check for an intersection with the lower end cap by intersecting
            // the ray with the plane at y = min
            let t = (min - ray.origin.y) / ray.direction.y;
            if check_cap(ray, t) {
                xs.push(Intersection::new(t, cyl.clone()));
            }

            // check for an intersection with the upper end cap by intersecting
            // the ray with the plane at y = max
            let t = (max - ray.origin.y) / ray.direction.y;
            if check_cap(ray, t) {
                xs.push(Intersection::new(t, cyl.clone()));
            }
        }
        _ => unreachable!(),
    }
}

pub fn intersect(shape: &Rc<RefCell<Shape>>, ray: &Ray) -> Vec<Intersection> {
    let ray = ray.transform(shape.borrow().transform.inverse().unwrap());
    local_intersect(shape, &ray)
}

pub fn local_intersect(shape: &Rc<RefCell<Shape>>, ray: &Ray) -> Vec<Intersection> {
    match shape.borrow().form {
        ShapeForm::Test => {
            unsafe {
                SAVED_RAY = Some(*ray);
            }
            Vec::new()
        }
        ShapeForm::Sphere => {
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
            vec![
                Intersection::new(t1, shape.clone()),
                Intersection::new(t2, shape.clone()),
            ]
        }
        ShapeForm::Plane => {
            if ray.direction.y.abs() < EPSILON {
                Vec::new()
            } else {
                let t = -ray.origin.y / ray.direction.y;
                vec![Intersection::new(t, shape.clone())]
            }
        }
        ShapeForm::Cube => {
            // How to avoid always doing all of these computations?
            let (xtmin, xtmax) = check_axis(ray.origin.x, ray.direction.x);
            let (ytmin, ytmax) = check_axis(ray.origin.y, ray.direction.y);
            let (ztmin, ztmax) = check_axis(ray.origin.z, ray.direction.z);

            let tmin = xtmin.max(ytmin).max(ztmin);
            let tmax = xtmax.min(ytmax).min(ztmax);

            if tmin > tmax {
                return Vec::new();
            }

            vec![
                Intersection::new(tmin, shape.clone()),
                Intersection::new(tmax, shape.clone()),
            ]
        }
        ShapeForm::Cylinder(min, max, _closed) => {
            let a = ray.direction.x.powi(2) + ray.direction.z.powi(2);

            // ray is parallel to the y axis
            if equal(a, 0.0) {
                let mut xs = Vec::new();
                intersect_caps(shape, ray, &mut xs);
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
            if min < y0 && y0 < max {
                xs.push(Intersection::new(t0, shape.clone()));
            }

            let y1 = ray.origin.y + t1 * ray.direction.y;
            if min < y1 && y1 < max {
                xs.push(Intersection::new(t1, shape.clone()));
            }
            intersect_caps(shape, ray, &mut xs);
            xs
        }
        ShapeForm::Group(..) => Vec::new(),
    }
}

impl Shape {
    pub fn normal_at(&self, world_point: Tuple) -> Tuple {
        let object_point = self.transform.inverse().unwrap() * world_point;
        let object_normal = self.local_normal_at(object_point);
        // technically should be self.tranform.submatrix(3, 3)
        // to avoid messing with the w coordinate when there is any kind of translation
        // in the transform
        let mut world_normal = self.transform.inverse().unwrap().transpose() * object_normal;
        // workaround to avoid the submatrix calculation
        world_normal.w = 0.0;
        world_normal.normalize()
    }

    pub fn local_normal_at(&self, local_point: Tuple) -> Tuple {
        match self.form {
            ShapeForm::Test => v(local_point.x, local_point.y, local_point.z),
            ShapeForm::Sphere => local_point - pt(0.0, 0.0, 0.0),
            ShapeForm::Plane => v(0.0, 1.0, 0.0),
            ShapeForm::Cube => {
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
            ShapeForm::Cylinder(min, max, _closed) => {
                // compute the square of the distance from the y axis
                let dist = local_point.x.powi(2) + local_point.z.powi(2);

                if dist < 1.0 && local_point.y >= max - EPSILON {
                    return v(0.0, 1.0, 0.0);
                } else if dist < 1.0 && local_point.y <= min + EPSILON {
                    return v(0.0, -1.0, 0.0);
                }
                v(local_point.x, 0.0, local_point.z)
            }
            ShapeForm::Group(..) => v(0.0, 0.0, 0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn shape_operations() {
        // the default transformation
        let s = test_shape();
        assert_eq!(s.borrow_mut().transform, Matrix4x4::identity());

        // assigning a transformation
        let t = Matrix4x4::translation(2.0, 3.0, 4.0);
        s.borrow_mut().transform = t;
        assert_eq!(s.borrow_mut().transform, t);

        // the default material
        assert_eq!(s.borrow_mut().material, Material::new());

        // assigning a material
        let mut m = Material::new();
        m.ambient = 1.0;
        s.borrow_mut().material = m;
        assert_eq!(s.borrow_mut().material, m);

        // intersecting a scaled shape with a ray
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let s = test_shape();
        s.borrow_mut().transform = Matrix4x4::scaling(2.0, 2.0, 2.0);
        unsafe {
            SAVED_RAY = None;
            intersect(&s, &r);
            assert_eq!(SAVED_RAY.unwrap().origin, pt(0.0, 0.0, -2.5));
            assert_eq!(SAVED_RAY.unwrap().direction, v(0.0, 0.0, 0.5));
        };

        // intersecting a translated shape with a ray
        s.borrow_mut().transform = Matrix4x4::translation(5.0, 0.0, 0.0);
        unsafe {
            SAVED_RAY = None;
            intersect(&s, &r);
            assert_eq!(SAVED_RAY.unwrap().origin, pt(-5.0, 0.0, -5.0));
            assert_eq!(SAVED_RAY.unwrap().direction, v(0.0, 0.0, 1.0));
        };
    }

    #[test]
    fn sphere_glass() {
        // a helper for producing a sphere with a glassy material
        let s = glass_sphere();
        assert_eq!(s.borrow().transform, Matrix4x4::identity());
        assert_eq!(s.borrow().material.transparency, 1.0);
        assert_eq!(s.borrow().material.refractive_index, 1.5);
    }

    #[test]
    fn group_create() {
        // creating a new group
        let g = group();
        assert_eq!(g.borrow().transform, Matrix4x4::identity());
        let g = &g.borrow();
        if let ShapeForm::Group(children) = &g.form {
            assert_eq!(children.len(), 0);
        } else {
            panic!("failed!");
        }
    }

    #[test]
    fn group_add_children() {
        // adding a child to a group
        let mut g = group();
        let mut s = test_shape();
        add_child(&mut g, &mut s);
        let children = children(&g);
        assert_eq!(children.len(), 1);
        // assert!(children.iter().any(|e| e == &s));
        // assert_eq!(s.borrow().parent.is_some(), true);
        // assert_eq!(s.borrow().parent.as_ref().unwrap(), &g);
        for c in children.iter() {
            println!("{:?}", c);
        }
        // s.borrow().parent.is_some();
        // s.borrow().parent.as_ref().unwrap();
    }

    #[test]
    fn shape_parent() {
        // a shape has a parent attribute
        let s = test_shape();
        assert_eq!(s.borrow().parent, None);
    }

    #[test]
    fn shape_transformed_normal() {
        // computing the normal on a translated shape
        let s = test_shape();
        s.borrow_mut().transform = Matrix4x4::translation(0.0, 1.0, 0.0);
        let n = s.borrow_mut().normal_at(pt(0.0, 1.70711, -0.70711));
        assert_eq!(n, v(0.0, 0.70711, -0.70711));

        // computing the normal on a transformed shape
        let s = test_shape();
        s.borrow_mut().transform =
            Matrix4x4::scaling(1.0, 0.5, 1.0) * Matrix4x4::rotation_z(PI / 5.0);
        let n = s
            .borrow_mut()
            .normal_at(pt(0.0, 2.0_f64 / 2.0, -2.0_f64 / 2.0));
        assert_eq!(n, v(0.0, 0.97014, -0.24254));
    }

    #[test]
    fn ray_sphere_intersection() {
        // a ray intersects a sphere at two points
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let s = sphere();
        let xs = intersect(&s, &r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 4.0);
        assert_eq!(xs[0].object, sphere());
        assert_eq!(xs[1].t, 6.0);
        assert_eq!(xs[1].object, sphere());

        // a ray intersects a sphere at a tangent
        let r = Ray::new(pt(0.0, 1.0, -5.0), v(0.0, 0.0, 1.0));
        let xs = intersect(&s, &r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 5.0);
        assert_eq!(xs[1].t, 5.0);

        // a ray misses a sphere
        let r = Ray::new(pt(0.0, 2.0, -5.0), v(0.0, 0.0, 1.0));
        let xs = intersect(&s, &r);
        assert_eq!(xs.len(), 0);

        // a ray originates inside a sphere
        let r = Ray::new(pt(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0));
        let xs = intersect(&s, &r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, -1.0);
        assert_eq!(xs[1].t, 1.0);

        // a sphere is behind a ray
        let r = Ray::new(pt(0.0, 0.0, 5.0), v(0.0, 0.0, 1.0));
        let xs = intersect(&s, &r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, -6.0);
        assert_eq!(xs[1].t, -4.0);
    }

    #[test]
    fn ray_plane_intersection() {
        // intersect with a ray parallel to the plane
        let p = plane();
        let r = Ray::new(pt(0.0, 10.0, 0.0), v(0.0, 0.0, 1.0));
        let xs = local_intersect(&p, &r);
        assert_eq!(xs.len(), 0);

        // intersect with a coplanar ray
        let r = Ray::new(pt(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0));
        let xs = local_intersect(&p, &r);
        assert_eq!(xs.len(), 0);

        // a ray intersecting a plane from above
        let r = Ray::new(pt(0.0, 1.0, 0.0), v(0.0, -1.0, 0.0));
        let xs = local_intersect(&p, &r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 1.0);
        assert_eq!(xs[0].object, p);

        // a ray intersection a plane from below
        let r = Ray::new(pt(0.0, -1.0, 0.0), v(0.0, 1.0, 0.0));
        let xs = local_intersect(&p, &r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 1.0);
        assert_eq!(xs[0].object, p);
    }

    #[test]
    fn ray_intersection_with_scaled_sphere() {
        // intersecting a scaled sphere with a ray
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let s = spheret(Matrix4x4::scaling(2.0, 2.0, 2.0));
        let xs = intersect(&s, &r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 3.0);
        assert_eq!(xs[1].t, 7.0);

        // intersecting a translated sphere with a ray
        let s = spheret(Matrix4x4::translation(5.0, 0.0, 0.0));
        let xs = intersect(&s, &r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn ray_intersection_with_cube() {
        // a ray intersects a cube
        fn test(origin: Tuple, direction: Tuple, t1: f64, t2: f64) {
            let c = cube();
            let r = Ray::new(origin, direction);
            let xs = local_intersect(&c, &r);
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
            let xs = local_intersect(&c, &r);
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
            let xs = local_intersect(&cyl, &r);
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
            let xs = local_intersect(&cyl, &r);
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
            let xs = local_intersect(&cyl, &r);
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
            let xs = intersect(&cyl, &r);
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

    #[test]
    fn sphere_normal_at() {
        // the normal on a sphere at a point on the x axis
        let s = sphere();
        let n = s.borrow().normal_at(pt(1.0, 0.0, 0.0));
        assert_eq!(n, v(1.0, 0.0, 0.0));

        // the normal on a sphere at a point on the y axis
        let n = s.borrow().normal_at(pt(0.0, 1.0, 0.0));
        assert_eq!(n, v(0.0, 1.0, 0.0));

        // the normal on a sphere at a point on the y axis
        let n = s.borrow().normal_at(pt(0.0, 0.0, 1.0));
        assert_eq!(n, v(0.0, 0.0, 1.0));

        // the normal on a sphere at a point on a nonaxial point
        let n = s.borrow().normal_at(pt(
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

    #[test]
    fn plane_normal_at() {
        // the normal of a plane is constant everywhere
        let p = plane();
        let n1 = p.borrow().normal_at(pt(0.0, 0.0, 0.0));
        let n2 = p.borrow().normal_at(pt(10.0, 0.0, -10.0));
        let n3 = p.borrow().normal_at(pt(-5.0, 0.0, 150.0));
        assert_eq!(n1, v(0.0, 1.0, 0.0));
        assert_eq!(n2, v(0.0, 1.0, 0.0));
        assert_eq!(n3, v(0.0, 1.0, 0.0));
    }

    #[test]
    fn cube_normal_at() {
        // the normal of the surface of a cube
        fn test(point: Tuple, normal: Tuple) {
            let c = cube();
            let n = c.borrow().normal_at(point);
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
            let n = cyl.borrow().local_normal_at(point);
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
            let n = cyl.borrow().local_normal_at(point);
            assert_eq!(n, normal);
        }

        test(pt(0.0, 1.0, 0.0), v(0.0, -1.0, 0.0));
        test(pt(0.5, 1.0, 0.0), v(0.0, -1.0, 0.0));
        test(pt(0.0, 1.0, 0.5), v(0.0, -1.0, 0.0));
        test(pt(0.0, 2.0, 0.0), v(0.0, 1.0, 0.0));
        test(pt(0.5, 2.0, 0.0), v(0.0, 1.0, 0.0));
        test(pt(0.0, 2.0, 0.5), v(0.0, 1.0, 0.0));
    }
}
