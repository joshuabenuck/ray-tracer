use crate::{pt, v, Intersection, Material, Matrix4x4, Ray, Tuple, EPSILON};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ShapeForm {
    Test,
    Sphere,
    Plane,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Shape {
    pub transform: Matrix4x4,
    pub material: Material,
    pub form: ShapeForm,
}

static mut SAVED_RAY: Option<Ray> = None;

pub fn test_shape() -> Shape {
    Shape {
        transform: Matrix4x4::identity(),
        material: Material::new(),
        form: ShapeForm::Test,
    }
}

#[inline]
pub fn plane() -> Shape {
    Shape {
        transform: Matrix4x4::identity(),
        material: Material::new(),
        form: ShapeForm::Plane,
    }
}

#[inline]
pub fn planet(transform: Matrix4x4) -> Shape {
    planetm(transform, Material::new())
}

#[inline]
pub fn planem(material: Material) -> Shape {
    planetm(Matrix4x4::identity(), material)
}

#[inline]
pub fn planetm(transform: Matrix4x4, material: Material) -> Shape {
    Shape {
        material,
        transform,
        form: ShapeForm::Plane,
    }
}

#[inline]
pub fn sphere() -> Shape {
    spheretm(Matrix4x4::identity(), Material::new())
}

pub fn spheretm(transform: Matrix4x4, material: Material) -> Shape {
    Shape {
        transform,
        material,
        form: ShapeForm::Sphere,
    }
}

#[inline]
pub fn spheret(transform: Matrix4x4) -> Shape {
    spheretm(transform, Material::new())
}

#[inline]
pub fn spherem(material: Material) -> Shape {
    spheretm(Matrix4x4::identity(), material)
}

#[inline]
pub fn glass_sphere() -> Shape {
    let mut m = Material::new();
    m.transparency = 1.0;
    m.refractive_index = 1.5;
    spherem(m)
}

#[inline]
pub fn glass_spheret(transform: Matrix4x4) -> Shape {
    let mut m = Material::new();
    m.transparency = 1.0;
    m.refractive_index = 1.5;
    spheretm(transform, m)
}

impl Shape {
    pub fn intersects(&self, ray: &Ray) -> Vec<Intersection> {
        let ray = ray.transform(self.transform.inverse().unwrap());
        self.local_intersects(&ray)
    }

    pub fn local_intersects(&self, ray: &Ray) -> Vec<Intersection> {
        match self.form {
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
                    Intersection::new(t1, self.clone()),
                    Intersection::new(t2, self.clone()),
                ]
            }
            ShapeForm::Plane => {
                if ray.direction.y.abs() < EPSILON {
                    Vec::new()
                } else {
                    let t = -ray.origin.y / ray.direction.y;
                    vec![Intersection::new(t, self.clone())]
                }
            }
        }
    }

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
        let mut s = test_shape();
        assert_eq!(s.transform, Matrix4x4::identity());

        // assigning a transformation
        let t = Matrix4x4::translation(2.0, 3.0, 4.0);
        s.transform = t;
        assert_eq!(s.transform, t);

        // the default material
        assert_eq!(s.material, Material::new());

        // assigning a material
        let mut m = Material::new();
        m.ambient = 1.0;
        s.material = m;
        assert_eq!(s.material, m);

        // intersecting a scaled shape with a ray
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let mut s = test_shape();
        s.transform = Matrix4x4::scaling(2.0, 2.0, 2.0);
        unsafe {
            SAVED_RAY = None;
            s.intersects(&r);
            assert_eq!(SAVED_RAY.unwrap().origin, pt(0.0, 0.0, -2.5));
            assert_eq!(SAVED_RAY.unwrap().direction, v(0.0, 0.0, 0.5));
        };

        // intersecting a translated shape with a ray
        s.transform = Matrix4x4::translation(5.0, 0.0, 0.0);
        unsafe {
            SAVED_RAY = None;
            s.intersects(&r);
            assert_eq!(SAVED_RAY.unwrap().origin, pt(-5.0, 0.0, -5.0));
            assert_eq!(SAVED_RAY.unwrap().direction, v(0.0, 0.0, 1.0));
        };
    }

    #[test]
    fn sphere_glass() {
        // a helper for producing a sphere with a glassy material
        let s = glass_sphere();
        assert_eq!(s.transform, Matrix4x4::identity());
        assert_eq!(s.material.transparency, 1.0);
        assert_eq!(s.material.refractive_index, 1.5);
    }

    #[test]
    fn shape_transformed_normal() {
        // computing the normal on a translated shape
        let mut s = test_shape();
        s.transform = Matrix4x4::translation(0.0, 1.0, 0.0);
        let n = s.normal_at(pt(0.0, 1.70711, -0.70711));
        assert_eq!(n, v(0.0, 0.70711, -0.70711));

        // computing the normal on a transformed shape
        let mut s = test_shape();
        s.transform = Matrix4x4::scaling(1.0, 0.5, 1.0) * Matrix4x4::rotation_z(PI / 5.0);
        let n = s.normal_at(pt(0.0, 2.0_f64 / 2.0, -2.0_f64 / 2.0));
        assert_eq!(n, v(0.0, 0.97014, -0.24254));
    }

    #[test]
    fn ray_sphere_intersection() {
        // a ray intersects a sphere at two points
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let s = sphere();
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 4.0);
        assert_eq!(xs[0].object, sphere());
        assert_eq!(xs[1].t, 6.0);
        assert_eq!(xs[1].object, sphere());

        // a ray intersects a sphere at a tangent
        let r = Ray::new(pt(0.0, 1.0, -5.0), v(0.0, 0.0, 1.0));
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 5.0);
        assert_eq!(xs[1].t, 5.0);

        // a ray misses a sphere
        let r = Ray::new(pt(0.0, 2.0, -5.0), v(0.0, 0.0, 1.0));
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 0);

        // a ray originates inside a sphere
        let r = Ray::new(pt(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0));
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, -1.0);
        assert_eq!(xs[1].t, 1.0);

        // a sphere is behind a ray
        let r = Ray::new(pt(0.0, 0.0, 5.0), v(0.0, 0.0, 1.0));
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, -6.0);
        assert_eq!(xs[1].t, -4.0);
    }

    #[test]
    fn ray_plane_intersection() {
        // intersect with a ray parallel to the plane
        let p = plane();
        let r = Ray::new(pt(0.0, 10.0, 0.0), v(0.0, 0.0, 1.0));
        let xs = p.local_intersects(&r);
        assert_eq!(xs.len(), 0);

        // intersect with a coplanar ray
        let r = Ray::new(pt(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0));
        let xs = p.local_intersects(&r);
        assert_eq!(xs.len(), 0);

        // a ray intersecting a plane from above
        let r = Ray::new(pt(0.0, 1.0, 0.0), v(0.0, -1.0, 0.0));
        let xs = p.local_intersects(&r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 1.0);
        assert_eq!(xs[0].object, p);

        // a ray intersection a plane from below
        let r = Ray::new(pt(0.0, -1.0, 0.0), v(0.0, 1.0, 0.0));
        let xs = p.local_intersects(&r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 1.0);
        assert_eq!(xs[0].object, p);
    }

    #[test]
    fn ray_intersection_with_scaled_sphere() {
        // intersecting a scaled sphere with a ray
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let s = spheret(Matrix4x4::scaling(2.0, 2.0, 2.0));
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 3.0);
        assert_eq!(xs[1].t, 7.0);

        // intersecting a translated sphere with a ray
        let s = spheret(Matrix4x4::translation(5.0, 0.0, 0.0));
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn sphere_normal_at() {
        // the normal on a sphere at a point on the x axis
        let s = sphere();
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

    #[test]
    fn plane_normal_at() {
        // the normal of a plane is constant everywhere
        let p = plane();
        let n1 = p.normal_at(pt(0.0, 0.0, 0.0));
        let n2 = p.normal_at(pt(10.0, 0.0, -10.0));
        let n3 = p.normal_at(pt(-5.0, 0.0, 150.0));
        assert_eq!(n1, v(0.0, 1.0, 0.0));
        assert_eq!(n2, v(0.0, 1.0, 0.0));
        assert_eq!(n3, v(0.0, 1.0, 0.0));
    }
}
