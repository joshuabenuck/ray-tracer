use ray_tracer::{Canvas, Color, Tuple};

struct Projectile {
    position: Tuple,
    velocity: Tuple,
}

impl Projectile {
    fn new(position: Tuple, velocity: Tuple) -> Projectile {
        Projectile { position, velocity }
    }
}

struct Environment {
    gravity: Tuple,
    wind: Tuple,
}

impl Environment {
    fn new(gravity: Tuple, wind: Tuple) -> Environment {
        Environment { gravity, wind }
    }
}

fn tick(environment: &Environment, projectile: Projectile) -> Projectile {
    Projectile {
        position: projectile.position + projectile.velocity,
        velocity: projectile.velocity + environment.gravity + environment.wind,
    }
}

fn main() -> Result<(), std::io::Error> {
    // projectile starts one unit above the origin
    // velocity is normalized to 1 unit / tick
    let mut p = Projectile::new(
        Tuple::point(0.0, 1.0, 0.0),
        Tuple::vector(1.0, 1.8, 0.0).normalize() * 10.75,
    );

    let e = Environment::new(
        Tuple::vector(0.0, -0.1, 0.0),
        Tuple::vector(-0.01, 0.0, 0.0),
    );

    let mut count = 0;
    let mut c = Canvas::new(800, 600);
    let color = Color::new(0.5, 0.0, 0.5);
    while p.position.y > 0.0 {
        println!("{} - x: {}; y: {}", count, p.position.x, p.position.y);
        p = tick(&e, p);
        let pos = p.position;
        c.write_pixel(pos.x.round() as usize, 600 - pos.y.round() as usize, color);
        count += 1;
    }

    std::fs::write("output.ppm", c.to_ppm())
}
