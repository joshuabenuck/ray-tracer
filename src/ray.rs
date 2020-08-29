use crate::{pt, v, Color, Matrix4x4, Pattern, Tuple, EPSILON};

#[derive(Debug, Copy, Clone, PartialEq)]
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

fn test_shape() -> Shape {
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
            _ => unreachable!(),
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
            _ => unreachable!(),
        }
    }
}

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

pub struct Comps {
    pub t: f64,
    pub object: Shape,
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Intersection {
    pub t: f64,
    pub object: Shape,
}

impl Intersection {
    pub fn new(t: f64, object: Shape) -> Intersection {
        Intersection { t, object }
    }

    pub fn prepare_computations(&self, ray: &Ray, xs: Vec<Intersection>) -> Comps {
        // copy the intersection's properties, for convenience
        let t = self.t;
        let object = self.object.clone();

        // precompute some useful values
        let point = ray.position(t);
        let eyev = -ray.direction;
        let normalv = object.normal_at(point);
        let inside = normalv.dot(&eyev) < 0.0;
        let normalv = if inside { -normalv } else { normalv };
        let reflectv = ray.direction.reflect(&normalv);
        let over_point = point + normalv * EPSILON;
        let under_point = point - normalv * EPSILON;

        let mut n1 = 1.0;
        let mut n2 = 1.0;
        let mut containers: Vec<Shape> = Vec::new();
        for i in xs {
            if &i == self {
                if containers.len() == 0 {
                    n1 = 1.0;
                } else {
                    n1 = containers.last().unwrap().material.refractive_index;
                }
            }

            let pos = containers.iter().position(|c| c == &i.object);
            if let Some(pos) = pos {
                containers.remove(pos);
            } else {
                containers.push(i.object);
            }

            if &i == self {
                if containers.len() == 0 {
                    n2 = 1.0;
                } else {
                    n2 = containers.last().unwrap().material.refractive_index;
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
    pub pattern: Option<Pattern>,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
    pub reflective: f64,
    pub transparency: f64,
    pub refractive_index: f64,
}

impl Material {
    pub fn new() -> Material {
        Material {
            color: Color::new(1.0, 1.0, 1.0),
            pattern: None,
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0,
            reflective: 0.0,
            transparency: 0.0,
            refractive_index: 1.0,
        }
    }

    pub fn lighting(
        &self,
        object: &Shape,
        light: &PointLight,
        point: &Tuple,
        eyev: &Tuple,
        normalv: &Tuple,
        in_shadow: bool,
    ) -> Color {
        let color = if let Some(pattern) = self.pattern {
            pattern.pattern_at_object(*object, *point)
        } else {
            self.color
        };
        // combine the surface color with the light's color / intensity
        let effective_color = color * light.intensity;

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
    use crate::{black, equal, stripe_pattern, white};
    use std::f64::consts::PI;

    #[test]
    fn ray_create() {
        // creating and querying a ray
        let origin = pt(1.0, 2.0, 3.0);
        let direction = v(4.0, 5.0, 6.0);
        let r = Ray::new(origin, direction);
        assert_eq!(r.origin, origin);
        assert_eq!(r.direction, direction);
    }

    #[test]
    fn ray_point_from_distance() {
        // computing a point from a distance
        let r = Ray::new(pt(2.0, 3.0, 4.0), v(1.0, 0.0, 0.0));
        assert_eq!(r.position(0.0), pt(2.0, 3.0, 4.0));
        assert_eq!(r.position(1.0), pt(3.0, 3.0, 4.0));
        assert_eq!(r.position(-1.0), pt(1.0, 3.0, 4.0));
        assert_eq!(r.position(2.5), pt(4.5, 3.0, 4.0));
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
    fn intersection() {
        // an intersection encapsulates t and object
        let s = sphere();
        let i = Intersection::new(3.5, s);
        assert_eq!(3.5, i.t);
        // assert_eq!(i.object, s);

        // aggregate intersections
        let s = sphere();
        let i1 = Intersection::new(1.0, s.clone());
        let i2 = Intersection::new(2.0, s);
        let mut xs = [i1, i2];
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 1.0);
        assert_eq!(xs[1].t, 2.0);

        // the hit when all intersections have positive t
        let s = sphere();
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
        let r = Ray::new(pt(1.0, 2.0, 3.0), v(0.0, 1.0, 0.0));
        let m = Matrix4x4::translation(3.0, 4.0, 5.0);
        let r2 = r.transform(m);
        assert_eq!(r2.origin, pt(4.0, 6.0, 8.0));
        assert_eq!(r2.direction, v(0.0, 1.0, 0.0));

        // scaling a ray
        let m = Matrix4x4::scaling(2.0, 3.0, 4.0);
        let r2 = r.transform(m);
        assert_eq!(r2.origin, pt(2.0, 6.0, 12.0));
        assert_eq!(r2.direction, v(0.0, 3.0, 0.0));
    }

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
    fn point_light() {
        // a point light has a position and an intensity
        let intensity = Color::new(1.0, 1.0, 1.0);
        let position = pt(0.0, 0.0, 0.0);
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

        // reflectivity for the default material
        assert_eq!(m.reflective, 0.0);

        // transparency and refractive_index for the default material
        assert_eq!(m.transparency, 0.0);
        assert_eq!(m.refractive_index, 1.0);
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
    fn lighting() {
        let m = Material::new();
        let position = pt(0.0, 0.0, 0.0);

        // lighting with the eye between the light and the surface
        // ambient, diffuse, and specular all at full strength
        let eyev = v(0.0, 0.0, -1.0);
        let normalv = v(0.0, 0.0, -1.0);
        let light = PointLight::new(pt(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = m.lighting(&sphere(), &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.9, 1.9, 1.9));

        // lighting with the surface in shadow
        let in_shadow = true;
        let result = m.lighting(&sphere(), &light, &position, &eyev, &normalv, in_shadow);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));

        // lighting with the eye between light and surface, eye offset 45 degrees
        // ambient and diffuse unchanged because the angle between them is unchanged
        // specular drops off to effectively zero
        let eyev = v(0.0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0);
        let result = m.lighting(&sphere(), &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.0, 1.0, 1.0));

        // lighting with eye opposite surface, light offset 45 degrees
        let eyev = v(0.0, 0.0, -1.0);
        let light = PointLight::new(pt(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = m.lighting(&sphere(), &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(0.7364, 0.7364, 0.7364));

        // lighting with eye in the path of the reflection vector
        // makes specular at full strength with ambient and diffuse same as last test
        let eyev = v(0.0, -2.0_f64.sqrt() / 2.0, -2.0_f64.sqrt() / 2.0);
        let light = PointLight::new(pt(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = m.lighting(&sphere(), &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.6364, 1.6364, 1.6364,));

        // light with the light behind the surface
        // Since the light doesn't illuminate the surface, the diffuse and specular
        // components go to zero.
        // The total intensity should be the same as the ambient component
        let eyev = v(0.0, 0.0, -1.0);
        let light = PointLight::new(pt(0.0, 0.0, 10.0), Color::new(1.0, 1.0, 1.0));
        let result = m.lighting(&sphere(), &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn lighting_with_pattern() {
        // lighting with a pattern applied
        let mut m = Material::new();
        m.pattern = Some(stripe_pattern(white(), black()));
        m.ambient = 1.0;
        m.diffuse = 0.0;
        m.specular = 0.0;
        let eyev = v(0.0, 0.0, -1.0);
        let normalv = v(0.0, 0.0, -1.0);
        let light = PointLight::new(pt(0.0, 0.0, -10.0), white());
        let c1 = m.lighting(
            &sphere(),
            &light,
            &pt(0.9, 0.0, 0.0),
            &eyev,
            &normalv,
            false,
        );
        let c2 = m.lighting(
            &sphere(),
            &light,
            &pt(1.1, 0.0, 0.0),
            &eyev,
            &normalv,
            false,
        );
        assert_eq!(c1, white());
        assert_eq!(c2, black());
    }

    #[test]
    fn intersection_precomputation() {
        // precomputing the state of an interesection
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let shape = sphere();
        let i = Intersection::new(4.0, shape);
        let comps = i.prepare_computations(&r, vec![i]);
        assert_eq!(comps.t, i.t);
        assert_eq!(comps.point, pt(0.0, 0.0, -1.0));
        assert_eq!(comps.eyev, v(0.0, 0.0, -1.0));
        assert_eq!(comps.normalv, v(0.0, 0.0, -1.0));

        // the hit, when an intersection occurs on the outside
        assert_eq!(comps.inside, false);

        // the hit, when an intersection occurs on the inside
        let r = Ray::new(pt(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0));
        let i = Intersection::new(1.0, shape);
        let comps = i.prepare_computations(&r, vec![i]);
        assert_eq!(comps.t, i.t);
        assert_eq!(comps.point, pt(0.0, 0.0, 1.0));
        assert_eq!(comps.eyev, v(0.0, 0.0, -1.0));
        assert_eq!(comps.inside, true);
        // normal would have been (0.0, 0.0, 1.0), but is inverted!
        assert_eq!(comps.normalv, v(0.0, 0.0, -1.0));

        // precomputing the reflection vector
        let shape = plane();
        let r = Ray::new(
            pt(0.0, 1.0, -1.0),
            v(0.0, -2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0),
        );
        let i = Intersection::new(2.0_f64.sqrt(), shape);
        let comps = i.prepare_computations(&r, vec![i]);
        assert_eq!(
            comps.reflectv,
            v(0.0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0)
        );
    }

    #[test]
    fn intersections_n1_n2() {
        let a = glass_spheret(Matrix4x4::scaling(2.0, 2.0, 2.0));
        let mut b = glass_spheret(Matrix4x4::translation(0.0, 0.0, -0.25));
        b.material.refractive_index = 2.0;
        let mut c = glass_spheret(Matrix4x4::translation(0.0, 0.0, 0.25));
        c.material.refractive_index = 2.5;
        let r = Ray::new(pt(0.0, 0.0, -4.0), v(0.0, 0.0, 1.0));
        let xs = vec![
            Intersection::new(2.0, a),
            Intersection::new(2.75, b),
            Intersection::new(3.25, c),
            Intersection::new(4.75, b),
            Intersection::new(5.25, c),
            Intersection::new(6.0, a),
        ];
        let i = xs[0];
        let comps = i.prepare_computations(&r, xs.clone());
        assert_eq!(comps.n1, 1.0);
        assert_eq!(comps.n2, 1.5);
        let i = xs[1];
        let comps = i.prepare_computations(&r, xs.clone());
        assert_eq!(comps.n1, 1.5);
        assert_eq!(comps.n2, 2.0);
        let i = xs[2];
        let comps = i.prepare_computations(&r, xs.clone());
        assert_eq!(comps.n1, 2.0);
        assert_eq!(comps.n2, 2.5);
        let i = xs[3];
        let comps = i.prepare_computations(&r, xs.clone());
        assert_eq!(comps.n1, 2.5);
        assert_eq!(comps.n2, 2.5);
        let i = xs[4];
        let comps = i.prepare_computations(&r, xs.clone());
        assert_eq!(comps.n1, 2.5);
        assert_eq!(comps.n2, 1.5);
        let i = xs[5];
        let comps = i.prepare_computations(&r, xs.clone());
        assert_eq!(comps.n1, 1.5);
        assert_eq!(comps.n2, 1.0);
    }

    #[test]
    fn comps_under_point() {
        // the under point is offset below the surface
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let shape = glass_spheret(Matrix4x4::translation(0.0, 0.0, 1.0));
        let i = Intersection::new(5.0, shape);
        let xs = vec![i];
        let comps = i.prepare_computations(&r, xs);
        assert_eq!(comps.under_point.z > EPSILON / 2.0, true);
        assert_eq!(comps.point.z < comps.under_point.z, true);
    }

    #[test]
    fn comps_schlick() {
        // the schlick approximation under total internal reflection
        let shape = glass_sphere();
        let r = Ray::new(pt(0.0, 0.0, 2.0_f64.sqrt() / 2.0), v(0.0, 1.0, 0.0));
        let xs = vec![
            Intersection::new(-2.0_f64.sqrt() / 2.0, shape),
            Intersection::new(2.0_f64.sqrt() / 2.0, shape),
        ];
        let i = xs[1];
        let comps = i.prepare_computations(&r, xs);
        let reflectance = schlick(&comps);
        // total internal feflection means all the light is reflected
        // and none is refracted. The fraction of the light that is
        // reflected must be 1 in this case. This is called reflectance.
        assert_eq!(reflectance, 1.0);

        // the schlick approximation with a perpendicular viewing angle
        let shape = glass_sphere();
        let r = Ray::new(pt(0.0, 0.0, 0.0), v(0.0, 1.0, 0.0));
        let xs = vec![
            Intersection::new(-1.0, shape),
            Intersection::new(1.0, shape),
        ];
        let i = xs[1];
        let comps = i.prepare_computations(&r, xs);
        let reflectance = schlick(&comps);
        assert_eq!(equal(reflectance, 0.04), true);

        // the schlick approximation with small angle and n2 > n1
        let shape = glass_sphere();
        let r = Ray::new(pt(0.0, 0.99, -2.0), v(0.0, 0.0, 1.0));
        let xs = vec![Intersection::new(1.8589, shape)];
        let i = xs[0];
        let comps = i.prepare_computations(&r, xs);
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
