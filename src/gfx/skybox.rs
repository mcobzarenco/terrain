use std::path::Path;
use std::time::Instant;
use std::fmt::Debug;
use glium::{BlitTarget, DrawParameters, Frame, Program, Rect, Surface, IndexBuffer, VertexBuffer};
use glium::draw_parameters::BackfaceCullingMode;
use glium::framebuffer::SimpleFrameBuffer;
use glium::index::PrimitiveType;
use glium::texture::{CubeLayer, Cubemap, RawImage2d, Texture2d};
use glium::uniforms::MagnifySamplerFilter;
use image;
use nalgebra::{PerspectiveMatrix3, Translation};

use errors::{ChainErr, Result};
use gfx::{Camera, Window};
use gfx::mesh::PlainVertex;
use math::{GpuScalar, Vec3f};

pub struct SkyboxRenderer<'a> {
    cubemap: Cubemap,
    draw_parameters: DrawParameters<'a>,
    program: Program,
    vertex_buffer: VertexBuffer<PlainVertex>,
    index_buffer: IndexBuffer<u32>,
    perspective: PerspectiveMatrix3<GpuScalar>,
}

impl<'a> SkyboxRenderer<'a> {
    pub fn new(window: &Window) -> Result<Self> {
        let program = try!(window.program(&VERTEX_SHADER, &FRAGMENT_SHADER));
        let params = DrawParameters {
            backface_culling: BackfaceCullingMode::CullingDisabled,
            ..Default::default()
        };

        let skybox_vertices: Vec<PlainVertex> =
            SKYBOX_VERTICES.iter().map(PlainVertex::from).collect();
        let skybox_indices: Vec<u32> = SKYBOX_INDICES.iter().cloned().collect();
        let vertex_buffer = try!(VertexBuffer::new(window.facade(), &skybox_vertices)
            .chain_err(|| "Cannot create vertex buffer."));
        let index_buffer = try!(IndexBuffer::new(window.facade(),
                                                 PrimitiveType::TrianglesList,
                                                 &skybox_indices)
            .chain_err(|| "Cannot create index buffer."));

        let perspective = perspective_matrix(window.aspect());
        Ok(SkyboxRenderer {
            cubemap: try!(Cubemap::empty(window.facade(), 1024)
                .chain_err(|| "Could not create cubemap texture.")),
            draw_parameters: params,
            program: program,
            index_buffer: index_buffer,
            vertex_buffer: vertex_buffer,
            perspective: perspective,
        })
    }

    pub fn load<P>(&mut self, window: &Window, path: P) -> Result<()>
        where P: AsRef<Path> + Debug
    {
        let instant = Instant::now();
        let image = try!(image::open(path.as_ref())
                .chain_err(|| format!("Could not load image at {:?}", path)))
            .to_rgb();
        info!("to_rgba - elapsed {:?}", instant.elapsed());

        let (width, height) = image.dimensions();
        info!("Loaded Skybox asset with width={:?} height={:?} path={:?}",
              width,
              height,
              path);
        assert!((width / 4) as u32 == (height / 3) as u32);
        let step = (height / 3) as u32;
        info!("step: {}", step);

        let image = RawImage2d::from_raw_rgb(image.into_raw(), (width, height));
        info!("RawImage2d::from_raw_rgba - elapsed {:?}",
              instant.elapsed());
        let source_tex = try!(Texture2d::new(window.facade(), image)
            .chain_err(|| format!("Could not create texture from {:?}", path)));
        info!("Texture2d::new() - elapsed {:?}", instant.elapsed());


        let target_rect = BlitTarget {
            left: 0,
            bottom: 0,
            width: 1024,
            height: 1024,
        };

        let source_rect = Rect {
            left: step,
            bottom: 0,
            width: step,
            height: step,
        };
        let cube_face = try!(self.surface_for_face(window, CubeLayer::PositiveY));
        source_tex.as_surface().blit_color(&source_rect,
                                           &cube_face,
                                           &target_rect,
                                           MagnifySamplerFilter::Linear);
        let source_rect = Rect {
            left: step,
            bottom: step,
            width: step,
            height: step,
        };
        let cube_face = try!(self.surface_for_face(window, CubeLayer::PositiveZ));
        source_tex.as_surface().blit_color(&source_rect,
                                           &cube_face,
                                           &target_rect,
                                           MagnifySamplerFilter::Linear);
        let source_rect = Rect {
            left: step,
            bottom: step * 2,
            width: step,
            height: step,
        };
        let cube_face = try!(self.surface_for_face(window, CubeLayer::NegativeY));
        source_tex.as_surface().blit_color(&source_rect,
                                           &cube_face,
                                           &target_rect,
                                           MagnifySamplerFilter::Linear);
        let source_rect = Rect {
            left: step * 2,
            bottom: step,
            width: step,
            height: step,
        };
        let cube_face = try!(self.surface_for_face(window, CubeLayer::PositiveX));
        source_tex.as_surface().blit_color(&source_rect,
                                           &cube_face,
                                           &target_rect,
                                           MagnifySamplerFilter::Linear);

        let source_rect = Rect {
            left: step * 3,
            bottom: step,
            width: step,
            height: step,
        };
        let cube_face = try!(self.surface_for_face(window, CubeLayer::NegativeZ));
        source_tex.as_surface().blit_color(&source_rect,
                                           &cube_face,
                                           &target_rect,
                                           MagnifySamplerFilter::Linear);

        let source_rect = Rect {
            left: 0,
            bottom: step,
            width: step,
            height: step,
        };
        let cube_face = try!(self.surface_for_face(window, CubeLayer::NegativeX));
        source_tex.as_surface().blit_color(&source_rect,
                                           &cube_face,
                                           &target_rect,
                                           MagnifySamplerFilter::Linear);
        info!("Blit - elapsed {:?}", instant.elapsed());

        Ok(())
    }

