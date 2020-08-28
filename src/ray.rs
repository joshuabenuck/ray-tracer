use crate::{Color, Matrix4x4, Tuple, EPSILON};

#[derive(Copy, Clone)]
pub struct Ray {
    pub origin: Tuple,
    pub direction: Tuple,
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
pub struct Sphere {
    pub transform: Matrix4x4,
    pub material: Material,
}

impl Sphere {
    pub fn new(transform: Option<Matrix4x4>, material: Option<Material>) -> Sphere {
        Sphere {
            transform: transform.unwrap_or(Matrix4x4::identity()),
            material: material.unwrap_or(Material::new()),
        }
    }

    pub fn intersects(&self, ray: &Ray) -> Vec<Intersection> {
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
            Intersection::new(t1, self.clone()),
            Intersection::new(t2, self.clone()),
        ]
    }

    pub fn normal_at(&self, world_point: Tuple) -> Tuple {
        let object_point = self.transform.inverse().unwrap() * world_point;
        let object_normal = object_point - Tuple::point(0.0, 0.0, 0.0);
        // technically should be self.tranform.submatrix(3, 3)
        // to avoid messing with the w coordinate when there is any kind of translation
        // in the transform
        let mut world_normal = self.transform.inverse().unwrap().transpose() * object_normal;
        // workaround to avoid the submatrix calculation
        world_normal.w = 0.0;
        world_normal.normalize()
    }
}

pub struct Comps {
    pub t: f64,
    pub object: Sphere,
    pub point: Tuple,
    pub eyev: Tuple,
    pub normalv: Tuple,
    pub inside: bool,
    pub over_point: Tuple,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Intersection {
    pub t: f64,
    pub object: Sphere,
}

impl Intersection {
    pub fn new(t: f64, object: Sphere) -> Intersection {
        Intersection { t, object }
    }

