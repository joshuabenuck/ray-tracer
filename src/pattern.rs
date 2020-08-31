use crate::{Color, Matrix4x4, Shape, Tuple};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PatternDesign {
    Stripe(Color, Color),
    Gradient(Color, Color),
    Ring(Color, Color),
    Checkers(Color, Color),
    Test,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Pattern {
    pub design: PatternDesign,
    pub transform: Matrix4x4,
}

impl Pattern {
    pub fn pattern_at(&self, point: Tuple) -> Color {
        match self.design {
            PatternDesign::Stripe(a, b) => {
                if point.x.floor() as isize % 2 == 0 {
                    a
                } else {
                    b
                }
            }
            // take the distance between the two colors, multiply by the
            // fractional portion of the x coordinate, and add the product
            // to the first color.
            PatternDesign::Gradient(a, b) => {
                let distance = b - a;
                let fraction = point.x - point.x.floor();

                a + distance * fraction
            }
            PatternDesign::Ring(a, b) => {
                let x2 = point.x * point.x;
                let z2 = point.z * point.z;
                if (x2 + z2).sqrt() as isize % 2 == 0 {
                    a
                } else {
                    b
                }
            }
            PatternDesign::Checkers(a, b) => {
                if (point.x.floor() + point.y.floor() + point.z.floor()) as isize % 2 == 0 {
                    a
                } else {
                    b
                }
            }
            PatternDesign::Test => Color::new(point.x, point.y, point.z),
        }
    }

    pub fn pattern_at_object(&self, object: &Shape, world_point: Tuple) -> Color {
        let object_point = object.transform.inverse().unwrap() * world_point;
        let pattern_point = self.transform.inverse().unwrap() * object_point;
        self.pattern_at(pattern_point)
    }
}

pub fn stripe_pattern(a: Color, b: Color) -> Pattern {
    Pattern {
        design: PatternDesign::Stripe(a, b),
        transform: Matrix4x4::identity(),
    }
}

pub fn stripe_patternt(a: Color, b: Color, transform: Matrix4x4) -> Pattern {
    Pattern {
        design: PatternDesign::Stripe(a, b),
        transform,
    }
}

pub fn gradient_pattern(a: Color, b: Color) -> Pattern {
    Pattern {
        design: PatternDesign::Gradient(a, b),
        transform: Matrix4x4::identity(),
    }
}

pub fn ring_pattern(a: Color, b: Color) -> Pattern {
    Pattern {
        design: PatternDesign::Ring(a, b),
        transform: Matrix4x4::identity(),
    }
}

pub fn checkers_pattern(a: Color, b: Color) -> Pattern {
    Pattern {
        design: PatternDesign::Checkers(a, b),
        transform: Matrix4x4::identity(),
    }
}

pub fn test_pattern() -> Pattern {
    Pattern {
        design: PatternDesign::Test,
        transform: Matrix4x4::identity(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{black, pt, sphere, spheret, white, Matrix4x4};

    #[test]
    fn pattern_default() {
        // the default pattern transformation
        let mut pattern = test_pattern();
        assert_eq!(pattern.transform, Matrix4x4::identity());

        // assigning a transformation
        pattern.transform = Matrix4x4::translation(1.0, 2.0, 3.0);
        assert_eq!(pattern.transform, Matrix4x4::translation(1.0, 2.0, 3.0));

        // a pattern with an object transformation
        let shape = spheret(Matrix4x4::scaling(2.0, 2.0, 2.0));
        let pattern = test_pattern();
        let c = pattern.pattern_at_object(&shape, pt(2.0, 3.0, 4.0));
        assert_eq!(c, Color::new(1.0, 1.5, 2.0));

        // a pattern with a pattern transformation
        let shape = spheret(Matrix4x4::scaling(2.0, 2.0, 2.0));
        let mut pattern = test_pattern();
        pattern.transform = Matrix4x4::translation(0.5, 1.0, 1.5);
        let c = pattern.pattern_at_object(&shape, pt(2.5, 3.0, 3.5));
        assert_eq!(c, Color::new(0.75, 0.5, 0.25));
    }

    #[test]
    fn pattern_stripe() {
        // creating a stripe pattern
        let pattern = stripe_pattern(white(), black());
        // assert_eq!(pattern.a, white());
        // assert_eq!(pattern.b, black());

        // a stripe pattern is constant in y
        assert_eq!(pattern.pattern_at(pt(0.0, 0.0, 0.0)), white());
        assert_eq!(pattern.pattern_at(pt(0.0, 1.0, 0.0)), white());
        assert_eq!(pattern.pattern_at(pt(0.0, 2.0, 0.0)), white());

        // a stripe pattern is constant in z
        assert_eq!(pattern.pattern_at(pt(0.0, 0.0, 0.0)), white());
        assert_eq!(pattern.pattern_at(pt(0.0, 0.0, 1.0)), white());
        assert_eq!(pattern.pattern_at(pt(0.0, 0.0, 2.0)), white());

        // a stripe pattern is constant in x
        assert_eq!(pattern.pattern_at(pt(0.0, 0.0, 0.0)), white());
        assert_eq!(pattern.pattern_at(pt(0.9, 0.0, 0.0)), white());
        assert_eq!(pattern.pattern_at(pt(1.0, 0.0, 0.0)), black());
        assert_eq!(pattern.pattern_at(pt(1.1, 0.0, 0.0)), black());
        assert_eq!(pattern.pattern_at(pt(-0.1, 0.0, 0.0)), black());
        assert_eq!(pattern.pattern_at(pt(-1.0, 0.0, 0.0)), black());
        assert_eq!(pattern.pattern_at(pt(-1.1, 0.0, 0.0)), white());
    }

    #[test]
    fn pattern_transformations() {
        // stripes with an object transformation
        let object = spheret(Matrix4x4::scaling(2.0, 2.0, 2.0));
        let pattern = stripe_pattern(white(), black());
        let c = pattern.pattern_at_object(&object, pt(1.5, 0.0, 0.0));
        assert_eq!(c, white());

        // stripes with a pattern tranformation
        let object = sphere();
        let pattern = stripe_patternt(white(), black(), Matrix4x4::scaling(2.0, 2.0, 2.0));
        let c = pattern.pattern_at_object(&object, pt(1.5, 0.0, 0.0));
        assert_eq!(c, white());

        // stripes with both an object and a pattern transformation
        let object = spheret(Matrix4x4::scaling(2.0, 2.0, 2.0));
        let pattern = stripe_patternt(white(), black(), Matrix4x4::scaling(2.0, 2.0, 2.0));
        let c = pattern.pattern_at_object(&object, pt(2.5, 0.0, 0.0));
        assert_eq!(c, white());
    }

    #[test]
    fn pattern_gradient() {
        // a gradient linearly interpolates between colors
        let pattern = gradient_pattern(white(), black());
        assert_eq!(pattern.pattern_at(pt(0.0, 0.0, 0.0)), white());
        assert_eq!(
            pattern.pattern_at(pt(0.25, 0.0, 0.0)),
            Color::new(0.75, 0.75, 0.75)
        );
        assert_eq!(
            pattern.pattern_at(pt(0.5, 0.0, 0.0)),
            Color::new(0.5, 0.5, 0.5)
        );
        assert_eq!(
            pattern.pattern_at(pt(0.75, 0.0, 0.0)),
            Color::new(0.25, 0.25, 0.25)
        );
    }

    #[test]
    fn pattern_ring() {
        // a ring should extend in both x and z
        let pattern = ring_pattern(white(), black());
        assert_eq!(pattern.pattern_at(pt(0.0, 0.0, 0.0)), white());
        assert_eq!(pattern.pattern_at(pt(1.0, 0.0, 0.0)), black());
        assert_eq!(pattern.pattern_at(pt(0.0, 0.0, 1.0)), black());
        // 0.708 = just slightly more than 2.0_f64.sqrt() / 2.0
        assert_eq!(pattern.pattern_at(pt(0.708, 0.0, 0.708)), black());
    }

    #[test]
    fn pattern_checkers() {
        // checkers should repeat in x
        let pattern = checkers_pattern(white(), black());
        assert_eq!(pattern.pattern_at(pt(0.0, 0.0, 0.0)), white());
        assert_eq!(pattern.pattern_at(pt(0.99, 0.0, 0.0)), white());
        assert_eq!(pattern.pattern_at(pt(1.01, 0.0, 0.0)), black());

        // checkers should repeat in y
        let pattern = checkers_pattern(white(), black());
        assert_eq!(pattern.pattern_at(pt(0.0, 0.0, 0.0)), white());
        assert_eq!(pattern.pattern_at(pt(0.0, 0.99, 0.0)), white());
        assert_eq!(pattern.pattern_at(pt(0.0, 1.01, 0.0)), black());

        // checkers should repeat in z
        let pattern = checkers_pattern(white(), black());
        assert_eq!(pattern.pattern_at(pt(0.0, 0.0, 0.0)), white());
        assert_eq!(pattern.pattern_at(pt(0.0, 0.0, 0.99)), white());
        assert_eq!(pattern.pattern_at(pt(0.0, 0.0, 1.01)), black());
    }
}
