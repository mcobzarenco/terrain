use std::borrow::Cow;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::mem::size_of;
use glium::vertex::{self, Attribute, AttributeType, VertexFormat};
use nalgebra::{Cross, Norm};
use num::Zero;
use wavefront_obj::obj as wavefront_obj;

use errors::*;
use utils::read_utf8_file;
use math::{GpuScalar, Vec2f, Vec3f};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PlainVertex {
    position: Vec3f,
}

impl<'a> From<&'a [GpuScalar; 3]> for PlainVertex {
    fn from(array: &'a [GpuScalar; 3]) -> PlainVertex {
        PlainVertex { position: Vec3f::new(array[0], array[1], array[2]) }
    }
}

implement_vertex!(PlainVertex, position);

pub trait NormalVertex {
    fn position(&self) -> &Vec3f;
    fn normal(&self) -> &Vec3f;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    pub position: Vec3f,
    pub normal: Vec3f,
}

impl NormalVertex for Vertex {
    fn position(&self) -> &Vec3f {
        &self.position
    }

    fn normal(&self) -> &Vec3f {
        &self.normal
    }
}

implement_vertex!(Vertex, position, normal);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VertexWithAttribute<A: Attribute> {
    pub position: Vec3f,
    pub normal: Vec3f,
    pub attribute: A,
}

impl<A> NormalVertex for VertexWithAttribute<A>
    where A: Attribute
{
    fn position(&self) -> &Vec3f {
        &self.position
    }

    fn normal(&self) -> &Vec3f {
        &self.normal
    }
}

impl<A> vertex::Vertex for VertexWithAttribute<A>
    where A: Attribute + Copy
{
    fn build_bindings() -> VertexFormat {
        let position_ix = size_of::<Vec3f>();
        let normal_ix = position_ix + size_of::<Vec3f>();
        let attribute_ix = normal_ix + size_of::<A>();

        Cow::Owned(vec![(Cow::Borrowed("position"), position_ix, Vec3f::get_type()),
                        (Cow::Borrowed("normal"), normal_ix, Vec3f::get_type()),
                        (Cow::Borrowed("attribute"), attribute_ix, Vec3f::get_type())])
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BarycentricVertex {
    pub position: Vec3f,
    pub normal: Vec3f,
    pub bary_coord: Vec3f,
}

impl NormalVertex for BarycentricVertex {
    fn position(&self) -> &Vec3f {
        &self.position
    }

    fn normal(&self) -> &Vec3f {
        &self.normal
    }
}

implement_vertex!(BarycentricVertex, position, normal, bary_coord);

#[inline]
pub fn triangle_normal(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Vec3f {
    Vec3f::from((v2.position - v1.position).cross(&(v3.position - v1.position)).normalize())
}

#[derive(Clone, Debug, PartialEq)]
pub struct TexVertex {
    pub uv: Vec2f,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mesh<V: NormalVertex> {
    pub name: String,
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
}

impl Mesh<Vertex> {
    pub fn with_barycentric_coordinates(self) -> Mesh<BarycentricVertex> {
        // TODO(mcobzarenco): This doesn't work if the vertices are used by more
        // than one triangle. Does it become a coloring problem then?
        // println!("{} {} {} {}",
        //          self.vertices.len(),
        //          self.indices.len(),
        //          HashSet::<u32>::from_iter(self.indices.clone().into_iter()).len(),
        //          self.indices.len());
        // assert!(self.vertices.len() == self.indices.len() &&
        //         HashSet::<u32>::from_iter(self.indices.clone().into_iter()).len() ==
        //         self.indices.len());

        // self.vertices.into_iter().map(|vertex| {
        //     BarycentricVertex {
        //         position: vertex.position,
        //         normal: vertex.normal,
        //         bary_coord: Vec3f::zero(),
        //     }
        // });

        let mut bary_vertices = vec![];
        let mut bary_indices = vec![];

        for index in self.indices.as_slice().chunks(3) {
            // Triangle corners:
            let (a, b, c) = (index[0] as usize, index[1] as usize, index[2] as usize);
            bary_indices.push(bary_vertices.len() as u32);
            bary_vertices.push(BarycentricVertex {
                position: self.vertices[a].position,
                normal: self.vertices[a].normal,
                bary_coord: Vec3f::new(0.0, 0.0, 1.0),
            });
            bary_indices.push(bary_vertices.len() as u32);
            bary_vertices.push(BarycentricVertex {
                position: self.vertices[b].position,
                normal: self.vertices[b].normal,
                bary_coord: Vec3f::new(0.0, 1.0, 0.0),
            });
            bary_indices.push(bary_vertices.len() as u32);
            bary_vertices.push(BarycentricVertex {
                position: self.vertices[c].position,
                normal: self.vertices[c].normal,
                bary_coord: Vec3f::new(1.0, 0.0, 0.0),
            });
        }

        Mesh {
            name: self.name,
            vertices: bary_vertices,
            indices: bary_indices,
        }
    }

    fn from_wavefront_obj(obj: wavefront_obj::Object) -> Self {
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
        }
    }
}

pub fn load_mesh_from_file(path: &str) -> Result<Vec<Mesh<Vertex>>> {
    let contents = try!(read_utf8_file(path).chain_err(|| "Couldn't open mesh file."));
    load_mesh_from_str(contents)
}

pub fn load_mesh_from_str(mesh_raw: String) -> Result<Vec<Mesh<Vertex>>> {
    let obj_set = try!(wavefront_obj::parse(mesh_raw)
        .map_err(|e| ErrorKind::LoadAssetError(e.message)));
    Ok(obj_set.objects.into_iter().map(Mesh::from_wavefront_obj).collect())
}

unsafe impl Attribute for Vec3f {
    fn get_type() -> AttributeType {
        AttributeType::F32F32F32
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_triangle_normal() {}
}
