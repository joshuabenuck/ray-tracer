use crate::{Ray, Shape, Tuple, EPSILON};

pub fn schlick(comps: &Comps) -> f64 {
    // find the cosine of the angle between the eye and the normal vectors
    let mut cos = comps.eyev.dot(&comps.normalv);

    // total internal reflection can only occur if n1 > n2
    if comps.n1 > comps.n2 {
        let n = comps.n1 / comps.n2;
        let sin2_t = n.powi(2) * (1.0 - cos.powi(2));
        if sin2_t > 1.0 {
            return 1.0;
        }

        // compute the cosine of theta_t using trig identity
        let cos_t = (1.0 - sin2_t).sqrt();

        // when n1 > n2, use cos(theta_t) insstead
        cos = cos_t;
    }

    let r0 = ((comps.n1 - comps.n2) / (comps.n1 + comps.n2)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cos).powi(5)
}

pub struct Comps<'a> {
    pub t: f64,
    pub object: &'a dyn Shape,
    pub point: Tuple,
    pub eyev: Tuple,
    pub normalv: Tuple,
    pub reflectv: Tuple,
    pub inside: bool,
    pub over_point: Tuple,
    pub under_point: Tuple,
    pub n1: f64,
    pub n2: f64,
}

#[derive(Clone, Debug)]
pub struct Intersection<'a> {
    pub t: f64,
    pub object: &'a dyn Shape,
    pub uv: Option<(f64, f64)>,
}

impl PartialEq for Intersection<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t && self.object == other.object
    }
}

impl<'a> Intersection<'a> {
    pub fn new(t: f64, object: &'a dyn Shape) -> Intersection<'a> {
        Intersection {
            t,
            object,
            uv: None,
        }
    }

    pub fn with_uv(t: f64, object: &'a dyn Shape, u: f64, v: f64) -> Intersection<'a> {
        Intersection {
            t,
            object,
            uv: Some((u, v)),
        }
    }

    pub fn prepare_computations(&self, ray: &Ray, xs: &Vec<Intersection>) -> Comps {
        // copy the intersection's properties, for convenience
        let t = self.t;
        let object = self.object.clone();

        // precompute some useful values
        let point = ray.position(t);
        let eyev = -ray.direction;
        let normalv = object.normal_at(point, &self);
        let inside = normalv.dot(&eyev) < 0.0;
        let normalv = if inside { -normalv } else { normalv };
        let reflectv = ray.direction.reflect(&normalv);
        let over_point = point + normalv * EPSILON;
        let under_point = point - normalv * EPSILON;

        let mut n1 = 1.0;
        let mut n2 = 1.0;
        let mut containers: Vec<&dyn Shape> = Vec::new();
        for i in xs {
            if i == self {
                if containers.len() == 0 {
                    n1 = 1.0;
                } else {
                    n1 = containers.last().unwrap().material().refractive_index;
                }
            }

            let pos = containers.iter().position(|c| &&**c == &i.object);
            if let Some(pos) = pos {
                containers.remove(pos);
            } else {
                containers.push(i.object);
            }

            if i == self {
                if containers.len() == 0 {
                    n2 = 1.0;
                } else {
                    n2 = containers.last().unwrap().material().refractive_index;
                }
                break;
            }
        }

        // instantiate a data structure for storing some precomputed values
        Comps {
            t,
            object,
            point,
            eyev,
            normalv,
            reflectv,
            inside,
            over_point,
            under_point,
            n1,
            n2,
        }
    }
}

pub trait Intersections {
    fn hit(&self) -> Option<&Intersection<'_>>;
}

impl Intersections for Vec<Intersection<'_>> {
    fn hit(&self) -> Option<&Intersection<'_>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{equal, pt, v, Matrix4x4, Plane, Sphere};

    #[test]
    fn intersection() {
        // an intersection encapsulates t and object
        let s = Sphere::new();
        let i = Intersection::new(3.5, &s);
        assert_eq!(3.5, i.t);
        assert_eq!(i.object, &s as &dyn Shape);

        // aggregate intersections
        let s = Sphere::new();
        let i1 = Intersection::new(1.0, &s);
        let i2 = Intersection::new(2.0, &s);
        let mut xs = [i1, i2];
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 1.0);
        assert_eq!(xs[1].t, 2.0);

        // the hit when all intersections have positive t
        let s = Sphere::new();
        let i1 = Intersection::new(1.0, &s);
        let i2 = Intersection::new(2.0, &s);
        let mut xs = vec![i2.clone(), i1.clone()];
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        let i = xs.hit();
        assert!(*i.unwrap() == i1);

        // the hit when intersections have negative t
        let i1 = Intersection::new(-1.0, &s);
        let i2 = Intersection::new(1.0, &s);
        let mut xs = vec![i2.clone(), i1.clone()];
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        let i = xs.hit();
        assert!(*i.unwrap() == i2);

        // the hit when all intersections have negative t
        let i1 = Intersection::new(-2.0, &s);
        let i2 = Intersection::new(-1.0, &s);
        let mut xs = vec![i2, i1];
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        let i = xs.hit();
        assert!(i == None);

        // the hit is always the lowest non-negative intersection
        let i1 = Intersection::new(5.0, &s);
        let i2 = Intersection::new(7.0, &s);
        let i3 = Intersection::new(-3.0, &s);
        let i4 = Intersection::new(2.0, &s);
        let mut xs = vec![i1, i2, i3, i4.clone()];
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        let i = xs.hit();
        assert!(*i.unwrap() == i4);
    }

