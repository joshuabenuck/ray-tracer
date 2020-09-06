// Minimal test case to reproduce a lifetime error I was seeing.
// Remove the "+ '_" lifetime bound on PartialEq to see it.

trait Shape {
    fn local_intersects(&'_ self) -> Vec<Intersection<'_>>;
}

impl PartialEq for dyn Shape + '_ {
    fn eq<'a, 'b>(&'a self, _other: &'b Self) -> bool {
        true
    }
}

struct Sphere {}

impl Shape for Sphere {
    fn local_intersects(&'_ self) -> Vec<Intersection<'_>> {
        vec![Intersection { object: self }]
    }
}

struct Intersection<'a> {
    object: &'a dyn Shape,
}

fn main() {
    let sphere = Sphere {};
    let xs = sphere.local_intersects();
    if xs[0].object == &sphere as &dyn Shape {
        println!("matches");
    }
}
