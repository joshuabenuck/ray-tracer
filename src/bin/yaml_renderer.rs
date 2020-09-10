use anyhow::{anyhow, Result};
use ray_tracer::*;
use std::collections::HashMap;
use yaml_rust::{Yaml, YamlLoader};

fn to_v(obj: &Yaml) -> Result<Tuple> {
    Ok(v(to_f64(&obj[0])?, to_f64(&obj[1])?, to_f64(&obj[2])?))
}

fn to_pt(obj: &Yaml) -> Result<Tuple> {
    Ok(pt(to_f64(&obj[0])?, to_f64(&obj[1])?, to_f64(&obj[2])?))
}

fn to_f64(obj: &Yaml) -> Result<f64> {
    match obj {
        Yaml::Real(_) => Ok(obj.as_f64().unwrap()),
        Yaml::Integer(_) => Ok(obj.as_i64().unwrap() as f64),
        _ => Err(anyhow!("Unable to read f64")),
    }
}

fn to_color(obj: &Yaml) -> Result<Color> {
    Ok(Color::new(
        to_f64(&obj[0])?,
        to_f64(&obj[1])?,
        to_f64(&obj[2])?,
    ))
}

fn apply_transfor(base: Matrix4x4, params: &Yaml) {}

fn to_transform(transforms: &Vec<Yaml>) -> Result<Matrix4x4> {
    let mut transform = id();
    for params in transforms {
        let transform_type = params[0].as_str().unwrap();
        match transform_type {
            "rotate-x" => {
                transform = transform.rotate_x(to_f64(&params[1])?);
            }
            "rotate-y" => {
                transform = transform.rotate_y(to_f64(&params[1])?);
            }
            "rotate-z" => {
                transform = transform.rotate_z(to_f64(&params[1])?);
            }
            "scale" => {
                transform = transform.scale(
                    to_f64(&params[1])?,
                    to_f64(&params[2])?,
                    to_f64(&params[3])?,
                );
            }
            "translate" => {
                transform = transform.translate(
                    to_f64(&params[1])?,
                    to_f64(&params[2])?,
                    to_f64(&params[3])?,
                );
            }
            _ => {
                panic!("Unrecognized plane transform: {}", transform_type);
            }
        }
    }
    Ok(transform)
}

fn to_material(obj: &Yaml) -> Result<Material> {
    let mut material = Material::new();
    let props = &obj["pattern"];
    if props != &Yaml::BadValue {
        let r#type = props["type"].as_str().unwrap();
        match r#type {
            "checkers" => {
                if let Yaml::Array(colors) = &props["colors"] {
                    let a = to_color(&colors[0])?;
                    let b = to_color(&colors[1])?;
                    let pattern = checkers_pattern(a, b);
                    material.pattern = Some(pattern);
                }
            }
            "stripes" => {
                if let Yaml::Array(colors) = &props["colors"] {
                    let a = to_color(&colors[0])?;
                    let b = to_color(&colors[1])?;
                    let mut pattern = stripe_pattern(a, b);
                    if let Yaml::Array(transforms) = &props["transform"] {
                        pattern.transform = to_transform(transforms)?;
                    }
                    material.pattern = Some(pattern);
                }
            }
            _ => panic!("Unexpected pattern type: {}", r#type),
        }
    }
    if let Ok(color) = to_color(&obj["color"]) {
        material.color = color;
    }
    if let Ok(ambient) = to_f64(&obj["ambient"]) {
        material.ambient = ambient;
    }
    if let Ok(diffuse) = to_f64(&obj["diffuse"]) {
        material.diffuse = diffuse;
    }
    if let Ok(specular) = to_f64(&obj["specular"]) {
        material.specular = specular;
    }
    if let Ok(reflective) = to_f64(&obj["reflective"]) {
        material.reflective = reflective;
    }
    if let Ok(shininess) = to_f64(&obj["shininess"]) {
        material.shininess = shininess;
    }
    if let Ok(transparency) = to_f64(&obj["transparency"]) {
        material.transparency = transparency;
    }
    if let Ok(refractive_index) = to_f64(&obj["refractive-index"]) {
        material.refractive_index = refractive_index;
    }
    Ok(material)
}

fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("no yaml file provided");
    let contents = std::fs::read_to_string(&path)?;
    let scene = YamlLoader::load_from_str(&contents)?;
    let mut world = World::empty();
    let mut camera: Option<Camera> = None;
    let mut materials: HashMap<String, Material> = HashMap::new();
    let mut transforms: HashMap<String, Matrix4x4> = HashMap::new();
    for obj in scene[0].as_vec().unwrap() {
        if let Yaml::String(r#type) = &obj["add"] {
            match r#type.as_str() {
                "camera" => {
                    //   width: 400
                    //   height: 200
                    //   field-of-view: 1.152
                    //   from: [-2.6, 1.5, -3.9]
                    //   to: [-0.6, 1, -0.8]
                    //   up: [0, 1, 0]
                    // println!("{:#?}", obj);
                    let width = to_f64(&obj["width"])? as usize;
                    let height = to_f64(&obj["height"])? as usize;
                    let fov = to_f64(&obj["field-of-view"])?;
                    let from = to_v(&obj["from"])?;
                    let to = to_v(&obj["to"])?;
                    let up = to_v(&obj["up"])?;
                    let mut c = Camera::new(width, height, fov);
                    c.transform(view_transform(from, to, up));
                    camera = Some(c);
                }
                "light" => {
                    //   at: [-4.9, 4.9, -1]
                    //   intensity: [1, 1, 1]
                    let at = to_pt(&obj["at"])?;
                    let intensity = to_color(&obj["intensity"])?;
                    let light = PointLight::new(at, intensity);
                    world.lights.push(light);
                }
                "plane" => {
                    // - add: plane
                    //   transform:
                    //     - [rotate-y, 0.31415]
                    let mut plane = Plane::new();
                    match &obj["transform"] {
                        Yaml::Array(transforms) => plane.set_transform(to_transform(transforms)?),
                        Yaml::String(name) => plane.set_transform(transforms[name].clone()),
                        _ => {}
                    }
                    //   material:
                    //     pattern:
                    //       type: checkers
                    //       colors:
                    //         - [0.35, 0.35, 0.35]
                    //         - [0.65, 0.65, 0.65]
                    //     specular: 0
                    //     reflective: 0.4
                    let props = &obj["material"];
                    match props {
                        Yaml::Hash(_) => plane.set_material(to_material(&props)?),
                        Yaml::String(name) => plane.set_material(materials[name].clone()),
                        _ => {}
                    }
                    world.objects.push(plane.shape());
                }
                "sphere" => {
                    // - add: sphere
                    //   transform:
                    //     - [scale, 0.4, 0.4, 0.4]
                    //     - [translate, 4.6, 0.4, 1]
                    //   material:
                    //     color: [0.8, 0.5, 0.3]
                    //     shininess: 50
                    let mut sphere = Sphere::new();
                    if let Yaml::Array(transforms) = &obj["transform"] {
                        sphere.set_transform(to_transform(transforms)?);
                    }
                    let props = &obj["material"];
                    if props != &Yaml::BadValue {
                        sphere.set_material(to_material(&props)?);
                    }
                    world.objects.push(sphere.shape());
                }
                _ => {
                    println!("Uknown object type: {}", r#type);
                }
            }
        }
        if let Yaml::String(name) = &obj["define"] {
            if name.contains("-material") {
                materials.insert(name.clone(), to_material(&obj["value"])?);
            } else if name.contains("-transform") {
                if let Yaml::Array(value) = &obj["value"] {
                    transforms.insert(name.clone(), to_transform(&value)?);
                } else {
                    panic!("Unexpected transform define structure!");
                }
            } else if name.contains("-object") {
                if let Yaml::Array(value) = &obj["value"] {
                    let base = value[0].as_str().unwrap();
                    let mut transform = transforms[base].clone();
                }
            } else {
                panic!("Unexpected define type: {}!", name);
            }
        }
    }

    match camera {
        Some(camera) => {
            let image = camera.render(&mut world);
            let path = std::path::PathBuf::from(path);
            let image_base = path.file_stem().unwrap().to_str().unwrap();
            let image_path = format!("./{}.ppm", image_base);
            std::fs::write(image_path, image.to_ppm())?;
        }
        None => {
            panic!("No camera set!");
        }
    }
    Ok(())
}
