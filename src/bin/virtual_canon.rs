use ray_tracer::Tuple;

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

fn main() {
    // projectile starts one unit above the origin
    // velocity is normalized to 1 unit / tick
    let mut p = Projectile::new(
        Tuple::point(0.0, 1.0, 0.0),
        Tuple::vector(1.0, 1.0, 0.0).normalize(),
    );

    let e = Environment::new(
        Tuple::vector(0.0, -0.1, 0.0),
        Tuple::vector(-0.01, 0.0, 0.0),
    );

    let mut count = 0;
    while p.position.y > 0.0 {
        println!("{} - x: {}; y: {}", count, p.position.x, p.position.y);
        p = tick(&e, p);
        count += 1;
    }
}
