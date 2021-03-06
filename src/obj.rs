use crate::{pt, Group, Triangle, Tuple};
use anyhow::Result;
use std::collections::HashMap;

pub struct ObjParser {
    lines_ignored: usize,
    contents: String,
    vertices: Vec<Tuple>,
    normals: Vec<Tuple>,
    default_group: Group,
    groups: HashMap<String, Group>,
    current_group: Option<String>,
}

impl ObjParser {
    pub fn from_str(contents: &str) -> Result<ObjParser> {
        let mut parser = ObjParser {
            lines_ignored: 0,
            contents: contents.to_string(),
            vertices: vec![pt(0.0, 0.0, 0.0)],
            normals: vec![pt(0.0, 0.0, 0.0)],
            default_group: Group::new(),
            groups: HashMap::new(),
            current_group: None,
        };
        parser.parse()?;
        Ok(parser)
    }

    fn parse_faces(exprs: &mut std::str::Split<&str>) -> Result<Vec<(usize, usize, usize)>> {
        // using 1-based indexes. put in a dummy entry to start
        let mut faces = vec![(0, 0, 0)];
        for p in exprs {
            let mut is = p.split("/");
            let vi = is.next().unwrap().parse()?;
            is.next();
            let ni = if let Some(ni) = is.next() {
                ni.parse()?
            } else {
                0
            };
            faces.push((vi, 0, ni));
        }

        Ok(faces)
    }

    fn parse_f64s(exprs: &mut std::str::Split<&str>) -> Result<Vec<f64>> {
        // using 1-based indexes. put in a dummy entry to start
        let mut f64s = vec![0.0];
        for p in exprs {
            let p = p.trim();
            if p.len() == 0 {
                continue;
            }
            f64s.push(p.parse()?);
        }

        Ok(f64s)
    }

    fn fan_triangulation(&self, idxs: Vec<(usize, usize, usize)>) -> Vec<Triangle> {
        // vertex_idxs is a 1-based array of at least three verticies
        let mut triangles = Vec::new();
        let p1 = self.vertices[idxs[1].0];
        let n1 = self.normals[idxs[1].2];
        for idx in 2..idxs.len() - 1 {
            let p2 = self.vertices[idxs[idx].0];
            let n2 = self.normals[idxs[idx].2];
            let p3 = self.vertices[idxs[idx + 1].0];
            let n3 = self.normals[idxs[idx + 1].2];
            let tri = if idxs[1].2 != 0 {
                Triangle::smooth(p1, p2, p3, n1, n2, n3)
            } else {
                Triangle::new(p1, p2, p3)
            };
            triangles.push(tri);
        }
        triangles
    }

    pub fn parse(&mut self) -> Result<()> {
        let lines = self.contents.split("\n");
        for line in lines {
            let line = line.trim();
            if line.len() == 0 {
                continue;
            }
            let mut exprs = line.split(" ");
            let command = exprs.next().unwrap();
            match command {
                "v" => {
                    let vs = Self::parse_f64s(&mut exprs)?;
                    self.vertices.push(pt(vs[1], vs[2], vs[3]));
                }
                "f" => {
                    for triangle in self.fan_triangulation(Self::parse_faces(&mut exprs)?) {
                        let group = if let Some(current_group) = &self.current_group {
                            self.groups.get_mut(current_group).unwrap()
                        } else {
                            &mut self.default_group
                        };
                        group.add_child(triangle.shape());
                    }
                }
                "g" => {
                    let current_group = exprs.next().unwrap().to_string();
                    self.groups.insert(current_group.clone(), Group::new());
                    self.current_group = Some(current_group);
                }
                "vn" => {
                    let ns = Self::parse_f64s(&mut exprs)?;
                    // 1-based index...
                    self.normals.push(pt(ns[1], ns[2], ns[3]));
                }
                _ => self.lines_ignored += 1,
            }
        }
        Ok(())
    }

