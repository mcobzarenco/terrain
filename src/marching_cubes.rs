use num::{Float, Zero, FromPrimitive, ToPrimitive};

use vector::{Vector, Vec3f};
use mesh::{Mesh, Vertex, triangle_normal};
use scalar_field::{ScalarField, SquareField};

pub fn marching_cubes<Field: ScalarField>(field: &Field,
                                          min: &Vec3f,
                                          max: &Vec3f,
                                          step: f32,
                                          iso_value: f32)
                                          -> Mesh {
    let mut vertices = vec![];
    let mut indices = vec![];

    let max_x = max[0];
    let max_y = max[1];
    let max_z = max[2];

    let mut x = min[0];

    let mut index_map: [usize; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    while x + step < max_x {
        let x_dx = x + step;
        let mut y = min[1];

        while y + step < max_y {
            let y_dy = y + step;
            let mut z = min[2];

            while z + step < max_z {
                let z_dz = z + step;
                let values_on_cube = eval_field_at_corners(field, x, y, z, x_dx, y_dy, z_dz);
                let cube_index = find_cube_index(iso_value, values_on_cube);

                // `edges[cube_index]` is a 12 bit number with a 1 for each of
                // the edges that cross the iso-surface
                let edges = EDGE_TABLE[cube_index];
                // println!("{}, {}, {} - edges{:?}", x, y, z, edges);
                if edges == 0 {
                    z += step;
                    continue;
                }

                // edges counted like in http://paulbourke.net/geometry/polygonise/
                // edge 0
                if edges & 0x001 != 0 {
                    index_map[0] = vertices.len();
                    let vertex = intersection_vertex(x,
                                                     y,
                                                     z,
                                                     x_dx,
                                                     Axis::X,
                                                     values_on_cube[0],
                                                     values_on_cube[1],
                                                     iso_value);
                    vertices.push(vertex);
                }
                // edge 1
                if edges & 0x002 != 0 {
                    index_map[1] = vertices.len();
                    let vertex = intersection_vertex(x_dx,
                                                     y,
                                                     z,
                                                     y_dy,
                                                     Axis::Y,
                                                     values_on_cube[1],
                                                     values_on_cube[2],
                                                     iso_value);
                    vertices.push(vertex);
                }
                // edge 2
                if edges & 0x004 != 0 {
                    index_map[2] = vertices.len();
                    let vertex = intersection_vertex(x_dx,
                                                     y_dy,
                                                     z,
                                                     x,
                                                     Axis::X,
                                                     values_on_cube[2],
                                                     values_on_cube[3],
                                                     iso_value);
                    vertices.push(vertex);
                }
                // edge 3
                if edges & 0x008 != 0 {
                    index_map[3] = vertices.len();
                    let vertex = intersection_vertex(x,
                                                     y_dy,
                                                     z,
                                                     y,
                                                     Axis::Y,
                                                     values_on_cube[3],
                                                     values_on_cube[0],
                                                     iso_value);
                    vertices.push(vertex);
                }
                // edge 4
                if edges & 0x010 != 0 {
                    index_map[4] = vertices.len();
                    let vertex = intersection_vertex(x,
                                                     y,
                                                     z_dz,
                                                     x_dx,
                                                     Axis::X,
                                                     values_on_cube[4],
                                                     values_on_cube[5],
                                                     iso_value);
                    vertices.push(vertex);
                }
                // edge 5
                if edges & 0x020 != 0 {
                    index_map[5] = vertices.len();
                    let vertex = intersection_vertex(x_dx,
                                                     y,
                                                     z_dz,
                                                     y_dy,
                                                     Axis::Y,
                                                     values_on_cube[5],
                                                     values_on_cube[6],
                                                     iso_value);
                    vertices.push(vertex);
                }
                // edge 6
                if edges & 0x040 != 0 {
                    index_map[6] = vertices.len();
                    let vertex = intersection_vertex(x_dx,
                                                     y_dy,
                                                     z_dz,
                                                     x,
                                                     Axis::X,
                                                     values_on_cube[6],
                                                     values_on_cube[7],
                                                     iso_value);
                    vertices.push(vertex);
                }
                // edge 7
                if edges & 0x080 != 0 {
                    index_map[7] = vertices.len();
                    let vertex = intersection_vertex(x,
                                                     y_dy,
                                                     z_dz,
                                                     y,
                                                     Axis::Y,
                                                     values_on_cube[7],
                                                     values_on_cube[4],
                                                     iso_value);
                    vertices.push(vertex);
                }
                // edge 8
                if edges & 0x100 != 0 {
                    index_map[8] = vertices.len();
                    let vertex = intersection_vertex(x,
                                                     y,
                                                     z,
                                                     z_dz,
                                                     Axis::Z,
                                                     values_on_cube[0],
                                                     values_on_cube[4],
                                                     iso_value);
                    vertices.push(vertex);
                }
                // edge 9
                if edges & 0x200 != 0 {
                    index_map[9] = vertices.len();
                    let vertex = intersection_vertex(x_dx,
                                                     y,
                                                     z,
                                                     z_dz,
                                                     Axis::Z,
                                                     values_on_cube[1],
                                                     values_on_cube[5],
                                                     iso_value);
                    vertices.push(vertex);
                }
                // edge 10
                if edges & 0x400 != 0 {
                    index_map[10] = vertices.len();
                    let vertex = intersection_vertex(x_dx,
                                                     y_dy,
                                                     z,
                                                     z_dz,
                                                     Axis::Z,
                                                     values_on_cube[2],
                                                     values_on_cube[6],
                                                     iso_value);
                    vertices.push(vertex);
                }
                // edge 11
                if edges & 0x800 != 0 {
                    index_map[11] = vertices.len();
                    let vertex = intersection_vertex(x,
                                                     y_dy,
                                                     z,
                                                     z_dz,
                                                     Axis::Z,
                                                     values_on_cube[3],
                                                     values_on_cube[7],
                                                     iso_value);
                    vertices.push(vertex);
                }

                let triangles_ixes = TRIANGLE_TABLE[cube_index];

                for ix in triangles_ixes.chunks(3) {
                    if ix[0] == -1 {
                        break;
                    }

                    // println!("{:?}", ix);

                //     for triangle in &triangles {
                //         let tri_normal = (vertex[triangle[0]] - vertex[triangle[1]]).cross(vertex[triangle[0]] - vertex[triangle[2]])).normalized();
                //     for i in 0..3 { normal[i] += tri_normal }
                // }
                // for normal in &mut normals {
                //     normal.normalize();
                    // }

                    let i0 = index_map[ix[0] as usize];
                    let i1 = index_map[ix[1] as usize];
                    let i2 = index_map[ix[2] as usize];

                    vertices[i0].normal = normalized_field_gradient_at_vertex::<Field>(field, &vertices[i0].position);
                    vertices[i1].normal = normalized_field_gradient_at_vertex::<Field>(field, &vertices[i1].position);
                    vertices[i2].normal = normalized_field_gradient_at_vertex::<Field>(field, &vertices[i2].position);

                    // normals.push(Normal { normal: normalized_field_gradient_at_vertex::<Field>(field, v0).array() });
                    // normals.push(Normal { normal: normalized_field_gradient_at_vertex::<Field>(field, v1).array() });
                    // normals.push(Normal { normal: normalized_field_gradient_at_vertex::<Field>(field, v2).array() });

                    // let n = triangle_normal(&vertices[i0], &vertices[i1], &vertices[i2]);
                    // vertices[i0].normal = n;
                    // vertices[i1].normal = n;
                    // vertices[i2].normal = n;

                    indices.push(i0 as u32);
                    indices.push(i1 as u32);
                    indices.push(i2 as u32);
                }
                z += step;
            }
            y += step;
        }
        x += step;
    }

    Mesh {
        name: "test".to_owned(),
        vertices: vertices,
        indices: indices,
        tex_vertices: vec![],
    }
}

#[inline]
fn eval_field_at_corners<Field: ScalarField>(field: &Field,
                                             x: f32,
                                             y: f32,
                                             z: f32,
                                             x_dx: f32,
                                             y_dy: f32,
                                             z_dz: f32)
                                             -> [f32; 8] {
    [field.value_at(x, y, z),
     field.value_at(x_dx, y, z),
     field.value_at(x_dx, y_dy, z),
     field.value_at(x, y_dy, z),
     field.value_at(x, y, z_dz),
     field.value_at(x_dx, y, z_dz),
     field.value_at(x_dx, y_dy, z_dz),
     field.value_at(x, y_dy, z_dz)]
}

#[inline]
fn find_cube_index(iso_value: f32, values_on_cube: [f32; 8]) -> usize {
    let mut index = 0;
    for vertex in 0..8 {
        if values_on_cube[vertex] < iso_value {
            index |= 1 << vertex;
        }
    }
    index
}

#[inline]
fn iso_value_interpolation(iso_value: f32,
                           p1: f32,
                           p2: f32,
                           field_value1: f32,
                           field_value2: f32)
                           -> f32 {
    if (field_value1 - field_value2).abs() < 1e-6 {
        (p1 + p2) / 2.0
    } else {
        (p2 - p1) * (iso_value - field_value1) / (field_value2 - field_value1) + p1
    }
}

enum Axis {
    X,
    Y,
    Z,
}

#[inline]
fn intersection_vertex(mut x: f32,
                       mut y: f32,
                       mut z: f32,
                       adjacent: f32,
                       axis: Axis,
                       field_value1: f32,
                       field_value2: f32,
                       iso_value: f32)
                       -> Vertex {
    match axis {
        Axis::X =>
            x = iso_value_interpolation(iso_value, x, adjacent, field_value1, field_value2),
        Axis::Y =>
            y = iso_value_interpolation(iso_value, y, adjacent, field_value1, field_value2),
        Axis::Z =>
            z = iso_value_interpolation(iso_value, z, adjacent, field_value1, field_value2),
    }
    Vertex { position: Vec3f::new(x, y, z), normal: Vec3f::zero() }
}

#[inline]
fn normalized_field_gradient_at_vertex<Field: ScalarField>(field: &ScalarField, vertex: &Vec3f)
                                                -> Vec3f {
    Vec3f::from_array(field.gradient_at(vertex[0], vertex[1], vertex[2]))
        .normalized()
}


#[derive(Copy, Clone, Debug)]
struct Linspace<Scalar: Float + FromPrimitive> {
    remaining: usize,
    step: Scalar,
    value: Scalar,
    stop: Scalar,
}

impl<Scalar: Float + FromPrimitive> Linspace<Scalar> {
    fn new(start: Scalar, stop: Scalar, num: usize) -> Self {
        let step: Scalar;
        if num == 0 {
            step = Scalar::zero();
        } else {
            step = (stop - start) / Scalar::from_usize(num - 1).unwrap()
        }
        Linspace {
            remaining: num,
            step: step,
            value: start,
            stop: stop,
        }
    }
}

impl<Scalar: Float + FromPrimitive> Iterator for Linspace<Scalar> {
    type Item = Scalar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        } else if self.remaining == 1 {
            self.remaining -= 1;
            return Some(self.stop);
        }
        let return_value = self.value;
        self.value = self.value + self.step;
        self.remaining -= 1;
        Some(return_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Linspace;
    use vector::Vec3f;
    use scalar_field::{ScalarField, SquareField};

    #[test]
    fn test_linspace() {
        let mut l1 = Linspace::new(-10.0, 10.0, 0);
        assert_eq!(None, l1.next());

        let mut l1 = Linspace::new(-10.0f32, 10.0, 1);
        let l1_elems: Vec<f32> = l1.collect();
        assert_eq!(vec![10.0], l1_elems);

        let mut l1 = Linspace::new(-10.0f32, 10.0, 2);
        let l1_elems: Vec<f32> = l1.collect();
        assert_eq!(vec![-10.0, 10.0], l1_elems);

        // let mut l1 = Linspace::new(-10.0f32, 10.0, 5);
        // let l1_elems: Vec<f32> = l1.collect();
        // assert_eq!(vec![10.0, -5.0, 0.0, 5.0, 10.0], l1_elems);
    }

    #[test]
    fn test() {
        let field = SquareField;
        let mesh = marching_cubes(&field,
                                  &Vec3f::new(-5., -1., 0.0),
                                  &Vec3f::new(5., 1., 1.),
                                  1.0,
                                  1.0);
        // println!("VERTICES = {:?}", mesh.vertices);
        // println!("INDICES = {:?}", mesh.indices);
    }
}

const EDGE_TABLE: [u16; 256] =
    [0x000, 0x109, 0x203, 0x30a, 0x406, 0x50f, 0x605, 0x70c, 0x80c, 0x905, 0xa0f, 0xb06, 0xc0a,
     0xd03, 0xe09, 0xf00, 0x190, 0x099, 0x393, 0x29a, 0x596, 0x49f, 0x795, 0x69c, 0x99c, 0x895,
     0xb9f, 0xa96, 0xd9a, 0xc93, 0xf99, 0xe90, 0x230, 0x339, 0x033, 0x13a, 0x636, 0x73f, 0x435,
     0x53c, 0xa3c, 0xb35, 0x83f, 0x936, 0xe3a, 0xf33, 0xc39, 0xd30, 0x3a0, 0x2a9, 0x1a3, 0x0aa,
     0x7a6, 0x6af, 0x5a5, 0x4ac, 0xbac, 0xaa5, 0x9af, 0x8a6, 0xfaa, 0xea3, 0xda9, 0xca0, 0x460,
     0x569, 0x663, 0x76a, 0x066, 0x16f, 0x265, 0x36c, 0xc6c, 0xd65, 0xe6f, 0xf66, 0x86a, 0x963,
     0xa69, 0xb60, 0x5f0, 0x4f9, 0x7f3, 0x6fa, 0x1f6, 0x0ff, 0x3f5, 0x2fc, 0xdfc, 0xcf5, 0xfff,
     0xef6, 0x9fa, 0x8f3, 0xbf9, 0xaf0, 0x650, 0x759, 0x453, 0x55a, 0x256, 0x35f, 0x055, 0x15c,
     0xe5c, 0xf55, 0xc5f, 0xd56, 0xa5a, 0xb53, 0x859, 0x950, 0x7c0, 0x6c9, 0x5c3, 0x4ca, 0x3c6,
     0x2cf, 0x1c5, 0x0cc, 0xfcc, 0xec5, 0xdcf, 0xcc6, 0xbca, 0xac3, 0x9c9, 0x8c0, 0x8c0, 0x9c9,
     0xac3, 0xbca, 0xcc6, 0xdcf, 0xec5, 0xfcc, 0x0cc, 0x1c5, 0x2cf, 0x3c6, 0x4ca, 0x5c3, 0x6c9,
     0x7c0, 0x950, 0x859, 0xb53, 0xa5a, 0xd56, 0xc5f, 0xf55, 0xe5c, 0x15c, 0x055, 0x35f, 0x256,
     0x55a, 0x453, 0x759, 0x650, 0xaf0, 0xbf9, 0x8f3, 0x9fa, 0xef6, 0xfff, 0xcf5, 0xdfc, 0x2fc,
     0x3f5, 0x0ff, 0x1f6, 0x6fa, 0x7f3, 0x4f9, 0x5f0, 0xb60, 0xa69, 0x963, 0x86a, 0xf66, 0xe6f,
     0xd65, 0xc6c, 0x36c, 0x265, 0x16f, 0x066, 0x76a, 0x663, 0x569, 0x460, 0xca0, 0xda9, 0xea3,
     0xfaa, 0x8a6, 0x9af, 0xaa5, 0xbac, 0x4ac, 0x5a5, 0x6af, 0x7a6, 0x0aa, 0x1a3, 0x2a9, 0x3a0,
     0xd30, 0xc39, 0xf33, 0xe3a, 0x936, 0x83f, 0xb35, 0xa3c, 0x53c, 0x435, 0x73f, 0x636, 0x13a,
     0x033, 0x339, 0x230, 0xe90, 0xf99, 0xc93, 0xd9a, 0xa96, 0xb9f, 0x895, 0x99c, 0x69c, 0x795,
     0x49f, 0x596, 0x29a, 0x393, 0x099, 0x190, 0xf00, 0xe09, 0xd03, 0xc0a, 0xb06, 0xa0f, 0x905,
     0x80c, 0x70c, 0x605, 0x50f, 0x406, 0x30a, 0x203, 0x109, 0x000];

const TRIANGLE_TABLE: [[i8; 16]; 256] =
    [[-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [0, 8, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [0, 1, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [1, 8, 3, 9, 8, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [1, 2, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [0, 8, 3, 1, 2, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [9, 2, 10, 0, 2, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [2, 8, 3, 2, 10, 8, 10, 9, 8, -1, -1, -1, -1, -1, -1, -1],
     [3, 11, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [0, 11, 2, 8, 11, 0, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [1, 9, 0, 2, 3, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [1, 11, 2, 1, 9, 11, 9, 8, 11, -1, -1, -1, -1, -1, -1, -1],
     [3, 10, 1, 11, 10, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [0, 10, 1, 0, 8, 10, 8, 11, 10, -1, -1, -1, -1, -1, -1, -1],
     [3, 9, 0, 3, 11, 9, 11, 10, 9, -1, -1, -1, -1, -1, -1, -1],
     [9, 8, 10, 10, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [4, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [4, 3, 0, 7, 3, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [0, 1, 9, 8, 4, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [4, 1, 9, 4, 7, 1, 7, 3, 1, -1, -1, -1, -1, -1, -1, -1],
     [1, 2, 10, 8, 4, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [3, 4, 7, 3, 0, 4, 1, 2, 10, -1, -1, -1, -1, -1, -1, -1],
     [9, 2, 10, 9, 0, 2, 8, 4, 7, -1, -1, -1, -1, -1, -1, -1],
     [2, 10, 9, 2, 9, 7, 2, 7, 3, 7, 9, 4, -1, -1, -1, -1],
     [8, 4, 7, 3, 11, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [11, 4, 7, 11, 2, 4, 2, 0, 4, -1, -1, -1, -1, -1, -1, -1],
     [9, 0, 1, 8, 4, 7, 2, 3, 11, -1, -1, -1, -1, -1, -1, -1],
     [4, 7, 11, 9, 4, 11, 9, 11, 2, 9, 2, 1, -1, -1, -1, -1],
     [3, 10, 1, 3, 11, 10, 7, 8, 4, -1, -1, -1, -1, -1, -1, -1],
     [1, 11, 10, 1, 4, 11, 1, 0, 4, 7, 11, 4, -1, -1, -1, -1],
     [4, 7, 8, 9, 0, 11, 9, 11, 10, 11, 0, 3, -1, -1, -1, -1],
     [4, 7, 11, 4, 11, 9, 9, 11, 10, -1, -1, -1, -1, -1, -1, -1],
     [9, 5, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [9, 5, 4, 0, 8, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [0, 5, 4, 1, 5, 0, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [8, 5, 4, 8, 3, 5, 3, 1, 5, -1, -1, -1, -1, -1, -1, -1],
     [1, 2, 10, 9, 5, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [3, 0, 8, 1, 2, 10, 4, 9, 5, -1, -1, -1, -1, -1, -1, -1],
     [5, 2, 10, 5, 4, 2, 4, 0, 2, -1, -1, -1, -1, -1, -1, -1],
     [2, 10, 5, 3, 2, 5, 3, 5, 4, 3, 4, 8, -1, -1, -1, -1],
     [9, 5, 4, 2, 3, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [0, 11, 2, 0, 8, 11, 4, 9, 5, -1, -1, -1, -1, -1, -1, -1],
     [0, 5, 4, 0, 1, 5, 2, 3, 11, -1, -1, -1, -1, -1, -1, -1],
     [2, 1, 5, 2, 5, 8, 2, 8, 11, 4, 8, 5, -1, -1, -1, -1],
     [10, 3, 11, 10, 1, 3, 9, 5, 4, -1, -1, -1, -1, -1, -1, -1],
     [4, 9, 5, 0, 8, 1, 8, 10, 1, 8, 11, 10, -1, -1, -1, -1],
     [5, 4, 0, 5, 0, 11, 5, 11, 10, 11, 0, 3, -1, -1, -1, -1],
     [5, 4, 8, 5, 8, 10, 10, 8, 11, -1, -1, -1, -1, -1, -1, -1],
     [9, 7, 8, 5, 7, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [9, 3, 0, 9, 5, 3, 5, 7, 3, -1, -1, -1, -1, -1, -1, -1],
     [0, 7, 8, 0, 1, 7, 1, 5, 7, -1, -1, -1, -1, -1, -1, -1],
     [1, 5, 3, 3, 5, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [9, 7, 8, 9, 5, 7, 10, 1, 2, -1, -1, -1, -1, -1, -1, -1],
     [10, 1, 2, 9, 5, 0, 5, 3, 0, 5, 7, 3, -1, -1, -1, -1],
     [8, 0, 2, 8, 2, 5, 8, 5, 7, 10, 5, 2, -1, -1, -1, -1],
     [2, 10, 5, 2, 5, 3, 3, 5, 7, -1, -1, -1, -1, -1, -1, -1],
     [7, 9, 5, 7, 8, 9, 3, 11, 2, -1, -1, -1, -1, -1, -1, -1],
     [9, 5, 7, 9, 7, 2, 9, 2, 0, 2, 7, 11, -1, -1, -1, -1],
     [2, 3, 11, 0, 1, 8, 1, 7, 8, 1, 5, 7, -1, -1, -1, -1],
     [11, 2, 1, 11, 1, 7, 7, 1, 5, -1, -1, -1, -1, -1, -1, -1],
     [9, 5, 8, 8, 5, 7, 10, 1, 3, 10, 3, 11, -1, -1, -1, -1],
     [5, 7, 0, 5, 0, 9, 7, 11, 0, 1, 0, 10, 11, 10, 0, -1],
     [11, 10, 0, 11, 0, 3, 10, 5, 0, 8, 0, 7, 5, 7, 0, -1],
     [11, 10, 5, 7, 11, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [10, 6, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [0, 8, 3, 5, 10, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [9, 0, 1, 5, 10, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [1, 8, 3, 1, 9, 8, 5, 10, 6, -1, -1, -1, -1, -1, -1, -1],
     [1, 6, 5, 2, 6, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [1, 6, 5, 1, 2, 6, 3, 0, 8, -1, -1, -1, -1, -1, -1, -1],
     [9, 6, 5, 9, 0, 6, 0, 2, 6, -1, -1, -1, -1, -1, -1, -1],
     [5, 9, 8, 5, 8, 2, 5, 2, 6, 3, 2, 8, -1, -1, -1, -1],
     [2, 3, 11, 10, 6, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [11, 0, 8, 11, 2, 0, 10, 6, 5, -1, -1, -1, -1, -1, -1, -1],
     [0, 1, 9, 2, 3, 11, 5, 10, 6, -1, -1, -1, -1, -1, -1, -1],
     [5, 10, 6, 1, 9, 2, 9, 11, 2, 9, 8, 11, -1, -1, -1, -1],
     [6, 3, 11, 6, 5, 3, 5, 1, 3, -1, -1, -1, -1, -1, -1, -1],
     [0, 8, 11, 0, 11, 5, 0, 5, 1, 5, 11, 6, -1, -1, -1, -1],
     [3, 11, 6, 0, 3, 6, 0, 6, 5, 0, 5, 9, -1, -1, -1, -1],
     [6, 5, 9, 6, 9, 11, 11, 9, 8, -1, -1, -1, -1, -1, -1, -1],
     [5, 10, 6, 4, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [4, 3, 0, 4, 7, 3, 6, 5, 10, -1, -1, -1, -1, -1, -1, -1],
     [1, 9, 0, 5, 10, 6, 8, 4, 7, -1, -1, -1, -1, -1, -1, -1],
     [10, 6, 5, 1, 9, 7, 1, 7, 3, 7, 9, 4, -1, -1, -1, -1],
     [6, 1, 2, 6, 5, 1, 4, 7, 8, -1, -1, -1, -1, -1, -1, -1],
     [1, 2, 5, 5, 2, 6, 3, 0, 4, 3, 4, 7, -1, -1, -1, -1],
     [8, 4, 7, 9, 0, 5, 0, 6, 5, 0, 2, 6, -1, -1, -1, -1],
     [7, 3, 9, 7, 9, 4, 3, 2, 9, 5, 9, 6, 2, 6, 9, -1],
     [3, 11, 2, 7, 8, 4, 10, 6, 5, -1, -1, -1, -1, -1, -1, -1],
     [5, 10, 6, 4, 7, 2, 4, 2, 0, 2, 7, 11, -1, -1, -1, -1],
     [0, 1, 9, 4, 7, 8, 2, 3, 11, 5, 10, 6, -1, -1, -1, -1],
     [9, 2, 1, 9, 11, 2, 9, 4, 11, 7, 11, 4, 5, 10, 6, -1],
     [8, 4, 7, 3, 11, 5, 3, 5, 1, 5, 11, 6, -1, -1, -1, -1],
     [5, 1, 11, 5, 11, 6, 1, 0, 11, 7, 11, 4, 0, 4, 11, -1],
     [0, 5, 9, 0, 6, 5, 0, 3, 6, 11, 6, 3, 8, 4, 7, -1],
     [6, 5, 9, 6, 9, 11, 4, 7, 9, 7, 11, 9, -1, -1, -1, -1],
     [10, 4, 9, 6, 4, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [4, 10, 6, 4, 9, 10, 0, 8, 3, -1, -1, -1, -1, -1, -1, -1],
     [10, 0, 1, 10, 6, 0, 6, 4, 0, -1, -1, -1, -1, -1, -1, -1],
     [8, 3, 1, 8, 1, 6, 8, 6, 4, 6, 1, 10, -1, -1, -1, -1],
     [1, 4, 9, 1, 2, 4, 2, 6, 4, -1, -1, -1, -1, -1, -1, -1],
     [3, 0, 8, 1, 2, 9, 2, 4, 9, 2, 6, 4, -1, -1, -1, -1],
     [0, 2, 4, 4, 2, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [8, 3, 2, 8, 2, 4, 4, 2, 6, -1, -1, -1, -1, -1, -1, -1],
     [10, 4, 9, 10, 6, 4, 11, 2, 3, -1, -1, -1, -1, -1, -1, -1],
     [0, 8, 2, 2, 8, 11, 4, 9, 10, 4, 10, 6, -1, -1, -1, -1],
     [3, 11, 2, 0, 1, 6, 0, 6, 4, 6, 1, 10, -1, -1, -1, -1],
     [6, 4, 1, 6, 1, 10, 4, 8, 1, 2, 1, 11, 8, 11, 1, -1],
     [9, 6, 4, 9, 3, 6, 9, 1, 3, 11, 6, 3, -1, -1, -1, -1],
     [8, 11, 1, 8, 1, 0, 11, 6, 1, 9, 1, 4, 6, 4, 1, -1],
     [3, 11, 6, 3, 6, 0, 0, 6, 4, -1, -1, -1, -1, -1, -1, -1],
     [6, 4, 8, 11, 6, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [7, 10, 6, 7, 8, 10, 8, 9, 10, -1, -1, -1, -1, -1, -1, -1],
     [0, 7, 3, 0, 10, 7, 0, 9, 10, 6, 7, 10, -1, -1, -1, -1],
     [10, 6, 7, 1, 10, 7, 1, 7, 8, 1, 8, 0, -1, -1, -1, -1],
     [10, 6, 7, 10, 7, 1, 1, 7, 3, -1, -1, -1, -1, -1, -1, -1],
     [1, 2, 6, 1, 6, 8, 1, 8, 9, 8, 6, 7, -1, -1, -1, -1],
     [2, 6, 9, 2, 9, 1, 6, 7, 9, 0, 9, 3, 7, 3, 9, -1],
     [7, 8, 0, 7, 0, 6, 6, 0, 2, -1, -1, -1, -1, -1, -1, -1],
     [7, 3, 2, 6, 7, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [2, 3, 11, 10, 6, 8, 10, 8, 9, 8, 6, 7, -1, -1, -1, -1],
     [2, 0, 7, 2, 7, 11, 0, 9, 7, 6, 7, 10, 9, 10, 7, -1],
     [1, 8, 0, 1, 7, 8, 1, 10, 7, 6, 7, 10, 2, 3, 11, -1],
     [11, 2, 1, 11, 1, 7, 10, 6, 1, 6, 7, 1, -1, -1, -1, -1],
     [8, 9, 6, 8, 6, 7, 9, 1, 6, 11, 6, 3, 1, 3, 6, -1],
     [0, 9, 1, 11, 6, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [7, 8, 0, 7, 0, 6, 3, 11, 0, 11, 6, 0, -1, -1, -1, -1],
     [7, 11, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [7, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [3, 0, 8, 11, 7, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [0, 1, 9, 11, 7, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [8, 1, 9, 8, 3, 1, 11, 7, 6, -1, -1, -1, -1, -1, -1, -1],
     [10, 1, 2, 6, 11, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [1, 2, 10, 3, 0, 8, 6, 11, 7, -1, -1, -1, -1, -1, -1, -1],
     [2, 9, 0, 2, 10, 9, 6, 11, 7, -1, -1, -1, -1, -1, -1, -1],
     [6, 11, 7, 2, 10, 3, 10, 8, 3, 10, 9, 8, -1, -1, -1, -1],
     [7, 2, 3, 6, 2, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [7, 0, 8, 7, 6, 0, 6, 2, 0, -1, -1, -1, -1, -1, -1, -1],
     [2, 7, 6, 2, 3, 7, 0, 1, 9, -1, -1, -1, -1, -1, -1, -1],
     [1, 6, 2, 1, 8, 6, 1, 9, 8, 8, 7, 6, -1, -1, -1, -1],
     [10, 7, 6, 10, 1, 7, 1, 3, 7, -1, -1, -1, -1, -1, -1, -1],
     [10, 7, 6, 1, 7, 10, 1, 8, 7, 1, 0, 8, -1, -1, -1, -1],
     [0, 3, 7, 0, 7, 10, 0, 10, 9, 6, 10, 7, -1, -1, -1, -1],
     [7, 6, 10, 7, 10, 8, 8, 10, 9, -1, -1, -1, -1, -1, -1, -1],
     [6, 8, 4, 11, 8, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [3, 6, 11, 3, 0, 6, 0, 4, 6, -1, -1, -1, -1, -1, -1, -1],
     [8, 6, 11, 8, 4, 6, 9, 0, 1, -1, -1, -1, -1, -1, -1, -1],
     [9, 4, 6, 9, 6, 3, 9, 3, 1, 11, 3, 6, -1, -1, -1, -1],
     [6, 8, 4, 6, 11, 8, 2, 10, 1, -1, -1, -1, -1, -1, -1, -1],
     [1, 2, 10, 3, 0, 11, 0, 6, 11, 0, 4, 6, -1, -1, -1, -1],
     [4, 11, 8, 4, 6, 11, 0, 2, 9, 2, 10, 9, -1, -1, -1, -1],
     [10, 9, 3, 10, 3, 2, 9, 4, 3, 11, 3, 6, 4, 6, 3, -1],
     [8, 2, 3, 8, 4, 2, 4, 6, 2, -1, -1, -1, -1, -1, -1, -1],
     [0, 4, 2, 4, 6, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [1, 9, 0, 2, 3, 4, 2, 4, 6, 4, 3, 8, -1, -1, -1, -1],
     [1, 9, 4, 1, 4, 2, 2, 4, 6, -1, -1, -1, -1, -1, -1, -1],
     [8, 1, 3, 8, 6, 1, 8, 4, 6, 6, 10, 1, -1, -1, -1, -1],
     [10, 1, 0, 10, 0, 6, 6, 0, 4, -1, -1, -1, -1, -1, -1, -1],
     [4, 6, 3, 4, 3, 8, 6, 10, 3, 0, 3, 9, 10, 9, 3, -1],
     [10, 9, 4, 6, 10, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [4, 9, 5, 7, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [0, 8, 3, 4, 9, 5, 11, 7, 6, -1, -1, -1, -1, -1, -1, -1],
     [5, 0, 1, 5, 4, 0, 7, 6, 11, -1, -1, -1, -1, -1, -1, -1],
     [11, 7, 6, 8, 3, 4, 3, 5, 4, 3, 1, 5, -1, -1, -1, -1],
     [9, 5, 4, 10, 1, 2, 7, 6, 11, -1, -1, -1, -1, -1, -1, -1],
     [6, 11, 7, 1, 2, 10, 0, 8, 3, 4, 9, 5, -1, -1, -1, -1],
     [7, 6, 11, 5, 4, 10, 4, 2, 10, 4, 0, 2, -1, -1, -1, -1],
     [3, 4, 8, 3, 5, 4, 3, 2, 5, 10, 5, 2, 11, 7, 6, -1],
     [7, 2, 3, 7, 6, 2, 5, 4, 9, -1, -1, -1, -1, -1, -1, -1],
     [9, 5, 4, 0, 8, 6, 0, 6, 2, 6, 8, 7, -1, -1, -1, -1],
     [3, 6, 2, 3, 7, 6, 1, 5, 0, 5, 4, 0, -1, -1, -1, -1],
     [6, 2, 8, 6, 8, 7, 2, 1, 8, 4, 8, 5, 1, 5, 8, -1],
     [9, 5, 4, 10, 1, 6, 1, 7, 6, 1, 3, 7, -1, -1, -1, -1],
     [1, 6, 10, 1, 7, 6, 1, 0, 7, 8, 7, 0, 9, 5, 4, -1],
     [4, 0, 10, 4, 10, 5, 0, 3, 10, 6, 10, 7, 3, 7, 10, -1],
     [7, 6, 10, 7, 10, 8, 5, 4, 10, 4, 8, 10, -1, -1, -1, -1],
     [6, 9, 5, 6, 11, 9, 11, 8, 9, -1, -1, -1, -1, -1, -1, -1],
     [3, 6, 11, 0, 6, 3, 0, 5, 6, 0, 9, 5, -1, -1, -1, -1],
     [0, 11, 8, 0, 5, 11, 0, 1, 5, 5, 6, 11, -1, -1, -1, -1],
     [6, 11, 3, 6, 3, 5, 5, 3, 1, -1, -1, -1, -1, -1, -1, -1],
     [1, 2, 10, 9, 5, 11, 9, 11, 8, 11, 5, 6, -1, -1, -1, -1],
     [0, 11, 3, 0, 6, 11, 0, 9, 6, 5, 6, 9, 1, 2, 10, -1],
     [11, 8, 5, 11, 5, 6, 8, 0, 5, 10, 5, 2, 0, 2, 5, -1],
     [6, 11, 3, 6, 3, 5, 2, 10, 3, 10, 5, 3, -1, -1, -1, -1],
     [5, 8, 9, 5, 2, 8, 5, 6, 2, 3, 8, 2, -1, -1, -1, -1],
     [9, 5, 6, 9, 6, 0, 0, 6, 2, -1, -1, -1, -1, -1, -1, -1],
     [1, 5, 8, 1, 8, 0, 5, 6, 8, 3, 8, 2, 6, 2, 8, -1],
     [1, 5, 6, 2, 1, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [1, 3, 6, 1, 6, 10, 3, 8, 6, 5, 6, 9, 8, 9, 6, -1],
     [10, 1, 0, 10, 0, 6, 9, 5, 0, 5, 6, 0, -1, -1, -1, -1],
     [0, 3, 8, 5, 6, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [10, 5, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [11, 5, 10, 7, 5, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [11, 5, 10, 11, 7, 5, 8, 3, 0, -1, -1, -1, -1, -1, -1, -1],
     [5, 11, 7, 5, 10, 11, 1, 9, 0, -1, -1, -1, -1, -1, -1, -1],
     [10, 7, 5, 10, 11, 7, 9, 8, 1, 8, 3, 1, -1, -1, -1, -1],
     [11, 1, 2, 11, 7, 1, 7, 5, 1, -1, -1, -1, -1, -1, -1, -1],
     [0, 8, 3, 1, 2, 7, 1, 7, 5, 7, 2, 11, -1, -1, -1, -1],
     [9, 7, 5, 9, 2, 7, 9, 0, 2, 2, 11, 7, -1, -1, -1, -1],
     [7, 5, 2, 7, 2, 11, 5, 9, 2, 3, 2, 8, 9, 8, 2, -1],
     [2, 5, 10, 2, 3, 5, 3, 7, 5, -1, -1, -1, -1, -1, -1, -1],
     [8, 2, 0, 8, 5, 2, 8, 7, 5, 10, 2, 5, -1, -1, -1, -1],
     [9, 0, 1, 5, 10, 3, 5, 3, 7, 3, 10, 2, -1, -1, -1, -1],
     [9, 8, 2, 9, 2, 1, 8, 7, 2, 10, 2, 5, 7, 5, 2, -1],
     [1, 3, 5, 3, 7, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [0, 8, 7, 0, 7, 1, 1, 7, 5, -1, -1, -1, -1, -1, -1, -1],
     [9, 0, 3, 9, 3, 5, 5, 3, 7, -1, -1, -1, -1, -1, -1, -1],
     [9, 8, 7, 5, 9, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [5, 8, 4, 5, 10, 8, 10, 11, 8, -1, -1, -1, -1, -1, -1, -1],
     [5, 0, 4, 5, 11, 0, 5, 10, 11, 11, 3, 0, -1, -1, -1, -1],
     [0, 1, 9, 8, 4, 10, 8, 10, 11, 10, 4, 5, -1, -1, -1, -1],
     [10, 11, 4, 10, 4, 5, 11, 3, 4, 9, 4, 1, 3, 1, 4, -1],
     [2, 5, 1, 2, 8, 5, 2, 11, 8, 4, 5, 8, -1, -1, -1, -1],
     [0, 4, 11, 0, 11, 3, 4, 5, 11, 2, 11, 1, 5, 1, 11, -1],
     [0, 2, 5, 0, 5, 9, 2, 11, 5, 4, 5, 8, 11, 8, 5, -1],
     [9, 4, 5, 2, 11, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [2, 5, 10, 3, 5, 2, 3, 4, 5, 3, 8, 4, -1, -1, -1, -1],
     [5, 10, 2, 5, 2, 4, 4, 2, 0, -1, -1, -1, -1, -1, -1, -1],
     [3, 10, 2, 3, 5, 10, 3, 8, 5, 4, 5, 8, 0, 1, 9, -1],
     [5, 10, 2, 5, 2, 4, 1, 9, 2, 9, 4, 2, -1, -1, -1, -1],
     [8, 4, 5, 8, 5, 3, 3, 5, 1, -1, -1, -1, -1, -1, -1, -1],
     [0, 4, 5, 1, 0, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [8, 4, 5, 8, 5, 3, 9, 0, 5, 0, 3, 5, -1, -1, -1, -1],
     [9, 4, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [4, 11, 7, 4, 9, 11, 9, 10, 11, -1, -1, -1, -1, -1, -1, -1],
     [0, 8, 3, 4, 9, 7, 9, 11, 7, 9, 10, 11, -1, -1, -1, -1],
     [1, 10, 11, 1, 11, 4, 1, 4, 0, 7, 4, 11, -1, -1, -1, -1],
     [3, 1, 4, 3, 4, 8, 1, 10, 4, 7, 4, 11, 10, 11, 4, -1],
     [4, 11, 7, 9, 11, 4, 9, 2, 11, 9, 1, 2, -1, -1, -1, -1],
     [9, 7, 4, 9, 11, 7, 9, 1, 11, 2, 11, 1, 0, 8, 3, -1],
     [11, 7, 4, 11, 4, 2, 2, 4, 0, -1, -1, -1, -1, -1, -1, -1],
     [11, 7, 4, 11, 4, 2, 8, 3, 4, 3, 2, 4, -1, -1, -1, -1],
     [2, 9, 10, 2, 7, 9, 2, 3, 7, 7, 4, 9, -1, -1, -1, -1],
     [9, 10, 7, 9, 7, 4, 10, 2, 7, 8, 7, 0, 2, 0, 7, -1],
     [3, 7, 10, 3, 10, 2, 7, 4, 10, 1, 10, 0, 4, 0, 10, -1],
     [1, 10, 2, 8, 7, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [4, 9, 1, 4, 1, 7, 7, 1, 3, -1, -1, -1, -1, -1, -1, -1],
     [4, 9, 1, 4, 1, 7, 0, 8, 1, 8, 7, 1, -1, -1, -1, -1],
     [4, 0, 3, 7, 4, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [4, 8, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [9, 10, 8, 10, 11, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [3, 0, 9, 3, 9, 11, 11, 9, 10, -1, -1, -1, -1, -1, -1, -1],
     [0, 1, 10, 0, 10, 8, 8, 10, 11, -1, -1, -1, -1, -1, -1, -1],
     [3, 1, 10, 11, 3, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [1, 2, 11, 1, 11, 9, 9, 11, 8, -1, -1, -1, -1, -1, -1, -1],
     [3, 0, 9, 3, 9, 11, 1, 2, 9, 2, 11, 9, -1, -1, -1, -1],
     [0, 2, 11, 8, 0, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [3, 2, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [2, 3, 8, 2, 8, 10, 10, 8, 9, -1, -1, -1, -1, -1, -1, -1],
     [9, 10, 2, 0, 9, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [2, 3, 8, 2, 8, 10, 0, 1, 8, 1, 10, 8, -1, -1, -1, -1],
     [1, 10, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [1, 3, 8, 9, 1, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [0, 9, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [0, 3, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
     [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]];