    #[test]
    fn intersection_precomputation() {
        // precomputing the state of an interesection
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let shape = Sphere::new();
        let i = Intersection::new(4.0, &shape);
        let comps = i.prepare_computations(&r, &vec![i.clone()]);
        assert_eq!(comps.t, i.t);
        assert_eq!(comps.point, pt(0.0, 0.0, -1.0));
        assert_eq!(comps.eyev, v(0.0, 0.0, -1.0));
        assert_eq!(comps.normalv, v(0.0, 0.0, -1.0));

        // the hit, when an intersection occurs on the outside
        assert_eq!(comps.inside, false);

        // the hit, when an intersection occurs on the inside
        let r = Ray::new(pt(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0));
        let i = Intersection::new(1.0, &shape);
        let comps = i.prepare_computations(&r, &vec![i.clone()]);
        assert_eq!(comps.t, i.t);
        assert_eq!(comps.point, pt(0.0, 0.0, 1.0));
        assert_eq!(comps.eyev, v(0.0, 0.0, -1.0));
        assert_eq!(comps.inside, true);
        // normal would have been (0.0, 0.0, 1.0), but is inverted!
        assert_eq!(comps.normalv, v(0.0, 0.0, -1.0));

        // precomputing the reflection vector
        let shape = Plane::new();
        let r = Ray::new(
            pt(0.0, 1.0, -1.0),
            v(0.0, -2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0),
        );
        let i = Intersection::new(2.0_f64.sqrt(), &shape);
        let comps = i.prepare_computations(&r, &vec![i.clone()]);
        assert_eq!(
            comps.reflectv,
            v(0.0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0)
        );
    }

    #[test]
    fn intersections_n1_n2() {
        let a = Sphere::glass().transform(Matrix4x4::scaling(2.0, 2.0, 2.0));
        let mut b = Sphere::glass().transform(Matrix4x4::translation(0.0, 0.0, -0.25));
        b.material_mut().refractive_index = 2.0;
        let mut c = Sphere::glass().transform(Matrix4x4::translation(0.0, 0.0, 0.25));
        c.material_mut().refractive_index = 2.5;
        let r = Ray::new(pt(0.0, 0.0, -4.0), v(0.0, 0.0, 1.0));
        let xs = vec![
            Intersection::new(2.0, &a),
            Intersection::new(2.75, &b),
            Intersection::new(3.25, &c),
            Intersection::new(4.75, &b),
            Intersection::new(5.25, &c),
            Intersection::new(6.0, &a),
        ];
        let i = &xs[0];
        let comps = i.prepare_computations(&r, &xs);
        assert_eq!(comps.n1, 1.0);
        assert_eq!(comps.n2, 1.5);
        let i = &xs[1];
        let comps = i.prepare_computations(&r, &xs);
        assert_eq!(comps.n1, 1.5);
        assert_eq!(comps.n2, 2.0);
        let i = &xs[2];
        let comps = i.prepare_computations(&r, &xs);
        assert_eq!(comps.n1, 2.0);
        assert_eq!(comps.n2, 2.5);
        let i = &xs[3];
        let comps = i.prepare_computations(&r, &xs);
        assert_eq!(comps.n1, 2.5);
        assert_eq!(comps.n2, 2.5);
        let i = &xs[4];
        let comps = i.prepare_computations(&r, &xs);
        assert_eq!(comps.n1, 2.5);
        assert_eq!(comps.n2, 1.5);
        let i = &xs[5];
        let comps = i.prepare_computations(&r, &xs);
        assert_eq!(comps.n1, 1.5);
        assert_eq!(comps.n2, 1.0);
    }

    #[test]
    fn comps_under_point() {
        // the under point is offset below the surface
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let shape = Sphere::glass().transform(Matrix4x4::translation(0.0, 0.0, 1.0));
        let i = Intersection::new(5.0, &shape);
        let xs = vec![i.clone()];
        let comps = i.prepare_computations(&r, &xs);
        assert_eq!(comps.under_point.z > EPSILON / 2.0, true);
        assert_eq!(comps.point.z < comps.under_point.z, true);
    }

    #[test]
    fn comps_schlick() {
        // the schlick approximation under total internal reflection
        let shape = Sphere::glass();
        let r = Ray::new(pt(0.0, 0.0, 2.0_f64.sqrt() / 2.0), v(0.0, 1.0, 0.0));
        let xs = vec![
            Intersection::new(-2.0_f64.sqrt() / 2.0, &shape),
            Intersection::new(2.0_f64.sqrt() / 2.0, &shape),
        ];
        let i = xs[1].clone();
        let comps = i.prepare_computations(&r, &xs);
        let reflectance = schlick(&comps);
        // total internal feflection means all the light is reflected
        // and none is refracted. The fraction of the light that is
        // reflected must be 1 in this case. This is called reflectance.
        assert_eq!(reflectance, 1.0);

        // the schlick approximation with a perpendicular viewing angle
        let shape = Sphere::glass();
        let r = Ray::new(pt(0.0, 0.0, 0.0), v(0.0, 1.0, 0.0));
        let xs = vec![
            Intersection::new(-1.0, &shape),
            Intersection::new(1.0, &shape),
        ];
        let i = xs[1].clone();
        let comps = i.prepare_computations(&r, &xs);
        let reflectance = schlick(&comps);
        assert_eq!(equal(reflectance, 0.04), true);

        // the schlick approximation with small angle and n2 > n1
        let shape = Sphere::glass();
        let r = Ray::new(pt(0.0, 0.99, -2.0), v(0.0, 0.0, 1.0));
        let xs = vec![Intersection::new(1.8589, &shape)];
        let i = xs[0].clone();
        let comps = i.prepare_computations(&r, &xs);
        let reflectance = schlick(&comps);
        assert_eq!(
            equal(reflectance, 0.48873),
            true,
            "{} != {}",
            reflectance,
            0.48873
        );
    }
}
