use crate::{Color, Matrix4x4, Shape, Tuple};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PatternDesign {
    Stripe(Color, Color),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Pattern {
    design: PatternDesign,
    transform: Matrix4x4,
}

impl Pattern {
    pub fn stripe_at(&self, point: Tuple) -> Color {
        match self.design {
            PatternDesign::Stripe(a, b) => {
                if point.x.floor() as isize % 2 == 0 {
                    a
                } else {
                    b
                }
            }
        }
    }

    pub fn stripe_at_object(&self, object: Shape, world_point: Tuple) -> Color {
        let object_point = object.transform.inverse().unwrap() * world_point;
        let pattern_point = self.transform.inverse().unwrap() * object_point;
        self.stripe_at(pattern_point)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{black, pt, sphere, spheret, white, Matrix4x4};

    #[test]
    fn pattern_stripe() {
        // creating a stripe pattern
        let pattern = stripe_pattern(white(), black());
        // assert_eq!(pattern.a, white());
        // assert_eq!(pattern.b, black());

        // a stripe pattern is constant in y
        assert_eq!(pattern.stripe_at(pt(0.0, 0.0, 0.0)), white());
        assert_eq!(pattern.stripe_at(pt(0.0, 1.0, 0.0)), white());
        assert_eq!(pattern.stripe_at(pt(0.0, 2.0, 0.0)), white());

        // a stripe pattern is constant in z
        assert_eq!(pattern.stripe_at(pt(0.0, 0.0, 0.0)), white());
        assert_eq!(pattern.stripe_at(pt(0.0, 0.0, 1.0)), white());
        assert_eq!(pattern.stripe_at(pt(0.0, 0.0, 2.0)), white());

        // a stripe pattern is constant in x
        assert_eq!(pattern.stripe_at(pt(0.0, 0.0, 0.0)), white());
        assert_eq!(pattern.stripe_at(pt(0.9, 0.0, 0.0)), white());
        assert_eq!(pattern.stripe_at(pt(1.0, 0.0, 0.0)), black());
        assert_eq!(pattern.stripe_at(pt(1.1, 0.0, 0.0)), black());
        assert_eq!(pattern.stripe_at(pt(-0.1, 0.0, 0.0)), black());
        assert_eq!(pattern.stripe_at(pt(-1.0, 0.0, 0.0)), black());
        assert_eq!(pattern.stripe_at(pt(-1.1, 0.0, 0.0)), white());
    }

    #[test]
    fn pattern_transformations() {
        // stripes with an object transformation
        let object = spheret(Matrix4x4::scaling(2.0, 2.0, 2.0));
        let pattern = stripe_pattern(white(), black());
        let c = pattern.stripe_at_object(object, pt(1.5, 0.0, 0.0));
        assert_eq!(c, white());

        // stripes with a pattern tranformation
        let object = sphere();
        let pattern = stripe_patternt(white(), black(), Matrix4x4::scaling(2.0, 2.0, 2.0));
        let c = pattern.stripe_at_object(object, pt(1.5, 0.0, 0.0));
        assert_eq!(c, white());

        // stripes with both an object and a pattern transformation
        let object = spheret(Matrix4x4::scaling(2.0, 2.0, 2.0));
        let pattern = stripe_patternt(white(), black(), Matrix4x4::scaling(2.0, 2.0, 2.0));
        let c = pattern.stripe_at_object(object, pt(2.5, 0.0, 0.0));
        assert_eq!(c, white());
    }
}