    pub fn into_group(self) -> Group {
        let mut group = Group::new();
        let ObjParser {
            default_group,
            groups,
            ..
        } = self;
        let mut groups = groups;
        group.add_child(default_group.shape());
        for (_k, g) in groups.drain() {
            group.add_child(g.shape());
        }
        group
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{pt, Normal};

    #[test]
    fn ignore_gibberish() -> Result<()> {
        // ignoring unrecognized lines
        let gibberish = "
        There was a young lady name Bright
        who traveled much faster than ight.
        She set out one day
        in a relative way,
        and came back the previous night.
        ";
        let parser = ObjParser::from_str(gibberish)?;
        assert_eq!(parser.lines_ignored, 5);
        Ok(())
    }

    #[test]
    fn vertex_records() -> Result<()> {
        let records = "
        v -1 1 0
        v -1.0000 0.5000 0.0000
        v 1 0 0
        v 1 1 0
        ";
        let parser = ObjParser::from_str(records)?;
        assert_eq!(parser.vertices[1], pt(-1.0, 1.0, 0.0));
        assert_eq!(parser.vertices[2], pt(-1.0, 0.5, 0.0));
        assert_eq!(parser.vertices[3], pt(1.0, 0.0, 0.0));
        assert_eq!(parser.vertices[4], pt(1.0, 1.0, 0.0));
        Ok(())
    }

    #[test]
    fn parse_triangle_faces() -> Result<()> {
        // parsing triangle faces
        let contents = "
        v -1 1 0
        v -1 0 0
        v 1 0 0
        v 1 1 0

        f 1 2 3
        f 1 3 4
        ";

        let parser = ObjParser::from_str(contents)?;
        let g = parser.default_group;
        assert_eq!(g.children.len(), 2);
        let t1 = g.children[0].as_any().downcast_ref::<Triangle>().unwrap();
        let t2 = g.children[1].as_any().downcast_ref::<Triangle>().unwrap();
        assert_eq!(t1.p1, parser.vertices[1]);
        assert_eq!(t1.p2, parser.vertices[2]);
        assert_eq!(t1.p3, parser.vertices[3]);
        assert_eq!(t2.p1, parser.vertices[1]);
        assert_eq!(t2.p2, parser.vertices[3]);
        assert_eq!(t2.p3, parser.vertices[4]);
        Ok(())
    }

    #[test]
    fn triangulate_polygons() -> Result<()> {
        // triangulating polygons
        let contents = "
        v -1 1 0
        v -1 0 0
        v 1 0 0
        v 1 1 0
        v 0 2 0

        f 1 2 3 4 5
        ";

        let parser = ObjParser::from_str(contents)?;
        let g = parser.default_group;
        assert_eq!(g.children.len(), 3);
        let t1 = g.children[0].as_any().downcast_ref::<Triangle>().unwrap();
        let t2 = g.children[1].as_any().downcast_ref::<Triangle>().unwrap();
        let t3 = g.children[2].as_any().downcast_ref::<Triangle>().unwrap();
        assert_eq!(t1.p1, parser.vertices[1]);
        assert_eq!(t1.p2, parser.vertices[2]);
        assert_eq!(t1.p3, parser.vertices[3]);
        assert_eq!(t2.p1, parser.vertices[1]);
        assert_eq!(t2.p2, parser.vertices[3]);
        assert_eq!(t2.p3, parser.vertices[4]);
        assert_eq!(t3.p1, parser.vertices[1]);
        assert_eq!(t3.p2, parser.vertices[4]);
        assert_eq!(t3.p3, parser.vertices[5]);
        Ok(())
    }

    #[test]
    fn parse_groups() -> Result<()> {
        // parsing triangle faces
        let contents = "
        v -1 1 0
        v -1 0 0
        v 1 0 0
        v 1 1 0

        g FirstGroup
        f 1 2 3
        g SecondGroup
        f 1 3 4
        ";

        let parser = ObjParser::from_str(contents)?;
        assert_eq!(parser.groups.len(), 2);
        let g1 = &parser.groups["FirstGroup"];
        let g2 = &parser.groups["SecondGroup"];
        let t1 = g1.children[0].as_any().downcast_ref::<Triangle>().unwrap();
        let t2 = g2.children[0].as_any().downcast_ref::<Triangle>().unwrap();
        assert_eq!(t1.p1, parser.vertices[1]);
        assert_eq!(t1.p2, parser.vertices[2]);
        assert_eq!(t1.p3, parser.vertices[3]);
        assert_eq!(t2.p1, parser.vertices[1]);
        assert_eq!(t2.p2, parser.vertices[3]);
        assert_eq!(t2.p3, parser.vertices[4]);

        let g = parser.into_group();
        assert_eq!(g.children.len(), 3);
        // assert_eq!(&*g.children[0], &parser.groups["FirstGroup"] as &dyn Shape);
        // assert_eq!(&*g.children[1], &parser.groups["SecondGroup"] as &dyn Shape);
        Ok(())
    }

    #[test]
    fn vertex_normal_records() -> Result<()> {
        let contents = "
        vn 0 0 1
        vn 0.707 0 -0.707
        vn 1 2 3
        ";
        let parser = ObjParser::from_str(contents)?;
        assert_eq!(parser.normals[1], pt(0.0, 0.0, 1.0));
        assert_eq!(parser.normals[2], pt(0.707, 0.0, -0.707));
        assert_eq!(parser.normals[3], pt(1.0, 2.0, 3.0));
        Ok(())
    }

    #[test]
    fn faces_with_normals() -> Result<()> {
        // faces with normals
        // two spaces after v
        let contents = "
        v  0 1 0
        v -1 0 0
        v 1 0 0

        vn -1 0 0
        vn 1 0 0
        vn 0 1 0

        f 1//3 2//1 3//2
        f 1/0/3 2/102/1 3/14/2
        ";
        let parser = ObjParser::from_str(contents)?;
        let g = parser.default_group;
        assert_eq!(g.children.len(), 2);
        let t1 = g.children[0].as_any().downcast_ref::<Triangle>().unwrap();
        let t2 = g.children[1].as_any().downcast_ref::<Triangle>().unwrap();
        assert_eq!(t1.p1, parser.vertices[1]);
        assert_eq!(t1.p2, parser.vertices[2]);
        assert_eq!(t1.p3, parser.vertices[3]);
        if let Normal::Smooth(n1, n2, n3) = t1.normal {
            assert_eq!(n1, parser.normals[3]);
            assert_eq!(n2, parser.normals[1]);
            assert_eq!(n3, parser.normals[2]);
        } else {
            panic!("wrong normal type");
        }
        assert_eq!(t2, t1);
        Ok(())
    }
}
