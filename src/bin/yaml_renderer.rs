use anyhow::{anyhow, Result};
use ray_tracer::*;
use std::collections::HashMap;
use yaml_rust::{Yaml, YamlLoader};

struct Definitions {
    transforms: HashMap<String, Matrix4x4>,
    materials: HashMap<String, Material>,
    shapes: HashMap<String, Yaml>,
}

impl Definitions {
    fn new() -> Definitions {
        Definitions {
            transforms: HashMap::new(),
            materials: HashMap::new(),
            shapes: HashMap::new(),
        }
    }

    fn define(&mut self, obj: &Yaml) -> Result<()> {
        let name = obj["define"].as_str().unwrap();
        println!("Defining {}", name);
        if name.contains("leg") || name.contains("cap") || name.contains("wacky") {
            let value = &obj["value"];
            self.shapes.insert(name.to_owned(), value.clone());
        } else if name.contains("-material") {
            let mut base = Material::new();
            if let Some(extend) = obj["extend"].as_str() {
                base = self.materials[extend].clone();
            }
            self.materials
                .insert(name.to_owned(), apply_material(base, &obj["value"], &self)?);
        } else if name.contains("-transform") {
            self.transforms
                .insert(name.to_owned(), obj["value"].as_transform(&self)?);
        } else if name.contains("-object") {
            if let Yaml::Array(value) = &obj["value"] {
                let base = value[0].as_str().unwrap();
                let mut transform = self.transforms[base].clone();
                transform = value[1].populate_transform(transform)?;
                self.transforms.insert(name.to_owned(), transform);
            }
        } else {
            panic!("Unexpected define type: {}!", name);
        }
        Ok(())
    }
}

trait YamlExt {
    fn as_float(&self) -> Result<f64>;
    fn as_v(&self) -> Result<Tuple>;
    fn as_pt(&self) -> Result<Tuple>;
    fn as_color(&self) -> Result<Color>;
    fn as_camera(&self) -> Result<Camera>;
    fn as_light(&self) -> Result<PointLight>;
    fn as_transform(&self, defs: &Definitions) -> Result<Matrix4x4>;
    fn populate_transform(&self, transform: Matrix4x4) -> Result<Matrix4x4>;
}

impl YamlExt for Yaml {
    fn as_float(&self) -> Result<f64> {
        match &self {
            Yaml::Real(_) => Ok(self.as_f64().unwrap()),
            Yaml::Integer(_) => Ok(self.as_i64().unwrap() as f64),
            _ => Err(anyhow!("Unable to read f64")),
        }
    }

    fn as_v(&self) -> Result<Tuple> {
        Ok(v(
            self[0].as_float()?,
            self[1].as_float()?,
            self[2].as_float()?,
        ))
    }

    fn as_pt(&self) -> Result<Tuple> {
        Ok(pt(
            self[0].as_float()?,
            self[1].as_float()?,
            self[2].as_float()?,
        ))
    }

    fn as_color(&self) -> Result<Color> {
        Ok(Color::new(
            self[0].as_float()?,
            self[1].as_float()?,
            self[2].as_float()?,
        ))
    }

    fn as_camera(&self) -> Result<Camera> {
        let mut camera = Camera::new(0, 0, 0.0);
        let mut from = None;
        let mut to = None;
        let mut up = None;
        let hash = self.as_hash().unwrap();
        for (key, value) in hash.iter() {
            let key = key
                .as_str()
                .expect("Unexpected key type for camera property");
            match key {
                "add" => {}
                "width" => camera.hsize = value.as_float()? as usize,
                "height" => camera.vsize = value.as_float()? as usize,
                "field-of-view" => camera.field_of_view = value.as_float()?,
                "from" => from = Some(value.as_v()?),
                "to" => to = Some(value.as_v()?),
                "up" => up = Some(value.as_v()?),
                _ => return Err(anyhow!("Unexpected camera property: {}", key)),
            }
        }
        camera.transform(view_transform(
            from.expect("camera missing required field 'from'"),
            to.expect("camera missing required field 'to'"),
            up.expect("camera missing required field 'up'"),
        ));
        Ok(camera)
    }

