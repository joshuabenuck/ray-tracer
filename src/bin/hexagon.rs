// use ray_tracer::*;
// use std::cell::RefCell;
// use std::f64::consts::PI;
// use std::rc::Rc;

// fn hexagon_corner() -> Sphere {
//     spheret(Matrix4x4::translation(0.0, 0.0, -1.0) * Matrix4x4::scaling(0.25, 0.25, 0.25))
// }

// fn hexagon_edge() -> Rc<RefCell<Shape>> {
//     cylindert(
//         id().scale(0.25, 1.0, 0.25)
//             .rotate_z(-PI / 2.0)
//             .rotate_y(-PI / 6.0)
//             .translate(0.0, 0.0, -1.0),
//         0.0,
//         1.0,
//         false,
//     )
// }

// fn hexagon_side() -> Rc<RefCell<Shape>> {
//     let side = group();
//     add_child(&side, hexagon_corner());
//     add_child(&side, hexagon_edge());
//     side
// }

// fn hexagon() -> Rc<RefCell<Shape>> {
//     let hex = group();

//     for n in 0..6 {
//         let side = hexagon_side();
//         side.borrow_mut().transform = Matrix4x4::rotation_y(n as f64 * PI / 3.0);
//         add_child(&hex, side);
//     }

//     hex
// }

fn main() { //-> Result<(), std::io::Error> {
            // let light = PointLight::new(pt(10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
            // let mut camera = Camera::new(400, 200, PI / 2.0);
            // camera.transform = view_transform(pt(0.0, 0.0, -5.0), pt(0.0, 0.0, 0.0), v(0.0, 1.0, 0.0));
            // let mut world = World::empty();
            // world.lights.push(light);
            // let hex = hexagon();
            // hex.borrow_mut().transform = Matrix4x4::rotation_x(-PI / 2.0);
            // // materials on groups currently have no effect. :(
            // let mut material = Material::new();
            // material.pattern = Some(stripe_pattern(black(), white()));
            // hex.borrow_mut().material = material;
            // world.objects = vec![hex];
            // let image = camera.render(&world);
            // std::fs::write("./hexagon.ppm", image.to_ppm())
}