    pub fn prepare_computations(&self, ray: &Ray) -> Comps {
        // copy the intersection's properties, for convenience
        let t = self.t;
        let object = self.object.clone();

        // precompute some useful values
        let point = ray.position(t);
        let eyev = -ray.direction;
        let normalv = object.normal_at(point);
        let inside = normalv.dot(&eyev) < 0.0;
        let normalv = if inside { -normalv } else { normalv };
        let over_point = point + normalv * EPSILON;

        // instantiate a data structure for storing some precomputed values
        Comps {
            t,
            object,
            point,
            eyev,
            normalv,
            inside,
            over_point,
        }
    }
}

pub trait Intersections {
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PointLight {
    pub position: Tuple,
    pub intensity: Color,
}

impl PointLight {
    pub fn new(position: Tuple, intensity: Color) -> PointLight {
        PointLight {
            position,
            intensity,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Material {
    pub color: Color,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
}

impl Material {
    pub fn new() -> Material {
        Material {
            color: Color::new(1.0, 1.0, 1.0),
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0,
        }
    }

    pub fn lighting(
        &self,
        light: &PointLight,
        point: &Tuple,
        eyev: &Tuple,
        normalv: &Tuple,
        in_shadow: bool,
    ) -> Color {
        // combine the surface color with the light's color / intensity
        let effective_color = self.color * light.intensity;

        // find the direction to the light source
        let lightv = (light.position - *point).normalize();

        // comput the ambient contribution
        let ambient = effective_color * self.ambient;
        if in_shadow {
            return ambient;
        }

        // light_dot_normal represents the cosine of the angle between the
        // light vector and the normal vector. A negative number means the
        // light is on the other side of teh surface.
        let light_dot_normal = lightv.dot(&normalv);
        let (diffuse, specular) = if light_dot_normal < 0.0 {
            (Color::new(0.0, 0.0, 0.0), Color::new(0.0, 0.0, 0.0))
        } else {
            // compute the diffuse contribution
            let diffuse = effective_color * self.diffuse * light_dot_normal;

            // reflect-dot_eye represents the cosine of the angle between the
            // reflection vector and the eye vector. A negative number means the
            // light reflects away from the eye.
            let reflectv = -lightv.reflect(&normalv);
            let reflect_dot_eye = reflectv.dot(eyev);

            let specular = if reflect_dot_eye <= 0.0 {
                Color::new(0.0, 0.0, 0.0)
            } else {
                // compute the specular contribution
                let factor = reflect_dot_eye.powf(self.shininess);
                light.intensity * self.specular * factor
            };
            (diffuse, specular)
        };
        ambient + diffuse + specular
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

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
        let s = Sphere::new(None, None);
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 4.0);
        assert_eq!(xs[0].object, Sphere::new(None, None));
        assert_eq!(xs[1].t, 6.0);
        assert_eq!(xs[1].object, Sphere::new(None, None));

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
        let s = Sphere::new(None, None);
        let i = Intersection::new(3.5, s);
        assert_eq!(3.5, i.t);
        // assert_eq!(i.object, s);

        // aggregate intersections
        let s = Sphere::new(None, None);
        let i1 = Intersection::new(1.0, s.clone());
        let i2 = Intersection::new(2.0, s);
        let mut xs = [i1, i2];
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 1.0);
        assert_eq!(xs[1].t, 2.0);

        // the hit when all intersections have positive t
        let s = Sphere::new(None, None);
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
        let mut s = Sphere::new(None, None);
        assert_eq!(s.transform, Matrix4x4::identity());

        // changing a sphere's transformation
        let t = Matrix4x4::translation(2.0, 3.0, 4.0);
        s.transform = t;
        assert_eq!(s.transform, t);
    }

    #[test]
    fn ray_intersection_with_scaled_sphere() {
        // intersecting a scaled sphere with a ray
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new(Some(Matrix4x4::scaling(2.0, 2.0, 2.0)), None);
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 3.0);
        assert_eq!(xs[1].t, 7.0);

        // intersecting a translated sphere with a ray
        let s = Sphere::new(Some(Matrix4x4::translation(5.0, 0.0, 0.0)), None);
        let xs = s.intersects(&r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn sphere_normal_at() {
        // the normal on a sphere at a point on the x axis
        let s = Sphere::new(None, None);
        let n = s.normal_at(Tuple::point(1.0, 0.0, 0.0));
        assert_eq!(n, Tuple::vector(1.0, 0.0, 0.0));

        // the normal on a sphere at a point on the y axis
        let n = s.normal_at(Tuple::point(0.0, 1.0, 0.0));
        assert_eq!(n, Tuple::vector(0.0, 1.0, 0.0));

        // the normal on a sphere at a point on the y axis
        let n = s.normal_at(Tuple::point(0.0, 0.0, 1.0));
        assert_eq!(n, Tuple::vector(0.0, 0.0, 1.0));

        // the normal on a sphere at a point on a nonaxial point
        let n = s.normal_at(Tuple::point(
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
        ));
        assert_eq!(
            n,
            Tuple::vector(
                3.0_f64.sqrt() / 3.0,
                3.0_f64.sqrt() / 3.0,
                3.0_f64.sqrt() / 3.0
            )
        );

        // the normal is a normalized vector
        assert_eq!(n, n.normalize());
    }

    #[test]
    fn sphere_transformed_normal() {
        // computing the normal on a translated sphere
        let s = Sphere::new(Some(Matrix4x4::translation(0.0, 1.0, 0.0)), None);
        let n = s.normal_at(Tuple::point(0.0, 1.70711, -0.70711));
        assert_eq!(n, Tuple::vector(0.0, 0.70711, -0.70711));

        // computing the normal on a transformed sphere
        let s = Sphere::new(
            Some(Matrix4x4::scaling(1.0, 0.5, 1.0) * Matrix4x4::rotation_z(PI / 5.0)),
            None,
        );
        let n = s.normal_at(Tuple::point(0.0, 2.0_f64 / 2.0, -2.0_f64 / 2.0));
        assert_eq!(n, Tuple::vector(0.0, 0.97014, -0.24254));
    }

    #[test]
    fn point_light() {
        // a point light has a position and an intensity
        let intensity = Color::new(1.0, 1.0, 1.0);
        let position = Tuple::point(0.0, 0.0, 0.0);
        let light = PointLight::new(position, intensity);
        assert_eq!(light.position, position);
        assert_eq!(light.intensity, intensity);
    }
    #[test]
    fn material() {
        // the default material
        let m = Material::new();
        assert_eq!(m.color, Color::new(1.0, 1.0, 1.0));
        assert_eq!(m.ambient, 0.1);
        assert_eq!(m.diffuse, 0.9);
        assert_eq!(m.specular, 0.9);
        assert_eq!(m.shininess, 200.0);
    }

    #[test]
    fn sphere_material() {
        // a sphere has a default material
        let s = Sphere::new(None, None);
        let m = s.material;
        assert_eq!(m, Material::new());

        // a sphere may be assigned a material
        let mut s = Sphere::new(None, None);
        let mut m = Material::new();
        m.ambient = 1.0;
        s.material = m;
        assert_eq!(s.material, m);
    }

    #[test]
    fn lighting() {
        let m = Material::new();
        let position = Tuple::point(0.0, 0.0, 0.0);

        // lighting with the eye between the light and the surface
        // ambient, diffuse, and specular all at full strength
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = m.lighting(&light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.9, 1.9, 1.9));

        // lighting with the surface in shadow
        let in_shadow = true;
        let result = m.lighting(&light, &position, &eyev, &normalv, in_shadow);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));

