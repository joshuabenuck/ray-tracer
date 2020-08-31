use crate::{Color, Pattern, PointLight, Shape, Tuple};

pub fn m() -> Material {
    Material::new()
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Material {
    pub color: Color,
    pub pattern: Option<Pattern>,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
    pub reflective: f64,
    pub transparency: f64,
    pub refractive_index: f64,
}

impl Material {
    pub fn new() -> Material {
        Material {
            color: Color::new(1.0, 1.0, 1.0),
            pattern: None,
            ambient: 0.1,
            diffuse: 0.9,
            specular: 0.9,
            shininess: 200.0,
            reflective: 0.0,
            transparency: 0.0,
            refractive_index: 1.0,
        }
    }

    pub fn rgb(mut self, r: f64, g: f64, b: f64) -> Material {
        self.color = Color::new(r, g, b);
        self
    }

    pub fn color(mut self, color: Color) -> Material {
        self.color = color;
        self
    }

    pub fn ambient(mut self, ambient: f64) -> Material {
        self.ambient = ambient;
        self
    }

    pub fn diffuse(mut self, diffuse: f64) -> Material {
        self.diffuse = diffuse;
        self
    }

    pub fn specular(mut self, specular: f64) -> Material {
        self.specular = specular;
        self
    }

    pub fn shininess(mut self, shininess: f64) -> Material {
        self.shininess = shininess;
        self
    }

    pub fn transparency(mut self, transparency: f64) -> Material {
        self.transparency = transparency;
        self
    }

    pub fn reflective(mut self, reflective: f64) -> Material {
        self.reflective = reflective;
        self
    }

    pub fn refractive_index(mut self, refractive_index: f64) -> Material {
        self.refractive_index = refractive_index;
        self
    }

    pub fn lighting(
        &self,
        object: &Shape,
        light: &PointLight,
        point: &Tuple,
        eyev: &Tuple,
        normalv: &Tuple,
        in_shadow: bool,
    ) -> Color {
        let color = if let Some(pattern) = self.pattern {
            pattern.pattern_at_object(object, *point)
        } else {
            self.color
        };
        // combine the surface color with the light's color / intensity
        let effective_color = color * light.intensity;

        // find the direction to the light source
        let lightv = (light.position - *point).normalize();

        // comput the ambient contribution
        let ambient = effective_color * self.ambient;
        if in_shadow {
            return ambient;
        }

        // light_dot_normal represents the cosine of the angle between the
        // light vector and the normal vector. A negative number means the
        // light is on the other side of teh surface.
        let light_dot_normal = lightv.dot(&normalv);
        let (diffuse, specular) = if light_dot_normal < 0.0 {
            (Color::new(0.0, 0.0, 0.0), Color::new(0.0, 0.0, 0.0))
        } else {
            // compute the diffuse contribution
            let diffuse = effective_color * self.diffuse * light_dot_normal;

            // reflect-dot_eye represents the cosine of the angle between the
            // reflection vector and the eye vector. A negative number means the
            // light reflects away from the eye.
            let reflectv = -lightv.reflect(&normalv);
            let reflect_dot_eye = reflectv.dot(eyev);

            let specular = if reflect_dot_eye <= 0.0 {
                Color::new(0.0, 0.0, 0.0)
            } else {
                // compute the specular contribution
                let factor = reflect_dot_eye.powf(self.shininess);
                light.intensity * self.specular * factor
            };
            (diffuse, specular)
        };
        ambient + diffuse + specular
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{black, pt, sphere, stripe_pattern, v, white};

    #[test]
    fn material() {
        // the default material
        let m = Material::new();
        assert_eq!(m.color, Color::new(1.0, 1.0, 1.0));
        assert_eq!(m.ambient, 0.1);
        assert_eq!(m.diffuse, 0.9);
        assert_eq!(m.specular, 0.9);
        assert_eq!(m.shininess, 200.0);

        // reflectivity for the default material
        assert_eq!(m.reflective, 0.0);

        // transparency and refractive_index for the default material
        assert_eq!(m.transparency, 0.0);
        assert_eq!(m.refractive_index, 1.0);
    }

    #[test]
    fn lighting() {
        let m = Material::new();
        let position = pt(0.0, 0.0, 0.0);

        // lighting with the eye between the light and the surface
        // ambient, diffuse, and specular all at full strength
        let eyev = v(0.0, 0.0, -1.0);
        let normalv = v(0.0, 0.0, -1.0);
        let light = PointLight::new(pt(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = m.lighting(&sphere(), &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.9, 1.9, 1.9));

        // lighting with the surface in shadow
        let in_shadow = true;
        let result = m.lighting(&sphere(), &light, &position, &eyev, &normalv, in_shadow);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));

        // lighting with the eye between light and surface, eye offset 45 degrees
        // ambient and diffuse unchanged because the angle between them is unchanged
        // specular drops off to effectively zero
        let eyev = v(0.0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0);
        let result = m.lighting(&sphere(), &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.0, 1.0, 1.0));

        // lighting with eye opposite surface, light offset 45 degrees
        let eyev = v(0.0, 0.0, -1.0);
        let light = PointLight::new(pt(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = m.lighting(&sphere(), &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(0.7364, 0.7364, 0.7364));

        // lighting with eye in the path of the reflection vector
        // makes specular at full strength with ambient and diffuse same as last test
        let eyev = v(0.0, -2.0_f64.sqrt() / 2.0, -2.0_f64.sqrt() / 2.0);
        let light = PointLight::new(pt(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let result = m.lighting(&sphere(), &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.6364, 1.6364, 1.6364,));

        // light with the light behind the surface
        // Since the light doesn't illuminate the surface, the diffuse and specular
        // components go to zero.
        // The total intensity should be the same as the ambient component
        let eyev = v(0.0, 0.0, -1.0);
        let light = PointLight::new(pt(0.0, 0.0, 10.0), Color::new(1.0, 1.0, 1.0));
        let result = m.lighting(&sphere(), &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn lighting_with_pattern() {
        // lighting with a pattern applied
        let mut m = Material::new();
        m.pattern = Some(stripe_pattern(white(), black()));
        m.ambient = 1.0;
        m.diffuse = 0.0;
        m.specular = 0.0;
        let eyev = v(0.0, 0.0, -1.0);
        let normalv = v(0.0, 0.0, -1.0);
        let light = PointLight::new(pt(0.0, 0.0, -10.0), white());
        let c1 = m.lighting(
            &sphere(),
            &light,
            &pt(0.9, 0.0, 0.0),
            &eyev,
            &normalv,
            false,
        );
        let c2 = m.lighting(
            &sphere(),
            &light,
            &pt(1.1, 0.0, 0.0),
            &eyev,
            &normalv,
            false,
        );
        assert_eq!(c1, white());
        assert_eq!(c2, black());
    }
}
