pub mod app;
pub mod camera;
pub mod skybox;
pub mod lod;
pub mod marching_cubes;
pub mod mesh;
pub mod window;

pub use self::app::App;
pub use self::camera::Camera;
pub use self::skybox::SkyboxRenderer;
pub use self::lod::LevelOfDetail;
pub use self::marching_cubes::marching_cubes;
pub use self::mesh::{BarycentricVertex, Vertex, Mesh};
pub use self::window::Window;

use glium::texture::{ClientFormat, PixelValue};
use glium::uniforms::{AsUniformValue, UniformValue};
use math::{Matrix4f, Vec2f, Vec3f, Vec4f};

impl AsUniformValue for Matrix4f {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Mat4([[self[(0, 0)], self[(1, 0)], self[(2, 0)], self[(3, 0)]],
                            [self[(0, 1)], self[(1, 1)], self[(2, 1)], self[(3, 1)]],
                            [self[(0, 2)], self[(1, 2)], self[(2, 2)], self[(3, 2)]],
                            [self[(0, 3)], self[(1, 3)], self[(2, 3)], self[(3, 3)]]])
    }
}

impl<'a> AsUniformValue for &'a Matrix4f {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Mat4([[self[(0, 0)], self[(1, 0)], self[(2, 0)], self[(3, 0)]],
                            [self[(0, 1)], self[(1, 1)], self[(2, 1)], self[(3, 1)]],
                            [self[(0, 2)], self[(1, 2)], self[(2, 2)], self[(3, 2)]],
                            [self[(0, 3)], self[(1, 3)], self[(2, 3)], self[(3, 3)]]])
    }
}

impl<'a> AsUniformValue for &'a Vec3f {
    fn as_uniform_value(&self) -> UniformValue {
        UniformValue::Vec3([self[0], self[1], self[2]])
    }
}

unsafe impl PixelValue for Vec2f {
    fn get_format() -> ClientFormat {
        ClientFormat::F32F32
    }
}

unsafe impl PixelValue for Vec3f {
    fn get_format() -> ClientFormat {
        ClientFormat::F32F32F32
    }
}

unsafe impl PixelValue for Vec4f {
    fn get_format() -> ClientFormat {
        ClientFormat::F32F32F32F32
    }
}