        // lighting with the eye between light and surface, eye offset 45 degrees
        // ambient and diffuse unchanged because the angle between them is unchanged
        // specular drops off to effectively zero
        let eyev = Tuple::vector(0.0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0);
        let result = m.lighting(&light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.0, 1.0, 1.0));

        // lighting with eye opposite surface, light offset 45 degrees
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = m.lighting(&light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(0.7364, 0.7364, 0.7364));

        // lighting with eye in the path of the reflection vector
        // makes specular at full strength with ambient and diffuse same as last test
        let eyev = Tuple::vector(0.0, -2.0_f64.sqrt() / 2.0, -2.0_f64.sqrt() / 2.0);
        let light = PointLight::new(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = m.lighting(&light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.6364, 1.6364, 1.6364,));

        // light with the light behind the surface
        // Since the light doesn't illuminate the surface, the diffuse and specular
        // components go to zero.
        // The total intensity should be the same as the ambient component
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Tuple::point(0.0, 0.0, 10.0), Color::new(1.0, 1.0, 1.0));
        let result = m.lighting(&light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn intersection_precomputation() {
        // precomputing the state of an interesection
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = Sphere::new(None, None);
        let i = Intersection::new(4.0, shape);
        let comps = i.prepare_computations(&r);
        assert_eq!(comps.t, i.t);
        assert_eq!(comps.point, Tuple::point(0.0, 0.0, -1.0));
        assert_eq!(comps.eyev, Tuple::vector(0.0, 0.0, -1.0));
        assert_eq!(comps.normalv, Tuple::vector(0.0, 0.0, -1.0));

        // the hit, when an intersection occurs on the outside
        assert_eq!(comps.inside, false);

        // the hit, when an intersection occurs on the inside
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let i = Intersection::new(1.0, shape);
        let comps = i.prepare_computations(&r);
        assert_eq!(comps.t, i.t);
        assert_eq!(comps.point, Tuple::point(0.0, 0.0, 1.0));
        assert_eq!(comps.eyev, Tuple::vector(0.0, 0.0, -1.0));
        assert_eq!(comps.inside, true);
        // normal would have been (0.0, 0.0, 1.0), but is inverted!
        assert_eq!(comps.normalv, Tuple::vector(0.0, 0.0, -1.0));
    }
}
