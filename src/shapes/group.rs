use crate::{Intersection, Material, Matrix4x4, Props, Ray, Shape, Tuple};
use std::any::Any;

pub struct Group {
    props: Props,
    pub children: Vec<Box<dyn Shape>>,
}

impl Group {
    pub fn new() -> Group {
        Group {
            props: Props::default(),
            children: Vec::new(),
        }
    }

    pub fn transform(mut self, transform: Matrix4x4) -> Self {
        self.props.transform = transform;
        self
    }

    pub fn material(mut self, material: Material) -> Self {
        self.props.material = material;
        self
    }

    pub fn add_child(&mut self, child: Box<dyn Shape>) {
        self.children.push(child);
    }

    pub fn shape(self) -> Box<dyn Shape> {
        Box::new(self)
    }
}

impl Shape for Group {
    fn local_intersect(&'_ self, ray: &Ray) -> Vec<Intersection<'_>> {
        let mut xs: Vec<Intersection> = self
            .children
            .iter()
            .flat_map(|c| c.intersect(&ray))
            .collect();
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        xs
    }

    fn local_normal_at(&self, _local_point: Tuple, _i: &Intersection) -> Tuple {
        unreachable!()
    }

    fn common(&self) -> &Props {
        &self.props
    }

    fn common_mut(&mut self) -> &mut Props {
        &mut self.props
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn shape_eq(&self, other: &dyn Any) -> bool {
        match other.downcast_ref::<Self>() {
            Some(other) => {
                self.type_id() == other.type_id() && self.children.len() == other.children.len()
            }
            None => false,
        }
    }

    fn refresh_parents(&mut self) {
        let material = self.common().material;
        let mut child_transforms = self.parent_transforms().clone();
        let mut child_inverses = self.parent_inverses().clone();
        child_transforms.push(*Shape::transform(self));
        child_inverses.push(Shape::inverse(self));
        for child in &mut self.children {
            if child.material() == &Material::new() {
                child.set_material(material);
            }
            child.set_parent_transforms(child_transforms.clone());
            child.set_parent_inverses(child_inverses.clone());
            if let Some(group) = child.as_any_mut().downcast_mut::<Group>() {
                group.refresh_parents();
            }
        }
    }

    fn includes(&self, other: &dyn Shape) -> bool {
        for child in &self.children {
            if child.includes(other) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{pt, v, Sphere};
    use std::f64::consts::PI;

    #[test]
    fn group_create() {
        // creating a new group
        let g = Group::new();
        assert_eq!(g.children.len(), 0);
        let g = g.shape();
        assert_eq!(g.transform(), &Matrix4x4::identity());
    }

    #[test]
    fn group_add_children() {
        // adding a child to a group
        let mut g = Group::new();
        let s = Sphere::new();
        g.add_child(Box::new(s));
        g.refresh_parents();
        assert_eq!(g.children.len(), 1);
        let s = &g.children[0];
        assert!(s.parent_transforms().len() > 0);
        assert_eq!(s.parent_transforms()[0], g.props.transform);
    }

    #[test]
    fn ray_intersection_with_empty_group() {
        // intersecting a ray with an empty group
        let g = Group::new();
        let r = Ray::new(pt(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0));
        let xs = g.local_intersect(&r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn ray_intersection_with_non_empty_group() {
        // intersecting a ray with a non-empty group
        let mut g = Group::new();
        let s1 = Sphere::new();
        let s1_transform = Matrix4x4::identity();
        let s2 = Sphere::new().transform(Matrix4x4::translation(0.0, 0.0, -3.0));
        let s2_transform = Matrix4x4::translation(0.0, 0.0, -3.0);
        let s3 = Sphere::new().transform(Matrix4x4::translation(5.0, 0.0, 0.0));
        g.add_child(s1.shape());
        g.add_child(s2.shape());
        g.add_child(s3.shape());
        let r = Ray::new(pt(0.0, 0.0, -5.0), v(0.0, 0.0, 1.0));
        let xs = g.local_intersect(&r);
        assert_eq!(g.children.len(), 3);
        assert_eq!(xs.len(), 4);
        assert_eq!(xs[0].object.transform(), &s2_transform);
        assert_eq!(xs[1].object.transform(), &s2_transform);
        assert_eq!(xs[2].object.transform(), &s1_transform);
        assert_eq!(xs[3].object.transform(), &s1_transform);
    }

    #[test]
    fn ray_intersection_with_transformed_group() {
        // intersecting a transformed group
        let mut g = Group::new();
        g.set_transform(Matrix4x4::scaling(2.0, 2.0, 2.0));
        let s = Sphere::new().transform(Matrix4x4::translation(5.0, 0.0, 0.0));
        g.add_child(s.shape());
        let r = Ray::new(pt(10.0, 0.0, -10.0), v(0.0, 0.0, 1.0));
        let xs = g.intersect(&r);
        assert_eq!(xs.len(), 2);
    }

    #[test]
    fn world_to_object_space() {
        // converting a point from world to object space
        let mut g1 = Group::new().transform(Matrix4x4::rotation_y(PI / 2.0));
        let mut g2 = Group::new().transform(Matrix4x4::scaling(2.0, 2.0, 2.0));
        let s = Sphere::new().transform(Matrix4x4::translation(5.0, 0.0, 0.0));
        g2.add_child(s.shape());
        g1.add_child(g2.shape());
        g1.refresh_parents();
        let s = &g1.as_any().downcast_ref::<Group>().unwrap().children[0]
            .as_any()
            .downcast_ref::<Group>()
            .unwrap()
            .children[0];
        assert!(s.parent_transforms().len() == 2);
        let p = s.world_to_object(pt(-2.0, 0.0, -10.0));
        assert_eq!(p, pt(0.0, 0.0, -1.0));
    }

    #[test]
    fn normal_from_object_to_world_space() {
        // converting a normal from object to world space
        let mut g1 = Group::new().transform(Matrix4x4::rotation_y(PI / 2.0));
        let mut g2 = Group::new().transform(Matrix4x4::scaling(1.0, 2.0, 3.0));
        let s = Sphere::new().transform(Matrix4x4::translation(5.0, 0.0, 0.0));
        g2.add_child(s.shape());
        g1.add_child(g2.shape());
        g1.refresh_parents();
        let s = &g1.as_any().downcast_ref::<Group>().unwrap().children[0]
            .as_any()
            .downcast_ref::<Group>()
            .unwrap()
            .children[0];
        let n = s.normal_to_world(v(
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
            3.0_f64.sqrt() / 3.0,
        ));
        assert_eq!(n, v(0.28571, 0.42857, -0.85714));
    }

    #[test]
    fn normal_on_child_object() {
        // finding the normal on a child object
        let mut g1 = Group::new().transform(Matrix4x4::rotation_y(PI / 2.0));
        let mut g2 = Group::new().transform(Matrix4x4::scaling(1.0, 2.0, 3.0));
        let s = Sphere::new().transform(Matrix4x4::translation(5.0, 0.0, 0.0));
        g2.add_child(s.shape());
        g1.add_child(g2.shape());
        g1.refresh_parents();
        let s = &g1.as_any().downcast_ref::<Group>().unwrap().children[0]
            .as_any()
            .downcast_ref::<Group>()
            .unwrap()
            .children[0]
            .as_any()
            .downcast_ref::<Sphere>()
            .unwrap();
        let i = &Intersection::new(0.0, *s);
        let n = s.normal_at(pt(1.7321, 1.1547, -5.5774), &i);
        assert_eq!(n, v(0.2857, 0.42854, -0.85716));
    }
}
