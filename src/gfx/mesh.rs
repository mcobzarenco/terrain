use glium::vertex::{Attribute, AttributeType};
use wavefront_obj::obj as wavefront_obj;

use errors::*;
use utils::read_utf8_file;
use math::{Vec2f, Vec3f, Vector};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    pub position: Vec3f,
    pub normal: Vec3f,
}

unsafe impl Attribute for Vec3f {
    fn get_type() -> AttributeType {
        AttributeType::F32F32F32
    }
}

implement_vertex!(Vertex, position, normal);

#[inline]
pub fn triangle_normal(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Vec3f {
    (v2.position - v1.position).cross(&(v3.position - v1.position)).normalized()
}

#[derive(Clone, Debug, PartialEq)]
pub struct TexVertex {
    pub uv: Vec2f,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub tex_vertices: Vec<TexVertex>,
}

impl Mesh {
    fn from_wavefront_obj(obj: wavefront_obj::Object) -> Mesh {
        Mesh {
            name: obj.name,
            vertices: obj.vertices
                .into_iter()
                .zip(obj.normals.into_iter())
                .map(|v| {
                    Vertex {
                        position: Vec3f::new(v.0.x as f32, v.0.y as f32, v.0.z as f32),
                        normal: Vec3f::new(v.1.x as f32, v.1.y as f32, v.1.z as f32),
                    }
                })
                .collect(),
            indices: obj.geometry
                .into_iter()
                .map(|g| {
                    g.shapes.into_iter().map(|s| {
                        if let wavefront_obj::Primitive::Triangle(i1, i2, i3) = s.primitive {
                            (i1.0 as u32, i2.0 as u32, i3.0 as u32)
                        } else {
                            panic!("Non-triangle shape.");
                        }
                    })
                })
                .fold(vec![], |mut acc, xss| {
                    for xs in xss {
                        acc.push(xs.0);
                        acc.push(xs.1);
                        acc.push(xs.2);
                    }
                    acc
                }),
            tex_vertices: vec![],
        }
    }
}

pub fn load_mesh_from_file(path: &str) -> Result<Vec<Mesh>> {
    let contents = try!(read_utf8_file(path).chain_err(|| "Couldn't open mesh file."));
    load_mesh_from_str(contents)
}

pub fn load_mesh_from_str(mesh_raw: String) -> Result<Vec<Mesh>> {
    let obj_set = try!(wavefront_obj::parse(mesh_raw)
        .map_err(|e| ErrorKind::LoadAssetError(e.message)));
    Ok(obj_set.objects.into_iter().map(Mesh::from_wavefront_obj).collect())
}

mod tests {
    use super::*;

    #[test]
    fn test_triangle_normal() {}
}
