use crate::{
    pt, v, Color, Comps, Intersection, Intersections, Material, Matrix4x4, PointLight, Ray, Sphere,
    Tuple,
};

pub struct World {
    pub objects: Vec<Sphere>,
    pub lights: Vec<PointLight>,
}

impl World {
    pub fn empty() -> World {
        World {
            objects: Vec::new(),
            lights: Vec::new(),
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let mut xs = Vec::new();
        for obj in &self.objects {
            xs.append(&mut obj.intersects(ray));
        }
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        xs
    }

    pub fn shade_hit(&self, comps: &Comps) -> Color {
        let mut cs = self.lights.iter().map(|l| {
            comps
                .object
                .material
                .lighting(&l, &comps.point, &comps.eyev, &comps.normalv)
        });
        let mut color = cs.next().unwrap();
        for c in cs {
            color = color + c;
        }
        color
    }

    pub fn color_at(&self, ray: &Ray) -> Color {
        let intersections = self.intersect(&ray);
        let hit = intersections.hit();
        if let Some(i) = hit {
            let comps = i.prepare_computations(&ray);
            return self.shade_hit(&comps);
        }

        Color::new(0.0, 0.0, 0.0)
    }
}

impl Default for World {
    fn default() -> World {
        let light = PointLight::new(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut material = Material::new();
        material.color = Color::new(0.8, 1.0, 0.6);
        material.diffuse = 0.7;
        material.specular = 0.2;
        // outer
        let s1 = Sphere::new(None, Some(material));
        // inner
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

    #[test]
    fn intersection_shading() {
        // shading an intersection
        let mut w = World::default();
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let shape = w.objects[0];
        let i = Intersection::new(4.0, shape);
        let comps = i.prepare_computations(&r);
        let c = w.shade_hit(&comps);
        assert_eq!(c, Color::new(0.38066, 0.47583, 0.2855));

        // shading an intersection from the inside
        w.lights = vec![PointLight::new(
            pt(0.0, 0.25, 0.0),
            Color::new(1.0, 1.0, 1.0),
        )];
        let r = Ray::new(pt(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0));
        let shape = w.objects[1];
        let i = Intersection::new(0.5, shape);
        let comps = i.prepare_computations(&r);
        let c = w.shade_hit(&comps);
        assert_eq!(c, Color::new(0.90498, 0.90498, 0.90498));
    }

    #[test]
    fn world_color_at() {
        // the color when a ray misses
        let w = World::default();
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 1.0, 0.0));
        let c = w.color_at(&r);
        assert_eq!(c, Color::new(0.0, 0.0, 0.0));

        // the color when a ray hits
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let c = w.color_at(&r);
        assert_eq!(c, Color::new(0.38066, 0.47583, 0.2855));

        // the color with an intersection behind the ray
        // inside the outer sphere, but outside the inner sphere
        let mut w = World::default();
        let outer = &mut w.objects[0];
        outer.material.ambient = 1.0;
        let inner = &mut w.objects[1];
        inner.material.ambient = 1.0;
        drop(inner);
        let r = Ray::new(pt(0.0, 0.0, 0.75), v(0.0, 0.0, -1.0));
        let c = w.color_at(&r);
        assert_eq!(c, w.objects[1].material.color);
    }
}
