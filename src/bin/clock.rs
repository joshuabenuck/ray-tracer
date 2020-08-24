use ray_tracer::{Canvas, Color, Matrix4x4, Tuple};
use std::f64::consts::PI;

fn main() -> Result<(), std::io::Error> {
    let mut c = Canvas::new(100, 100);
    let radius = 100.0 * 3.0 / 8.0;
    println!("radius: {}", radius);
    let p = Tuple::point(0.0, 1.0, 0.0);
    let mut radians = 0.0;
    c.write_pixel(50, 50, Color::new(0.5, 0.0, 0.5));
    for _i in 0..12 {
        let mut h = Matrix4x4::rotation_z(radians) * p;
        println!("x: {}; y: {}", h.x, h.y);
        h.x *= radius;
        h.y *= radius;
        h.x += 50.0;
        h.y += 50.0;
        c.write_pixel(
            h.x.round() as usize,
            h.y.round() as usize,
            Color::new(0.5, 0.0, 0.5),
        );
        radians += PI / 6.0;
    }
    std::fs::write("./clock.ppm", c.to_ppm())
}
