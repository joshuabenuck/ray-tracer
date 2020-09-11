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

fn apply_transform(transform: Matrix4x4, params: &Yaml) -> Result<Matrix4x4> {
    let mut transform = transform;
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
    Ok(transform)
}

fn to_transform(transforms: &Vec<Yaml>, defines: &HashMap<String, Matrix4x4>) -> Result<Matrix4x4> {
    let mut transform = id();
    for params in transforms {
        match &params {
            Yaml::Array(_) => transform = apply_transform(transform, params)?,
            Yaml::String(name) => transform = defines[name],
            _ => unreachable!(),
        }
    }
    Ok(transform)
}

fn apply_material(
    material: Material,
    obj: &Yaml,
    transforms: &HashMap<String, Matrix4x4>,
) -> Result<Material> {
    let mut material = material;
    let props = &obj["pattern"];
    if props != &Yaml::BadValue {
        let r#type = props["type"].as_str().unwrap();
        match r#type {
            "checkers" => {
                if let Yaml::Array(colors) = &props["colors"] {
                    let a = to_color(&colors[0])?;
                    let b = to_color(&colors[1])?;
                    let mut pattern = checkers_pattern(a, b);
                    if let Yaml::Array(ts) = &props["transform"] {
                        pattern.transform = to_transform(ts, &transforms)?;
                    }
                    material.pattern = Some(pattern);
                }
            }
            "stripes" => {
                if let Yaml::Array(colors) = &props["colors"] {
                    let a = to_color(&colors[0])?;
                    let b = to_color(&colors[1])?;
                    let mut pattern = stripe_pattern(a, b);
                    if let Yaml::Array(ts) = &props["transform"] {
                        pattern.transform = to_transform(ts, &transforms)?;
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

fn to_material(obj: &Yaml, transforms: &HashMap<String, Matrix4x4>) -> Result<Material> {
    apply_material(Material::new(), obj, transforms)
}

fn apply_shape(
    shape: &mut dyn Shape,
    obj: &Yaml,
    materials: &HashMap<String, Material>,
    transforms: &HashMap<String, Matrix4x4>,
) -> Result<()> {
    match &obj["transform"] {
        Yaml::Array(ts) => shape.set_transform(to_transform(ts, &transforms)?),
        Yaml::String(name) => shape.set_transform(transforms[name].clone()),
        _ => {}
    }
    let props = &obj["material"];
    match props {
        Yaml::Hash(_) => shape.set_material(to_material(&props, &transforms)?),
        Yaml::String(name) => shape.set_material(materials[name].clone()),
        _ => {}
    }
    if let Some(shadow) = &obj["shadow"].as_bool() {
        shape.set_shadow(*shadow);
    }
    Ok(())
}

fn add_shape(
    obj: &Yaml,
    materials: &HashMap<String, Material>,
    transforms: &HashMap<String, Matrix4x4>,
    shapes: &HashMap<String, Yaml>,
) -> Result<Box<dyn Shape>> {
    let r#type = obj["add"].as_str().unwrap();
    println!("Adding {}", r#type);
    let shape = match r#type {
        "cube" => {
            let mut cube = Cube::new().shape();
            apply_shape(&mut *cube, &obj, &materials, &transforms)?;
            cube
        }
        "plane" => {
            let mut plane = Plane::new().shape();
            apply_shape(&mut *plane, &obj, &materials, &transforms)?;
            plane
        }
        "sphere" => {
            let mut sphere = Sphere::new().shape();
            apply_shape(&mut *sphere, &obj, &materials, &transforms)?;
            sphere
        }
        "cylinder" => {
            let min = to_f64(&obj["min"])?;
            let max = to_f64(&obj["max"])?;
            let closed = obj["closed"].as_bool().unwrap();
            let mut cylinder = Cylinder::new(min, max, closed).shape();
            apply_shape(&mut *cylinder, &obj, &materials, &transforms)?;
            cylinder
        }
        "cone" => {
            let min = to_f64(&obj["min"])?;
            let max = to_f64(&obj["max"])?;
            let closed = obj["closed"].as_bool().unwrap();
            let mut cone = Cone::new(min, max, closed).shape();
            apply_shape(&mut *cone, &obj, &materials, &transforms)?;
            cone
        }
        "group" => {
            let mut group = Group::new();
            for child_obj in obj["children"].as_vec().unwrap() {
                let child = add_shape(child_obj, materials, transforms, shapes)?;
                group.add_child(child);
            }
            let mut group = group.shape();
            apply_shape(&mut *group, &obj, &materials, &transforms)?;
            group
        }
        name => {
            if let Some(def) = shapes.get(name) {
                let mut shape = add_shape(&def.clone(), materials, transforms, shapes)?;
                apply_shape(&mut *shape, &obj, &materials, &transforms)?;
                shape
            } else {
                panic!("Unknown shape: {}", name);
            }
        }
    };
    Ok(shape)
}

fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("no yaml file provided");
    let contents = std::fs::read_to_string(&path)?;
    let scene = YamlLoader::load_from_str(&contents)?;
    let mut world = World::empty();
    let mut camera: Option<Camera> = None;
    let mut materials: HashMap<String, Material> = HashMap::new();
    let mut transforms: HashMap<String, Matrix4x4> = HashMap::new();
    let mut shapes: HashMap<String, Yaml> = HashMap::new();
    for obj in scene[0].as_vec().unwrap() {
        if let Yaml::String(r#type) = &obj["add"] {
            println!("Adding {}", r#type);
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
                    let at = to_pt(&obj["at"])?;
                    let intensity = to_color(&obj["intensity"])?;
                    let light = PointLight::new(at, intensity);
                    world.lights.push(light);
                }
                "group" | "cube" | "plane" | "sphere" | "cylinder" | "cone" | "wacky" | "cap"
                | "leg" => {
                    world
                        .objects
                        .push(add_shape(&obj, &materials, &transforms, &shapes)?);
                }
                _ => {
                    println!("Uknown object type: {}", r#type);
                }
            }
        }
        if let Yaml::String(name) = &obj["define"] {
            println!("Defining {}", name);
            if name.contains("leg") || name.contains("cap") || name.contains("wacky") {
                let value = &obj["value"];
                shapes.insert(name.clone(), value.clone());
            } else if name.contains("-material") {
                let mut base = Material::new();
                if let Some(extend) = obj["extend"].as_str() {
                    base = materials[extend].clone();
                }
                materials.insert(
                    name.clone(),
                    apply_material(base, &obj["value"], &transforms)?,
                );
            } else if name.contains("-transform") {
                if let Yaml::Array(value) = &obj["value"] {
                    transforms.insert(name.clone(), to_transform(&value, &transforms)?);
                } else {
                    panic!("Unexpected transform define structure!");
                }
            } else if name.contains("-object") {
                if let Yaml::Array(value) = &obj["value"] {
                    let base = value[0].as_str().unwrap();
                    let mut transform = transforms[base].clone();
                    transform = apply_transform(transform, &value[1])?;
                    transforms.insert(name.clone(), transform);
                }
            } else {
                panic!("Unexpected define type: {}!", name);
            }
        }
    }

    match camera {
        Some(camera) => {
            println!("Rendering");
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
