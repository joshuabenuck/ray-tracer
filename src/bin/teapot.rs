use anyhow::Result;
use ray_tracer::*;
use std::f64::consts::PI;

fn main() -> Result<()> {
    let light = PointLight::new(pt(10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
    let mut camera = Camera::new(100, 100, PI / 2.0);
    camera.transform = view_transform(pt(0.0, 4.0, -4.0), pt(0.0, 0.0, 0.0), v(0.0, 1.0, 0.0));
    let mut world = World::empty();
    world.lights.push(light);
    let teapot_contents = std::fs::read_to_string("./objs/teapot.obj")?;
    let parser = ObjParser::from_str(&teapot_contents)?;
    world.objects = vec![parser.into_group().shape()];
    let image = camera.render(&mut world);
    std::fs::write("./teapot.ppm", image.to_ppm())?;
    Ok(())
}