    fn as_light(&self) -> Result<PointLight> {
        let mut light = PointLight::default();
        let hash = self.as_hash().unwrap();
        for (key, value) in hash.iter() {
            let key = key
                .as_str()
                .expect("Unexpected key type for camera property");
            match key {
                "add" => {}
                "at" => light.position = value.as_pt()?,
                "intensity" => light.intensity = value.as_color()?,
                _ => return Err(anyhow!("Unexpected camera property: {}", key)),
            }
        }
        Ok(light)
    }

    fn as_transform(&self, defs: &Definitions) -> Result<Matrix4x4> {
        let mut transform = id();
        for params in self.as_vec().expect("transforms should be an array") {
            match &params {
                Yaml::Array(_) => transform = params.populate_transform(transform)?,
                Yaml::String(name) => transform = defs.transforms[name],
                _ => unreachable!(),
            }
        }
        Ok(transform)
    }

    fn populate_transform(&self, transform: Matrix4x4) -> Result<Matrix4x4> {
        let mut transform = transform;
        let transform_type = self[0].as_str().unwrap();
        match transform_type {
            "rotate-x" => {
                transform = transform.rotate_x(self[1].as_float()?);
            }
            "rotate-y" => {
                transform = transform.rotate_y(self[1].as_float()?);
            }
            "rotate-z" => {
                transform = transform.rotate_z(self[1].as_float()?);
            }
            "scale" => {
                transform = transform.scale(
                    self[1].as_float()?,
                    self[2].as_float()?,
                    self[3].as_float()?,
                );
            }
            "translate" => {
                transform = transform.translate(
                    self[1].as_float()?,
                    self[2].as_float()?,
                    self[3].as_float()?,
                );
            }
            _ => {
                panic!("Unrecognized plane transform: {}", transform_type);
            }
        }
        Ok(transform)
    }
}

struct YamlScene {
    yaml: Vec<Yaml>,
    camera: Option<Camera>,
    world: World,
    definitions: Definitions,
}

impl YamlScene {
    fn new(path: &str) -> Result<YamlScene> {
        let contents = std::fs::read_to_string(&path)?;
        Ok(YamlScene {
            yaml: YamlLoader::load_from_str(&contents)?,
            world: World::empty(),
            camera: None,
            definitions: Definitions::new(),
        })
    }

