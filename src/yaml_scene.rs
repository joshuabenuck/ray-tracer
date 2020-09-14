use crate::{
    checkers_pattern, id, pt, stripe_pattern, v, view_transform, Camera, Canvas, Color, Cone, Cube,
    Cylinder, Group, Material, Matrix4x4, Plane, PointLight, Shape, Sphere, Tuple, World,
};
use anyhow::{anyhow, Result};
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
        let name = obj["define"].as_str().expect("define is not a String");
        if name.contains("leg") || name.contains("cap") || name.contains("wacky") {
            let value = &obj["value"];
            self.shapes.insert(name.to_owned(), value.clone());
        } else if name.contains("-material") {
            let mut base = Material::new();
            if let Some(extend) = obj["extend"].as_str() {
                base = self.materials[extend].clone();
            }
            self.materials.insert(
                name.to_owned(),
                obj["value"].populate_material(base, &self)?,
            );
        } else if name.contains("-transform") {
            self.transforms
                .insert(name.to_owned(), obj["value"].as_transform(&self)?);
        } else if name.contains("-object") {
            if let Yaml::Array(value) = &obj["value"] {
                let base = value[0].as_str().expect("base object name not a String");
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
    fn as_material(&self, defs: &Definitions) -> Result<Material>;
    fn populate_material(&self, material: Material, defs: &Definitions) -> Result<Material>;
    fn as_shape(&self, defs: &Definitions) -> Result<Box<dyn Shape>>;
    fn populate_shape(&self, shape: Box<dyn Shape>, defs: &Definitions) -> Result<Box<dyn Shape>>;
    fn pattern_props(&self, defs: &Definitions) -> Result<(Color, Color, Matrix4x4)>;
}

impl YamlExt for Yaml {
    fn as_float(&self) -> Result<f64> {
        match &self {
            Yaml::Real(_) => Ok(self.as_f64().expect("Unable to extract f64 from Real")),
            Yaml::Integer(_) => {
                Ok(self.as_i64().expect("Unable to extract i64 from Integer") as f64)
            }
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
        let mut width = None;
        let mut height = None;
        let mut field_of_view = None;
        let mut from = None;
        let mut to = None;
        let mut up = None;
        let hash = self.as_hash().expect("Camera properties are not a Hash");
        for (key, value) in hash.iter() {
            let key = key
                .as_str()
                .expect("Unexpected key type for camera property");
            match key {
                "add" => {}
                "width" => width = Some(value.as_float()? as usize),
                "height" => height = Some(value.as_float()? as usize),
                "field-of-view" => field_of_view = Some(value.as_float()?),
                "from" => from = Some(value.as_v()?),
                "to" => to = Some(value.as_v()?),
                "up" => up = Some(value.as_v()?),
                _ => return Err(anyhow!("Unexpected camera property: {}", key)),
            }
        }
        let mut camera = Camera::new(
            width.expect("camera missing required field 'width'"),
            height.expect("camera missing required field 'height"),
            field_of_view.expect("camera missing required field 'field-of-view'"),
        );
        camera.transform(view_transform(
            from.expect("camera missing required field 'from'"),
            to.expect("camera missing required field 'to'"),
            up.expect("camera missing required field 'up'"),
        ));
        Ok(camera)
    }

    fn as_light(&self) -> Result<PointLight> {
        let mut light = PointLight::default();
        let hash = self.as_hash().expect("light properties are not a Hash");
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
        let transform_type = self[0].as_str().expect("tranform type is not a String");
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

    fn as_material(&self, defs: &Definitions) -> Result<Material> {
        self.populate_material(Material::new(), defs)
    }

    fn populate_material(&self, material: Material, defs: &Definitions) -> Result<Material> {
        let mut material = material;
        for (key, value) in self
            .as_hash()
            .expect("material properties are not a Hash")
            .iter()
        {
            let key = key
                .as_str()
                .expect("Unexpected key type for pattern property");
            match key {
                "pattern" => {
                    let r#type = value["type"]
                        .as_str()
                        .expect("pattern type is not a String");
                    match r#type {
                        "checkers" => {
                            let (a, b, transform) = value.pattern_props(&defs)?;
                            let mut pattern = checkers_pattern(a, b);
                            pattern.transform = transform;
                            material.pattern = Some(pattern);
                        }
                        "stripes" => {
                            let (a, b, transform) = value.pattern_props(&defs)?;
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

    fn as_shape(&self, defs: &Definitions) -> Result<Box<dyn Shape>> {
        let r#type = self["add"].as_str().expect("add parameter is not a String");
        let shape = match r#type {
            "cube" => {
                let mut shape = Cube::new().shape();
                shape = self.populate_shape(shape, defs)?;
                shape
            }
            "plane" => {
                let mut shape = Plane::new().shape();
                shape = self.populate_shape(shape, defs)?;
                shape
            }
            "sphere" => {
                let mut shape = Sphere::new().shape();
                shape = self.populate_shape(shape, defs)?;
                shape
            }
            "cylinder" => {
                let min = self["min"].as_float()?;
                let max = self["max"].as_float()?;
                let closed = self["closed"]
                    .as_bool()
                    .expect("closed property is not a Bool");
                let mut shape = Cylinder::new(min, max, closed).shape();
                shape = self.populate_shape(shape, defs)?;
                shape
            }
            "cone" => {
                let min = self["min"].as_float()?;
                let max = self["max"].as_float()?;
                let closed = self["closed"]
                    .as_bool()
                    .expect("close property is not a Bool");
                let mut shape = Cone::new(min, max, closed).shape();
                shape = self.populate_shape(shape, defs)?;
                shape
            }
            "group" => {
                let mut group = Group::new();
                for child_obj in self["children"]
                    .as_vec()
                    .expect("children property not an Array")
                {
                    let child = child_obj.as_shape(defs)?;
                    group.add_child(child);
                }
                let mut shape = group.shape();
                shape = self.populate_shape(shape, defs)?;
                shape
            }
            name => {
                if let Some(def) = defs.shapes.get(name) {
                    let mut shape = def.clone().as_shape(defs)?;
                    shape = self.populate_shape(shape, defs)?;
                    shape
                } else {
                    panic!("Unknown shape: {}", name);
                }
            }
        };
        Ok(shape)
    }

    fn populate_shape(&self, shape: Box<dyn Shape>, defs: &Definitions) -> Result<Box<dyn Shape>> {
        let mut shape = shape;
        match &self["transform"] {
            Yaml::Array(_) => shape.set_transform(self["transform"].as_transform(&defs)?),
            Yaml::String(name) => shape.set_transform(defs.transforms[name].clone()),
            _ => {}
        }
        match &self["material"] {
            Yaml::Hash(_) => shape.set_material(self["material"].as_material(defs)?),
            Yaml::String(name) => shape.set_material(defs.materials[name].clone()),
            _ => {}
        }
        if let Some(shadow) = self["shadow"].as_bool() {
            shape.set_shadow(shadow);
        }
        Ok(shape)
    }

    fn pattern_props(&self, defs: &Definitions) -> Result<(Color, Color, Matrix4x4)> {
        let mut a = Color::default();
        let mut b = Color::default();
        let mut transform = Matrix4x4::default();
        for (key, value) in self.as_hash().unwrap().iter() {
            let key = key
                .as_str()
                .expect("Unexpected key type for pattern property");
            match key {
                "type" => {}
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
}

pub struct YamlScene {
    yaml: Vec<Yaml>,
    pub camera: Option<Camera>,
    world: World,
    definitions: Definitions,
}

impl YamlScene {
    pub fn new(path: &str) -> Result<YamlScene> {
        let contents = std::fs::read_to_string(&path)?;
        Ok(YamlScene {
            yaml: YamlLoader::load_from_str(&contents)?,
            world: World::empty(),
            camera: None,
            definitions: Definitions::new(),
        })
    }

    pub fn parse(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let mut previous = "";
        for obj in self.yaml[0].as_vec().unwrap() {
            if let Yaml::String(r#type) = &obj["add"] {
                if r#type == previous {
                    print!(".");
                } else {
                    print!("\nAdding {}", r#type);
                    previous = r#type;
                }
                match r#type.as_str() {
                    "camera" => {
                        self.camera = Some(obj.as_camera()?);
                    }
                    "light" => {
                        self.world.lights.push(obj.as_light()?);
                    }
                    _ => {
                        self.world.objects.push(obj.as_shape(&self.definitions)?);
                    }
                }
            } else if let Yaml::String(name) = &obj["define"] {
                print!("\nDefining {}", name);
                self.definitions.define(obj)?;
            } else {
                panic!("Unexpected object type: {:?}", obj);
            }
        }
        println!("\nParsed in: {:?}", start.elapsed());
        Ok(())
    }

    pub fn render(&mut self) -> Result<Canvas> {
        let start = std::time::Instant::now();
        let camera = self.camera.as_ref().expect("no camera set");
        println!("Rendering");
        let image = camera.render(&mut self.world)?;
        println!("Rendered in: {:?}", start.elapsed());
        Ok(image)
    }

    pub fn save(&self, path: &str, image: Canvas) -> Result<()> {
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
