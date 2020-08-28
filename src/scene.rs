use crate::{
    pt, v, Canvas, Color, Comps, Intersection, Intersections, Material, Matrix4x4, PointLight, Ray,
    Sphere, Tuple,
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

pub fn view_transform(from: Tuple, to: Tuple, up: Tuple) -> Matrix4x4 {
    let forward = (to - from).normalize();
    let upn = up.normalize();
    let left = forward * upn;
    let true_up = left * forward;
    let orientation = Matrix4x4([
        [left.x, left.y, left.z, 0.0],
        [true_up.x, true_up.y, true_up.z, 0.0],
        [-forward.x, -forward.y, -forward.z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    orientation * Matrix4x4::translation(-from.x, -from.y, -from.z)
}

struct Camera {
    hsize: usize,
    vsize: usize,
    field_of_view: f64,
    transform: Matrix4x4,
    pixel_size: f64,
    half_width: f64,
    half_height: f64,
}

impl Camera {
    fn new(hsize: usize, vsize: usize, field_of_view: f64) -> Camera {
        let hsize = hsize as f64;
        let vsize = vsize as f64;
        let half_view = (field_of_view / 2.0).tan();
        let aspect = hsize / vsize;

        let half_width;
        let half_height;
        if aspect >= 1.0 {
            half_width = half_view;
            half_height = half_view / aspect;
        } else {
            half_width = half_view * aspect;
            half_height = half_view;
        }

        let pixel_size = (half_width * 2.0) / hsize;

        Camera {
            hsize: hsize as usize,
            vsize: vsize as usize,
            field_of_view,
            transform: Matrix4x4::identity(),
            half_width,
            half_height,
            pixel_size,
        }
    }

    pub fn ray_for_pixel(&self, px: usize, py: usize) -> Ray {
        let px = px as f64;
        let py = py as f64;
        // the offset from the edge of the canvas to the pixel's center
        let xoffset = (px + 0.5) * self.pixel_size;
        let yoffset = (py + 0.5) * self.pixel_size;

        // the untransformed coordinates of the pixel in world space
        // remember that the camera looks toward -z, so +x is to the *left*
        let world_x = self.half_width - xoffset;
        let world_y = self.half_height - yoffset;

        // using the camera matrix, transform the canvas point and the origin
        // and then compute the ray's direction vector
        let pixel = self.transform.inverse().unwrap() * pt(world_x, world_y, -1.0);
        let origin = self.transform.inverse().unwrap() * pt(0.0, 0.0, 0.0);
        let direction = (pixel - origin).normalize();

        Ray::new(origin, direction)
    }

    pub fn render(&self, world: &World) -> Canvas {
        let mut image = Canvas::new(self.hsize, self.vsize);

        for y in 0..self.vsize {
            for x in 0..self.hsize {
                let ray = self.ray_for_pixel(x, y);
                let color = world.color_at(&ray);
                image.write_pixel(x, y, color);
            }
        }
        image
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

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

    #[test]
    fn view_transform_() {
        // the transformation matrix for the default orientation
        let from = pt(0.0, 0.0, 0.0);
        let to = pt(0.0, 0.0, -1.0);
        let up = v(0.0, 1.0, 0.0);
        let t = view_transform(from, to, up);
        assert_eq!(t, Matrix4x4::identity());

        // a view transformation matrix looking in positive z direction
        let from = pt(0.0, 0.0, 0.0);
        let to = pt(0.0, 0.0, 1.0);
        let up = v(0.0, 1.0, 0.0);
        let t = view_transform(from, to, up);
        assert_eq!(t, Matrix4x4::scaling(-1.0, 1.0, -1.0));

        // the view transformation moves the world
        let from = pt(0.0, 0.0, 8.0);
        let to = pt(0.0, 0.0, 0.0);
        let up = v(0.0, 1.0, 0.0);
        let t = view_transform(from, to, up);
        assert_eq!(t, Matrix4x4::translation(0.0, 0.0, -8.0));

        // an arbitrary view transformation
        let from = pt(1.0, 3.0, 2.0);
        let to = pt(4.0, -2.0, 8.0);
        let up = v(1.0, 1.0, 0.0);
        let t = view_transform(from, to, up);
        assert_eq!(
            t,
            Matrix4x4([
                [-0.50709, 0.50709, 0.67612, -2.36643],
                [0.76772, 0.60609, 0.12122, -2.82843],
                [-0.35857, 0.59761, -0.71714, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ])
        );
    }

    const EPSILON: f64 = 0.00001;

    fn equal(a: f64, b: f64) -> bool {
        f64::abs(a - b) < EPSILON
    }

    #[test]
    fn camera_create() {
        // constructing a camera
        let hsize = 160;
        let vsize = 120;
        let field_of_view = PI / 2.0;
        let c = Camera::new(hsize, vsize, field_of_view);
        assert_eq!(c.hsize, hsize);
        assert_eq!(c.vsize, vsize);
        assert_eq!(c.field_of_view, PI / 2.0);
        assert_eq!(c.transform, Matrix4x4::identity());

        // the pixel size for a horizontal canvas
        let c = Camera::new(200, 125, PI / 2.0);
        assert_eq!(equal(c.pixel_size, 0.01), true);

        // the pixel size for a vertical canvas
        let c = Camera::new(125, 200, PI / 2.0);
        assert_eq!(equal(c.pixel_size, 0.01), true);
    }

    #[test]
    fn camera_ray_for_pixel() {
        // constructing a ray through the center of the canvas
        let mut c = Camera::new(201, 101, PI / 2.0);
        let r = c.ray_for_pixel(100, 50);
        assert_eq!(r.origin, pt(0.0, 0.0, 0.0));
        assert_eq!(r.direction, v(0.0, 0.0, -1.0));

        // constructing a ray through a corner of the canvas
        let r = c.ray_for_pixel(0, 0);
        assert_eq!(r.origin, pt(0.0, 0.0, 0.0));
        assert_eq!(r.direction, v(0.66519, 0.33259, -0.66851));

        // constructing a ray when the camera is transformed
        c.transform = Matrix4x4::rotation_y(PI / 4.0) * Matrix4x4::translation(0.0, -2.0, 5.0);
        let r = c.ray_for_pixel(100, 50);
        assert_eq!(r.origin, pt(0.0, 2.0, -5.0));
        assert_eq!(
            r.direction,
            v(2.0_f64.sqrt() / 2.0, 0.0, -2.0_f64.sqrt() / 2.0)
        );
    }

    #[test]
    fn camera_render() {
        // rendering a world with a camera
        let w = World::default();
        let mut c = Camera::new(11, 11, PI / 2.0);
        let from = pt(0.0, 0.0, -5.0);
        let to = pt(0.0, 0.0, 0.0);
        let up = v(0.0, 1.0, 0.0);
        c.transform = view_transform(from, to, up);
        let image = c.render(&w);
        assert_eq!(image.pixel_at(5, 5), Color::new(0.38066, 0.47583, 0.2855));
    }
}
