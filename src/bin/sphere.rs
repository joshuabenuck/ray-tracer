use ray_tracer::{
    Canvas, Color, Intersections, Material, Matrix4x4, PointLight, Ray, Sphere, Tuple,
};

fn main() -> Result<(), std::io::Error> {
    let ray_origin = Tuple::point(0.0, 0.0, -5.0);
    let wall_z = 10.0;
    let wall_size = 7.0;
    let canvas_pixels = 100;
    let pixel_size = wall_size / canvas_pixels as f64;
    let half = wall_size / 2.0;
    let mut canvas = Canvas::new(canvas_pixels, canvas_pixels);
    // // shrink it along the y axis​
    // let transform = Matrix4x4::scaling(1.0, 0.5, 1.0);
    // //# shrink it along the x axis​
    // let transform = Matrix4x4::scaling(0.5, 1.0, 1.0);
    // // shrink it, and rotate it!​
    // use std::f64::consts::PI;
    // let transform = Matrix4x4::rotation_z(PI / 4.0) * Matrix4x4::scaling(0.5, 1.0, 1.0);
    // // shrink it, and skew it!​
    // let transform =
    //     Matrix4x4::shearing(1.0, 0.0, 0.0, 0.0, 0.0, 0.0) * Matrix4x4::scaling(0.5, 1.0, 1.0);
    let transform = Matrix4x4::identity();

    let mut material = Material::new();
    material.color = Color::new(1.0, 0.2, 1.0);
    let shape = Sphere::new(Some(transform), Some(material));
    let light_position = Tuple::point(-10.0, 10.0, -10.0);
    let light_color = Color::new(1.0, 1.0, 1.0);
    let light = PointLight::new(light_position, light_color);
    // for each row of pixels in the canvas
    for y in 0..canvas_pixels {
        // compute the world y coordinate (top = +half, bottom = -half)
        let world_y: f64 = half - pixel_size * y as f64;

        // for each pixel in the row
        for x in 0..canvas_pixels {
            // compute the world x coorinate (left = -half, right = +half)
            let world_x: f64 = -half + pixel_size * x as f64;

            // describe the point on the wall that the ray will target
            let position = Tuple::point(world_x, world_y, wall_z);

            let r = Ray::new(ray_origin, (position - ray_origin).normalize());
            let xs = shape.intersects(&r);

            if let Some(hit) = xs.hit() {
                let point = r.position(hit.t);
                let normal = hit.object.normal_at(point);
                let eye = -r.direction;
                let color = hit.object.material.lighting(&light, &point, &eye, &normal);
                canvas.write_pixel(x, y, color);
            }
        }
    }
    std::fs::write("./sphere.ppm", canvas.to_ppm())
}
