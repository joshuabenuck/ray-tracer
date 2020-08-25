use crate::{Matrix4x4, Tuple};

#[derive(Copy, Clone)]
pub struct Ray {
    origin: Tuple,
    direction: Tuple,
}

impl Ray {
    pub fn new(origin: Tuple, direction: Tuple) -> Ray {
        Ray { origin, direction }
    }

    pub fn position(&self, t: f64) -> Tuple {
        self.origin + self.direction * t
    }

    pub fn transform(&self, m: Matrix4x4) -> Ray {
        Ray {
            origin: m * self.origin,
            direction: m * self.direction,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Sphere {
    transform: Matrix4x4,
}

impl Sphere {
    fn new(transform: Option<Matrix4x4>) -> Sphere {
        Sphere {
            transform: transform.unwrap_or(Matrix4x4::identity()),
        }
    }

    fn intersects(&self, ray: &Ray) -> Vec<Intersection> {
        let ray = ray.transform(self.transform.inverse().unwrap());
        let sphere_to_ray = ray.origin - Tuple::point(0.0, 0.0, 0.0);

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
            Intersection::new(t1, Sphere::new(None)),
            Intersection::new(t2, Sphere::new(None)),
        ]
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Intersection {
    t: f64,
    object: Sphere,
}

impl Intersection {
    fn new(t: f64, object: Sphere) -> Intersection {
        Intersection { t, object }
    }
}

trait Intersections {
    fn hit(&self) -> Option<&Intersection>;
}

impl Intersections for Vec<Intersection> {
    fn hit(&self) -> Option<&Intersection> {
        if self.len() == 0 {
            return None;
        }
        for i in self.iter() {
            if i.t > 0.0 {
                return Some(i);
            }
        }
        None
    }
}

mod tests {
    use super::*;

    #[test]
    fn ray_create() {
        // creating and querying a ray
        let origin = Tuple::point(1.0, 2.0, 3.0);
        let direction = Tuple::vector(4.0, 5.0, 6.0);
        let r = Ray::new(origin, direction);
        assert_eq!(r.origin, origin);
        assert_eq!(r.direction, direction);
    }

    #[test]
    fn ray_point_from_distance() {
        // computing a point from a distance
        let r = Ray::new(Tuple::point(2.0, 3.0, 4.0), Tuple::vector(1.0, 0.0, 0.0));
        assert_eq!(r.position(0.0), Tuple::point(2.0, 3.0, 4.0));
        assert_eq!(r.position(1.0), Tuple::point(3.0, 3.0, 4.0));
        assert_eq!(r.position(-1.0), Tuple::point(1.0, 3.0, 4.0));
        assert_eq!(r.position(2.5), Tuple::point(4.5, 3.0, 4.0));
    }

    #[test]
    fn ray_sphere_intersection() {
        // a ray intersects a sphere at two points
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new(None);
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 4.0);
        assert_eq!(xs[0].object, Sphere::new(None));
        assert_eq!(xs[1].t, 6.0);
        assert_eq!(xs[1].object, Sphere::new(None));

        // a ray intersects a sphere at a tangent
        let r = Ray::new(Tuple::point(0.0, 1.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 5.0);
        assert_eq!(xs[1].t, 5.0);

        // a ray misses a sphere
        let r = Ray::new(Tuple::point(0.0, 2.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 0);

        // a ray originates inside a sphere
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, -1.0);
        assert_eq!(xs[1].t, 1.0);

        // a sphere is behind a ray
        let r = Ray::new(Tuple::point(0.0, 0.0, 5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, -6.0);
        assert_eq!(xs[1].t, -4.0);
    }

    #[test]
    fn intersection() {
        // an intersection encapsulates t and object
        let s = Sphere::new(None);
        let i = Intersection::new(3.5, s);
        assert_eq!(3.5, i.t);
        // assert_eq!(i.object, s);

        // aggregate intersections
        let s = Sphere::new(None);
        let i1 = Intersection::new(1.0, s.clone());
        let i2 = Intersection::new(2.0, s);
        let mut xs = [i1, i2];
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 1.0);
        assert_eq!(xs[1].t, 2.0);

        // the hit when all intersections have positive t
        let s = Sphere::new(None);
        let i1 = Intersection::new(1.0, s.clone());
        let i2 = Intersection::new(2.0, s.clone());
        let mut xs = vec![i2, i1];
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        let i = xs.hit();
        assert_eq!(*i.unwrap(), i1);

        // the hit when intersections have negative t
        let i1 = Intersection::new(-1.0, s.clone());
        let i2 = Intersection::new(1.0, s.clone());
        let mut xs = vec![i2, i1];
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        let i = xs.hit();
        assert_eq!(*i.unwrap(), i2);

        // the hit when all intersections have negative t
        let i1 = Intersection::new(-2.0, s.clone());
        let i2 = Intersection::new(-1.0, s.clone());
        let mut xs = vec![i2, i1];
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        let i = xs.hit();
        assert_eq!(i, None);

        // the hit is always the lowest non-negative intersection
        let i1 = Intersection::new(5.0, s.clone());
        let i2 = Intersection::new(7.0, s.clone());
        let i3 = Intersection::new(-3.0, s.clone());
        let i4 = Intersection::new(2.0, s.clone());
        let mut xs = vec![i1, i2, i3, i4];
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        let i = xs.hit();
        assert_eq!(*i.unwrap(), i4);
    }

    #[test]
    fn ray_translation() {
        // translating a ray
        let r = Ray::new(Tuple::point(1.0, 2.0, 3.0), Tuple::vector(0.0, 1.0, 0.0));
        let m = Matrix4x4::translation(3.0, 4.0, 5.0);
        let r2 = r.transform(m);
        assert_eq!(r2.origin, Tuple::point(4.0, 6.0, 8.0));
        assert_eq!(r2.direction, Tuple::vector(0.0, 1.0, 0.0));

        // scaling a ray
        let m = Matrix4x4::scaling(2.0, 3.0, 4.0);
        let r2 = r.transform(m);
        assert_eq!(r2.origin, Tuple::point(2.0, 6.0, 12.0));
        assert_eq!(r2.direction, Tuple::vector(0.0, 3.0, 0.0));
    }

    #[test]
    fn sphere_transformation() {
        // a sphere's default transformation
        let mut s = Sphere::new(None);
        assert_eq!(s.transform, Matrix4x4::identity());

        // changing a sphere's transformation
        let t = Matrix4x4::translation(2.0, 3.0, 4.0);
        s.transform = t;
        assert_eq!(s.transform, t);
    }

    fn ray_intersection_with_scaled_sphere() {
        // intersecting a scaled sphere with a ray
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new(Some(Matrix4x4::scaling(2.0, 2.0, 2.0)));
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 3.0);
        assert_eq!(xs[1].t, 7.0);

        // intersecting a translated sphere with a ray
        let s = Sphere::new(Some(Matrix4x4::translation(5.0, 0.0, 0.0)));
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 0);
    }
}