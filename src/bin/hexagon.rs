use anyhow::Result;
use ray_tracer::*;
use std::f64::consts::PI;

fn hexagon_corner() -> Box<dyn Shape> {
    Sphere::new()
        .transform(Matrix4x4::translation(0.0, 0.0, -1.0) * Matrix4x4::scaling(0.25, 0.25, 0.25))
        .shape()
}

fn hexagon_edge() -> Box<dyn Shape> {
    Cylinder::new(0.0, 1.0, false)
        .transform(
            id().scale(0.25, 1.0, 0.25)
                .rotate_z(-PI / 2.0)
                .rotate_y(-PI / 6.0)
                .translate(0.0, 0.0, -1.0),
        )
        .shape()
}

fn hexagon_side() -> Box<dyn Shape> {
    let mut side = Group::new();
    side.add_child(hexagon_corner());
    side.add_child(hexagon_edge());
    side.shape()
}

fn hexagon() -> Box<dyn Shape> {
    let mut hex = Group::new();

    for n in 0..6 {
        let mut side = hexagon_side();
        side.set_transform(Matrix4x4::rotation_y(n as f64 * PI / 3.0));
        hex.add_child(side);
    }

    hex.shape()
}

fn main() -> Result<()> {
    let light = PointLight::new(pt(10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
    let mut camera = Camera::new(400, 200, PI / 2.0);
    camera.transform = view_transform(pt(0.0, 0.0, -5.0), pt(0.0, 0.0, 0.0), v(0.0, 1.0, 0.0));
    let mut world = World::empty();
    world.lights.push(light);
    let mut hex = hexagon();
    hex.set_transform(Matrix4x4::rotation_x(-PI / 2.0));
    // materials on groups currently have no effect. :(
    let mut material = Material::new();
    material.pattern = Some(stripe_pattern(black(), white()));
    hex.set_material(material);
    world.objects = vec![hex];
    let image = camera.render(&mut world)?;
    std::fs::write("./hexagon.ppm", image.to_ppm())?;
    Ok(())
}
