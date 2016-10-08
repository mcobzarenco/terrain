use std::sync::Arc;

use glium::{self, Frame, DrawParameters, Program, Surface};
use glium::backend::glutin_backend::GlutinFacade;
use noise::{self, Seed, Brownian3};
use threadpool::ThreadPool;

use errors::{ChainErr, Result};
use gfx::{Camera, LevelOfDetail};
use math::{Vec3f, Vector, ScalarField};
use utils::read_utf8_file;


#[derive(Clone, Debug)]
pub struct PlanetSpec {
    pub base_radius: f32,
    pub landscape_deviation: f32,
    pub num_octaves: usize,
    pub persistence: f32,
    pub wavelength: f32,
    pub lacunarity: f32,
}

impl Default for PlanetSpec {
    fn default() -> Self {
        PlanetSpec {
            base_radius: 32.0,
            landscape_deviation: 0.4,
            num_octaves: 12,
            persistence: 0.8,
            wavelength: 7.0,
            lacunarity: 1.91,
        }
    }
}

pub struct PlanetField {
    seed: Seed,
    spec: PlanetSpec,
}

impl PlanetField {
    pub fn new(seed: u32, planet_spec: PlanetSpec) -> Self {
        PlanetField {
            seed: Seed::new(seed),
            spec: planet_spec,
        }
    }
}

impl ScalarField for PlanetField {
    #[inline]
    fn value_at(&self, x: f32, y: f32, z: f32) -> f32 {
        let PlanetField { ref seed, ref spec } = *self;

        let mut position = Vec3f::new(x, y, z);
        let distance = position.norm();
        position.normalize();

        let mountains = Brownian3::new(noise::open_simplex3, spec.num_octaves)
            .persistence(spec.persistence)
            .wavelength(spec.wavelength)
            .lacunarity(spec.lacunarity);
        let plains = Brownian3::new(noise::open_simplex3, 3)
            .persistence(0.9)
            .wavelength(3.0)
            .lacunarity(1.8);
        let mix = Brownian3::new(noise::open_simplex3, 2).wavelength(2.0);

        let mut perturbation = 0.0;
        let mut alpha = (1.0 + mix.apply(&self.seed, &(position * 3.0 + 10.0).array())) / 2.0;
        let u = spec.landscape_deviation * spec.base_radius * 0.01;
        if alpha > 0.45 && alpha < 0.55 {
            alpha = (alpha - 0.45) * 10.0;
            perturbation = alpha * (mountains.apply(&self.seed, &(position * 4.0).array()) + u) +
                           (1.0 - alpha) * plains.apply(&self.seed, &(position).array());
        } else if alpha < 0.45 {
            perturbation = plains.apply(&self.seed, &(position).array());
        } else {
            perturbation = mountains.apply(&self.seed, &(position * 4.0).array()) + u;
        }

        let radius = spec.base_radius + spec.landscape_deviation * spec.base_radius * perturbation;
        (distance - radius)
    }
}

pub struct PlanetRenderer<'a, 'b, Field: ScalarField> {
    lod: LevelOfDetail<'a, Field>,
    draw_parameters: DrawParameters<'b>,
    program: Program,
    scalar_field: Arc<Field>,
}

impl<'a, 'b, Field> PlanetRenderer<'a, 'b, Field>
    where Field: 'static + ScalarField + Send + Sync
{
    pub fn new(scalar_field: Field,
               facade: &'a GlutinFacade,
               thread_pool: &'a ThreadPool)
               -> Result<Self> {

        let vertex_shader = try!(read_utf8_file(VERTEX_SHADER));
        let fragment_shader = try!(read_utf8_file(FRAGMENT_SHADER));
        let program =
            try!(glium::Program::from_source(facade, &vertex_shader, &fragment_shader, None)
                .chain_err(|| "Could not compile the shaders."));

        let scalar_field = Arc::new(scalar_field);
        let lod = LevelOfDetail::new(scalar_field.clone(), thread_pool, facade, 16, 16.0, 16.0);

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        Ok(PlanetRenderer {
            lod: lod,
            draw_parameters: params,
            program: program,
            scalar_field: scalar_field,
        })
    }

    pub fn render(&mut self, frame: &mut Frame, camera: &Camera) -> Result<()> {
        let view = camera.view_matrix();

        let light = [-40.0f32, 0.0, -60.0];

        let uniforms = uniform! {
            perspective: PlanetRenderer::<Field>::perspective_matrix(frame),
            model: PlanetRenderer::<Field>::model_matrix(),
            view: view,
            u_light: light,
        };

        let program = &self.program;
        let draw_parameters = &self.draw_parameters;
        try!(self.lod.update(camera, |vertex_buffer, index_buffer| {
            frame.draw(vertex_buffer,
                      index_buffer,
                      &program,
                      &uniforms,
                      &draw_parameters)
                .chain_err(|| "Could not render frame.")
        }));

        Ok(())
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn model_matrix() -> [[f32; 4]; 4] {
        [[1.0, 0.0, 0.0, 0.0],
         [0.0, 1.0, 0.0, 0.0],
         [0.0, 0.0, 1.0, 0.0],
         [0.0, 0.0, 0.0, 1.0f32]]
    }

    fn perspective_matrix(frame: &Frame) -> [[f32; 4]; 4] {
        let (width, height) = frame.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;

        let fov: f32 = 3.141592 / 3.0;
        let zfar = 512.0;
        let znear = 1e-5;

        let f = 1.0 / (fov / 2.0).tan();

        [[f * aspect_ratio, 0.0, 0.0, 0.0],
         [0.0, f, 0.0, 0.0],
         [0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
         [0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0]]
    }
}

const VERTEX_SHADER: &'static str = "src/gfx/shaders/planet.vert";
const FRAGMENT_SHADER: &'static str = "src/gfx/shaders/planet.frag";
