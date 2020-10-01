use ray_tracer::*;
use std::f64::consts::PI;

fn main() -> Result<(), std::io::Error> {
    let mut camera = Camera::new(400, 200, 1.152);
    camera.transform = view_transform(pt(-2.6, 2.0, -5.9), pt(0.0, 0.0, 0.0), v(0.0, 1.0, 0.0));

    let mut wall_material = Material::new();
    let wall_pattern = checkers_pattern(Color::new(0.45, 0.45, 0.45), Color::new(0.65, 0.65, 0.65));
    // wall_pattern.transform = Matrix4x4::scaling(0.25, 0.25, 0.25) * Matrix4x4::rotation_y(1.5708);
    wall_material.pattern = Some(wall_pattern);
    wall_material.ambient = 0.7;
    // wall_material.diffuse = 0.4;
    wall_material.specular = 0.0;
    // wall_material.reflective = 0.3;

    let mut floor_material = Material::new();
    let floor_pattern =
        checkers_pattern(Color::new(0.45, 0.45, 0.45), Color::new(0.65, 0.65, 0.65));
    floor_material.pattern = Some(floor_pattern);
    floor_material.ambient = 0.7;
    floor_material.specular = 0.0;
    // floor_material.reflective = 0.4;
    let floor = Plane::new()
        .transform(id().rotate_y(0.31415).translate(0.0, -3.0, 0.0))
        .material(floor_material);

    let mut ceiling_material = Material::new();
    let ceiling_pattern =
        checkers_pattern(Color::new(0.45, 0.45, 0.45), Color::new(0.65, 0.65, 0.65));
    ceiling_material.pattern = Some(ceiling_pattern);
    ceiling_material.ambient = 1.0;
    ceiling_material.specular = 0.0;
    let ceiling = Plane::new()
        .transform(Matrix4x4::translation(0.0, 5.0, 0.0))
        .material(ceiling_material);

    // let west_wall = Plane::new()
    //     .transform(
    //         // Matrix4x4::rotation_y(1.5708),
    //         Matrix4x4::identity()
    //             .rotate_y(1.5708)
    //             .rotate_z(1.5708)
    //             .translate(-5.0, 0.0, 0.0),
    //     )
    //     .material(wall_material);
    // let east_wall = Plane::new()
    //     .transform(
    //         // Matrix4x4::rotation_y(1.5708),
    //         Matrix4x4::identity()
    //             .rotate_y(1.5708)
    //             .rotate_z(1.5708)
    //             .translate(5.0, 0.0, 0.0),
    //     )
    //     .material(wall_material);
    let north_wall = Plane::new()
        .transform(
            Matrix4x4::identity()
                .rotate_x(1.5708)
                .translate(0.0, 0.0, 8.0),
        )
        .material(wall_material);
    // let south_wall = Plane::new()
    //     .transform(
    //         Matrix4x4::identity()
    //             .rotate_x(1.5708)
    //             .translate(0.0, 0.0, -5.0),
    //     )
    //     .material(wall_material);

    // color: [0, 0, 0.6]
    // diffuse: 0.1
    // specular: 0.9
    // shininess: 300
    // reflective: 0.9

    let cube_material = Material::new()
        .rgb(0.0, 0.0, 0.0)
        .diffuse(0.1)
        .specular(0.9)
        .shininess(300.0)
        // .ambient(0.1)
        // .transparency(1.0)
        // .refractive_index(2.417)
        .reflective(0.9);
    let cube = Cube::new().material(cube_material.clone());
    let cyl_z = Cylinder::new(-4.0, 4.0, true)
        .transform(id().scale(0.5, 0.5, 0.5).rotate_x(PI / 2.0))
        .material(Material::new().rgb(0.0, 1.0, 0.0));
    let cyl_x = Cylinder::new(-4.0, 4.0, true)
        .transform(id().scale(0.5, 0.5, 0.5).rotate_z(PI / 2.0))
        .material(Material::new().rgb(0.0, 1.0, 0.0));
    let cyl_y = Cylinder::new(-2.0, 2.0, false)
        .transform(Matrix4x4::scaling(0.5, 1.0, 0.5))
        .material(Material::new().rgb(1.0, 0.0, 0.0));
    let sphere = Sphere::new()
        .transform(id().scale(1.00, 1.00, 1.00))
        .material(cube_material);
    // .material(Material::new().rgb(0.0, 0.0, 0.0));
    // .transform(id().scale(0.05, 0.05, 1.00).rotate_x(PI / 02.0));
    let csg = Csg::difference(cube.shape(), cyl_y.shape()).shape();
    let csg = Csg::difference(csg, cyl_z.shape()).shape();
    let csg = Csg::difference(csg, cyl_x.shape()).shape();
    let csg = Csg::intersection(csg, sphere.shape()).shape();
    // csg.set_transform(id().rotate_y(PI / 3.0));
    let mut world = World::empty();
    let light = PointLight::new(pt(-4.9, 4.9, -1.0), Color::new(1.0, 1.0, 1.0));
    world.lights.push(light);
    // let light = PointLight::new(pt(0.0, 2.0, 0.0), Color::new(0.1, 0.1, 0.1));
    // world.lights.push(light);
    world.objects = vec![
        floor.into(),
        ceiling.into(),
        // west_wall.into(),
        // east_wall.into(),
        north_wall.into(),
        // south_wall.into(),
        csg.into(),
        // holder.shape(),
    ];
    let image = camera.render(&mut world);
    std::fs::write("./csg.ppm", image.to_ppm())
}
