use crate::{
    Color, Intersection, Intersections, Material, Matrix4x4, PointLight, Ray, Sphere, Tuple,
};

pub struct World {
    objects: Vec<Sphere>,
    lights: Vec<PointLight>,
}

impl World {
    fn empty() -> World {
        World {
            objects: Vec::new(),
            lights: Vec::new(),
        }
    }

    fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let mut xs = Vec::new();
        for obj in &self.objects {
            xs.append(&mut obj.intersects(ray));
        }
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        xs
    }
}

impl Default for World {
    fn default() -> World {
        let light = PointLight::new(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut material = Material::new();
        material.color = Color::new(0.8, 1.0, 0.6);
        material.diffuse = 0.7;
        material.specular = 0.2;
        let s1 = Sphere::new(None, Some(material));
        let s2 = Sphere::new(Some(Matrix4x4::scaling(0.5, 0.5, 0.5)), None);
        World {
            objects: vec![s1, s2],
            lights: vec![light],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn world_create() {
        // creating a world
        let w = World::empty();
        assert_eq!(w.objects.len(), 0);
        assert_eq!(w.lights.len(), 0);

        // the default world
        let light = PointLight::new(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut material = Material::new();
        material.color = Color::new(0.8, 1.0, 0.6);
        material.diffuse = 0.7;
        material.specular = 0.2;
        let s1 = Sphere::new(None, Some(material));
        let s2 = Sphere::new(Some(Matrix4x4::scaling(0.5, 0.5, 0.5)), None);
        let w = World::default();
        assert_eq!(w.lights[0], light);
        assert_eq!(w.objects.contains(&s1), true);
        assert_eq!(w.objects.contains(&s2), true);
    }

    #[test]
    fn world_ray_intersect() {
        // intersect a world with a ray
        let w = World::default();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = w.intersect(&r);
        assert_eq!(xs.len(), 4);
        assert_eq!(xs[0].t, 4.0);
        assert_eq!(xs[1].t, 4.5);
        assert_eq!(xs[2].t, 5.5);
        assert_eq!(xs[3].t, 6.0);
    }
}
