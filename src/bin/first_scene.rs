use ray_tracer::{
    gradient_pattern, planem, pt, ring_pattern, stripe_pattern, v, view_transform, Camera, Color,
    Material, Matrix4x4, PointLight, Sphere, World,
};
use std::f64::consts::PI;

fn main() -> Result<(), std::io::Error> {
    let mut material = Material::new();
    material.color = Color::new(1.0, 0.9, 0.9);
    let pattern = Some(gradient_pattern(
        Color::new(1.0, 1.0, 1.0),
        Color::new(0.6, 0.6, 0.4),
    ));
    material.pattern = pattern;
    material.specular = 0.0;
    let floor = planem(material);

    // let transform = Matrix4x4::translation(0.0, 0.0, 5.0)
    //     * Matrix4x4::rotation_y(-PI / 4.0)
    //     * Matrix4x4::rotation_x(PI / 2.0)
    //     * Matrix4x4::scaling(10.0, 0.01, 10.0);
    // let left_wall = spheretm(transform, material);

    // let transform = Matrix4x4::translation(0.0, 0.0, 5.0)
    //     * Matrix4x4::rotation_y(PI / 4.0)
    //     * Matrix4x4::rotation_x(PI / 2.0)
    //     * Matrix4x4::scaling(10.0, 0.01, 10.0);
    // let right_wall = spheretm(transform, material);

    let transform = Matrix4x4::translation(-0.5, 1.0, 0.5);
    let mut material = Material::new();
    material.color = Color::new(0.1, 1.0, 0.5);
    let pattern = Some(ring_pattern(
        Color::new(1.0, 1.0, 1.0),
        Color::new(0.6, 0.6, 0.4),
    ));
    material.pattern = pattern;
    material.diffuse = 0.7;
    material.specular = 0.3;
    let middle = Sphere::new().transform(transform).material(material);

    let transform = Matrix4x4::translation(1.5, 0.50, -0.50) * Matrix4x4::scaling(0.5, 0.5, 0.5);
    let mut material = Material::new();
    material.color = Color::new(-0.5, 1.0, 0.1);
    let pattern = Some(gradient_pattern(
        Color::new(1.0, 0.0, 0.0),
        Color::new(0.0, 1.0, 0.0),
    ));
    material.pattern = pattern;
    material.diffuse = 0.7;
    material.specular = 0.3;
    let right = Sphere::new().transform(transform).material(material);

    let transform =
        Matrix4x4::translation(-1.5, 0.33, -0.75) * Matrix4x4::scaling(0.33, 0.33, 0.33);
    let mut material = Material::new();
    material.color = Color::new(1.0, 0.8, 0.1);
    let pattern = Some(stripe_pattern(
        Color::new(1.0, 1.0, 1.0),
        Color::new(0.6, 0.0, 0.4),
    ));
    material.pattern = pattern;
    material.diffuse = 0.7;
    material.specular = 0.3;
    let left = Sphere::new().transform(transform).material(material);

    let mut world = World::empty();
    world.lights.push(PointLight::new(
        pt(-10.0, 10.0, -10.0),
        Color::new(1.0, 1.0, 1.0),
    ));
    world.objects = vec![floor.into(), middle.into(), left.into(), right.into()];

    let mut camera = Camera::new(400, 200, PI / 3.0);
    camera.transform = view_transform(pt(0.0, 1.5, -5.0), pt(0.0, 1.0, 0.0), v(0.0, 1.0, 0.0));

    let canvas = camera.render(&world);
    std::fs::write("./first_scene.ppm", canvas.to_ppm())
}