    fn parse(&mut self) -> Result<()> {
        for obj in self.yaml[0].as_vec().unwrap() {
            if let Yaml::String(r#type) = &obj["add"] {
                println!("Adding {}", r#type);
                match r#type.as_str() {
                    "camera" => {
                        self.camera = Some(obj.as_camera()?);
                    }
                    "light" => {
                        self.world.lights.push(obj.as_light()?);
                    }
                    "group" | "cube" | "plane" | "sphere" | "cylinder" | "cone" | "wacky"
                    | "cap" | "leg" => {
                        self.world.objects.push(add_shape(&obj, &self.definitions)?);
                    }
                    _ => {
                        println!("Uknown object type: {}", r#type);
                    }
                }
            } else if let Yaml::String(_) = &obj["define"] {
                self.definitions.define(obj)?;
            } else {
                panic!("Unexpected object type: {:?}", obj);
            }
        }
        Ok(())
    }

    fn render(&mut self) -> Canvas {
        let camera = self.camera.as_ref().expect("no camera set");
        println!("Rendering");
        let image = camera.render(&mut self.world);
        image
    }

    fn save(&self, path: &str, image: Canvas) -> Result<()> {
        let path = std::path::PathBuf::from(path);
        let image_base = path.file_stem().unwrap().to_str().unwrap();
        let image_path = format!("./{}.png", image_base);
        if image_path.contains("ppm") {
            std::fs::write(image_path, image.to_ppm())?;
        } else {
            image.to_image().save(image_path)?;
        }
        Ok(())
    }
}

fn pattern_props(obj: &Yaml, defs: &Definitions) -> Result<(Color, Color, Matrix4x4)> {
    let mut a = Color::default();
    let mut b = Color::default();
    let mut transform = Matrix4x4::default();
    for (key, value) in obj.as_hash().unwrap().iter() {
        let key = key
            .as_str()
            .expect("Unexpected key type for pattern property");
        match key {
            "colors" => {
                a = value[0].as_color()?;
                b = value[1].as_color()?;
            }
            "transform" => {
                transform = value.as_transform(defs)?;
            }
            _ => return Err(anyhow!("Unexpected pattern property: {}", key)),
        }
    }
    Ok((a, b, transform))
}

fn apply_material(material: Material, obj: &Yaml, defs: &Definitions) -> Result<Material> {
    let mut material = material;
    for (key, value) in obj.as_hash().unwrap().iter() {
        let key = key
            .as_str()
            .expect("Unexpected key type for pattern property");
        match key {
            "pattern" => {
                let r#type = value["type"].as_str().unwrap();
                match r#type {
                    "checkers" => {
                        let (a, b, transform) = pattern_props(value, &defs)?;
                        let mut pattern = checkers_pattern(a, b);
                        pattern.transform = transform;
                        material.pattern = Some(pattern);
                    }
                    "stripes" => {
                        let (a, b, transform) = pattern_props(value, &defs)?;
                        let mut pattern = stripe_pattern(a, b);
                        pattern.transform = transform;
                        material.pattern = Some(pattern);
                    }
                    _ => panic!("Unexpected pattern type: {}", r#type),
                }
            }
            "color" => material.color = value.as_color()?,
            "ambient" => material.ambient = value.as_float()?,
            "diffuse" => material.diffuse = value.as_float()?,
            "specular" => material.specular = value.as_float()?,
            "reflective" => material.reflective = value.as_float()?,
            "shininess" => material.shininess = value.as_float()?,
            "transparency" => material.transparency = value.as_float()?,
            "refractive-index" => material.refractive_index = value.as_float()?,
            _ => {
                return Err(anyhow!("Unknown material property: {}", key));
            }
        }
    }
    Ok(material)
}

fn to_material(obj: &Yaml, defs: &Definitions) -> Result<Material> {
    apply_material(Material::new(), obj, defs)
}

fn apply_shape(shape: &mut dyn Shape, obj: &Yaml, defs: &Definitions) -> Result<()> {
    match &obj["transform"] {
        Yaml::Array(_) => shape.set_transform(obj["transform"].as_transform(&defs)?),
        Yaml::String(name) => shape.set_transform(defs.transforms[name].clone()),
        _ => {}
    }
    let props = &obj["material"];
    match props {
        Yaml::Hash(_) => shape.set_material(to_material(&props, defs)?),
        Yaml::String(name) => shape.set_material(defs.materials[name].clone()),
        _ => {}
    }
    if let Some(shadow) = &obj["shadow"].as_bool() {
        shape.set_shadow(*shadow);
    }
    Ok(())
}

fn add_shape(obj: &Yaml, defs: &Definitions) -> Result<Box<dyn Shape>> {
    let r#type = obj["add"].as_str().unwrap();
    println!("Adding {}", r#type);
    let shape = match r#type {
        "cube" => {
            let mut cube = Cube::new().shape();
            apply_shape(&mut *cube, &obj, defs)?;
            cube
        }
        "plane" => {
            let mut plane = Plane::new().shape();
            apply_shape(&mut *plane, &obj, defs)?;
            plane
        }
        "sphere" => {
            let mut sphere = Sphere::new().shape();
            apply_shape(&mut *sphere, &obj, defs)?;
            sphere
        }
        "cylinder" => {
            let min = obj["min"].as_float()?;
            let max = obj["max"].as_float()?;
            let closed = obj["closed"].as_bool().unwrap();
            let mut cylinder = Cylinder::new(min, max, closed).shape();
            apply_shape(&mut *cylinder, &obj, defs)?;
            cylinder
        }
        "cone" => {
            let min = obj["min"].as_float()?;
            let max = obj["max"].as_float()?;
            let closed = obj["closed"].as_bool().unwrap();
            let mut cone = Cone::new(min, max, closed).shape();
            apply_shape(&mut *cone, &obj, defs)?;
            cone
        }
        "group" => {
            let mut group = Group::new();
            for child_obj in obj["children"].as_vec().unwrap() {
                let child = add_shape(child_obj, defs)?;
                group.add_child(child);
            }
            let mut group = group.shape();
            apply_shape(&mut *group, &obj, defs)?;
            group
        }
        name => {
            if let Some(def) = defs.shapes.get(name) {
                let mut shape = add_shape(&def.clone(), defs)?;
                apply_shape(&mut *shape, &obj, defs)?;
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
    let mut scene = YamlScene::new(&path)?;
    scene.parse()?;

    let image = scene.render();
    // scene.save(&path, image)?;

    Ok(())
}