    #[inline]
    pub fn render(&mut self, frame: &mut Frame, camera: &Camera) -> Result<()> {
        let SkyboxRenderer { ref cubemap,
                             ref draw_parameters,
                             ref program,
                             ref vertex_buffer,
                             ref index_buffer,
                             ref mut perspective,
                             .. } = *self;

        let frame_aspect = frame_aspect(frame);
        if perspective.aspect() != frame_aspect {
            info!("Aspect ratio ({:?} -> {:?}) - recomputing perspective matrix.",
                  perspective.aspect(),
                  frame_aspect);
            *perspective = perspective_matrix(frame_aspect);
        }
        // info!("New aspect{:?}", perspective.aspect());

        let view = camera.view_matrix();
        // let mvp = perspective *a
        let camera_position = Vec3f::from(camera.position().translation());
        // Matrix4f::from(*perspective.as_matrix())
        // perspective_matrix2(&frame),
        let uniforms = uniform! {
            camera_position: &camera_position,
            perspective: perspective_matrix2(&frame),
            view: view,
            skybox: cubemap.sampled().magnify_filter(MagnifySamplerFilter::Linear),
        };

        try!(frame.draw(vertex_buffer,
                  index_buffer,
                  program,
                  &uniforms,
                  draw_parameters)
            .chain_err(|| "Could not render skybox."));

        Ok(())
    }

    #[inline]
    fn surface_for_face(&self, window: &Window, face: CubeLayer) -> Result<SimpleFrameBuffer> {
        SimpleFrameBuffer::new(window.facade(), self.cubemap.main_level().image(face))
            .chain_err(|| format!("Could not create a framebuffer for {:?}", face))
    }
}

#[inline]
fn perspective_matrix(aspect: GpuScalar) -> PerspectiveMatrix3<GpuScalar> {
    let aspect = aspect;
    let fov = 3.141592 / 3.0;
    let zfar = 10.0;
    let znear = 0.1;
    PerspectiveMatrix3::new(aspect, fov, znear, zfar)
}

fn perspective_matrix2(frame: &Frame) -> [[f32; 4]; 4] {
    let (width, height) = frame.get_dimensions();
    let aspect_ratio = height as f32 / width as f32;

    let fov: f32 = 3.141592 / 3.0;
    let zfar = 10.0;
    let znear = 0.1;

    let f = 1.0 / (fov / 2.0).tan();

    [[f * aspect_ratio, 0.0, 0.0, 0.0],
     [0.0, f, 0.0, 0.0],
     [0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
     [0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0]]
}

#[inline]
fn frame_aspect(frame: &Frame) -> GpuScalar {
    let (width, height) = frame.get_dimensions();
    height as f32 / width as f32
}

const VERTEX_SHADER: &'static str = "src/gfx/shaders/skybox.vert";
const FRAGMENT_SHADER: &'static str = "src/gfx/shaders/skybox.frag";

#[cfg_attr(rustfmt, rustfmt_skip)]
const SKYBOX_VERTICES: [[f32; 3]; 36] = [
    [-1.0,  1.0, -1.0],
    [-1.0, -1.0, -1.0],
    [ 1.0, -1.0, -1.0],
    [ 1.0, -1.0, -1.0],
    [ 1.0,  1.0, -1.0],
    [-1.0,  1.0, -1.0],

    [-1.0, -1.0,  1.0],
    [-1.0, -1.0, -1.0],
    [-1.0,  1.0, -1.0],
    [-1.0,  1.0, -1.0],
    [-1.0,  1.0,  1.0],
    [-1.0, -1.0,  1.0],

    [ 1.0, -1.0, -1.0],
    [ 1.0, -1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0,  1.0, -1.0],
    [ 1.0, -1.0, -1.0],

    [-1.0, -1.0,  1.0],
    [-1.0,  1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0, -1.0,  1.0],
    [-1.0, -1.0,  1.0],

    [-1.0,  1.0, -1.0],
    [ 1.0,  1.0, -1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [-1.0,  1.0,  1.0],
    [-1.0,  1.0, -1.0],

    [-1.0, -1.0, -1.0],
    [-1.0, -1.0,  1.0],
    [ 1.0, -1.0, -1.0],
    [ 1.0, -1.0, -1.0],
    [-1.0, -1.0,  1.0],
    [ 1.0, -1.0,  1.0],
];

#[cfg_attr(rustfmt,rustfmt_skip)]
const SKYBOX_INDICES:[u32; 36] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
                                  12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                                  24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35];
