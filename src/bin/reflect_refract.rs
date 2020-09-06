use ray_tracer::*;

fn main() -> Result<(), std::io::Error> {
    let mut camera = Camera::new(400, 200, 1.152);
    camera.transform = view_transform(pt(-2.6, 1.5, -3.9), pt(-0.6, 1.0, -0.8), v(0.0, 1.0, 0.0));

    let light = PointLight::new(pt(-4.9, 4.9, -1.0), Color::new(1.0, 1.0, 1.0));

    let mut wall_material = Material::new();
    let mut wall_pattern =
        stripe_pattern(Color::new(0.45, 0.45, 0.45), Color::new(0.55, 0.55, 0.55));
    wall_pattern.transform = Matrix4x4::scaling(0.25, 0.25, 0.25) * Matrix4x4::rotation_y(1.5708);
    wall_material.pattern = Some(wall_pattern);
    wall_material.ambient = 0.0;
    wall_material.diffuse = 0.4;
    wall_material.specular = 0.0;
    wall_material.reflective = 0.3;

    let mut floor_material = Material::new();
    let floor_pattern =
        checkers_pattern(Color::new(0.35, 0.35, 0.35), Color::new(0.65, 0.65, 0.65));
    floor_material.pattern = Some(floor_pattern);
    floor_material.specular = 0.0;
    floor_material.reflective = 0.4;
    let floor = planetm(Matrix4x4::rotation_y(0.31415), floor_material);

    let mut ceiling_material = Material::new();
    ceiling_material.color = Color::new(0.8, 0.8, 0.8);
    ceiling_material.ambient = 0.3;
    ceiling_material.specular = 0.0;
    let ceiling = planetm(Matrix4x4::translation(0.0, 5.0, 0.0), ceiling_material);

    let west_wall = planetm(
        // Matrix4x4::rotation_y(1.5708),
        Matrix4x4::identity()
            .rotate_y(1.5708)
            .rotate_z(1.5708)
            .translate(-5.0, 0.0, 0.0),
        wall_material,
    );
    let east_wall = planetm(
        // Matrix4x4::rotation_y(1.5708),
        Matrix4x4::identity()
            .rotate_y(1.5708)
            .rotate_z(1.5708)
            .translate(5.0, 0.0, 0.0),
        wall_material,
    );
    let north_wall = planetm(
        Matrix4x4::identity()
            .rotate_x(1.5708)
            .translate(0.0, 0.0, 5.0),
        wall_material,
    );
    let south_wall = planetm(
        Matrix4x4::identity()
            .rotate_x(1.5708)
            .translate(0.0, 0.0, -5.0),
        wall_material,
    );

    // background
    let sphere1 = Sphere::new()
        .transform(id().scale(0.4, 0.4, 0.4).translate(4.6, 0.4, 1.0))
        .material(m().rgb(0.8, 0.5, 0.3).shininess(50.0));
    let sphere2 = Sphere::new()
        .transform(id().scale(0.3, 0.3, 0.3).translate(4.7, 0.3, 0.4))
        .material(m().rgb(0.9, 0.4, 0.5).shininess(50.0));
    let sphere3 = Sphere::new()
        .transform(id().scale(0.5, 0.5, 0.5).translate(-1.0, 0.5, 4.5))
        .material(m().rgb(0.4, 0.9, 0.6).shininess(50.0));
    let sphere4 = Sphere::new()
        .transform(id().scale(0.3, 0.3, 0.3).translate(-1.7, 0.3, 4.7))
        .material(m().rgb(0.4, 0.6, 0.9).shininess(50.0));

    // foreground
    let red = Sphere::new()
        .transform(id().translate(-0.6, 1.0, 0.6))
        .material(m().rgb(1.0, 0.3, 0.2).specular(0.4).shininess(5.0));
    let blue = Sphere::new()
        .transform(id().scale(0.7, 0.7, 0.7).translate(0.6, 0.7, -0.6))
        .material(
            m().rgb(0.0, 0.0, 0.2)
                .ambient(0.0)
                .diffuse(0.4)
                .specular(0.9)
                .shininess(300.0)
                .reflective(0.9)
                .transparency(0.9)
                .refractive_index(1.5),
        );
    let green = Sphere::new()
        .transform(id().scale(0.5, 0.5, 0.5).translate(-0.7, 0.5, -0.8))
        .material(
            m().rgb(0.0, 0.2, 0.0)
                .ambient(0.0)
                .diffuse(0.4)
                .specular(0.9)
                .shininess(300.0)
                .reflective(0.9)
                .transparency(0.9)
                .refractive_index(1.5),
        );

    let mut world = World::empty();
    world.lights.push(light);
    world.objects = vec![
        floor.into(),
        ceiling.into(),
        west_wall.into(),
        east_wall.into(),
        north_wall.into(),
        south_wall.into(),
        sphere1.into(),
        sphere2.into(),
        sphere3.into(),
        sphere4.into(),
        red.into(),
        blue.into(),
        green.into(),
    ];
    let image = camera.render(&world);
    std::fs::write("./reflect_refract.ppm", image.to_ppm())
}
