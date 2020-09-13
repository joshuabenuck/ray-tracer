use crate::{pt, Color, Matrix4x4, Tuple};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ray {
    pub origin: Tuple,
    pub direction: Tuple,
}

impl Ray {
    pub fn new(origin: Tuple, direction: Tuple) -> Ray {
        Ray { origin, direction }
    }

    pub fn position(&self, t: f64) -> Tuple {
        self.origin + self.direction * t
    }

    pub fn transform(&self, m: Matrix4x4) -> Ray {
        Ray {
            origin: m * self.origin,
            direction: m * self.direction,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PointLight {
    pub position: Tuple,
    pub intensity: Color,
}

impl PointLight {
    pub fn new(position: Tuple, intensity: Color) -> PointLight {
        PointLight {
            position,
            intensity,
        }
    }
}

impl Default for PointLight {
    fn default() -> Self {
        PointLight {
            position: pt(0.0, 0.0, 0.0),
            intensity: Color::new(1.0, 1.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{pt, v};

    #[test]
    fn ray_create() {
        // creating and querying a ray
        let origin = pt(1.0, 2.0, 3.0);
        let direction = v(4.0, 5.0, 6.0);
        let r = Ray::new(origin, direction);
        assert_eq!(r.origin, origin);
        assert_eq!(r.direction, direction);
    }

    #[test]
    fn ray_point_from_distance() {
        // computing a point from a distance
        let r = Ray::new(pt(2.0, 3.0, 4.0), v(1.0, 0.0, 0.0));
        assert_eq!(r.position(0.0), pt(2.0, 3.0, 4.0));
        assert_eq!(r.position(1.0), pt(3.0, 3.0, 4.0));
        assert_eq!(r.position(-1.0), pt(1.0, 3.0, 4.0));
        assert_eq!(r.position(2.5), pt(4.5, 3.0, 4.0));
    }

    #[test]
    fn ray_translation() {
        // translating a ray
        let r = Ray::new(pt(1.0, 2.0, 3.0), v(0.0, 1.0, 0.0));
        let m = Matrix4x4::translation(3.0, 4.0, 5.0);
        let r2 = r.transform(m);
        assert_eq!(r2.origin, pt(4.0, 6.0, 8.0));
        assert_eq!(r2.direction, v(0.0, 1.0, 0.0));

        // scaling a ray
        let m = Matrix4x4::scaling(2.0, 3.0, 4.0);
        let r2 = r.transform(m);
        assert_eq!(r2.origin, pt(2.0, 6.0, 12.0));
        assert_eq!(r2.direction, v(0.0, 3.0, 0.0));
    }

    #[test]
    fn point_light() {
        // a point light has a position and an intensity
        let intensity = Color::new(1.0, 1.0, 1.0);
        let position = pt(0.0, 0.0, 0.0);
        let light = PointLight::new(position, intensity);
        assert_eq!(light.position, position);
        assert_eq!(light.intensity, intensity);
    }
}
