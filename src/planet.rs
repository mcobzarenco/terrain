use std::collections::HashSet;
use std::sync::Arc;

use glium::{self, Frame, DrawParameters, Program, Surface};
use glium::backend::glutin_backend::GlutinFacade;
use nalgebra::{Eye, Norm, Matrix4, Isometry3, Translation};
use ncollide::shape::{Ball, ShapeHandle};
use ncollide::world::{CollisionWorld3, CollisionGroups, GeometricQueryType, CollisionObject3};
use noise::{self, Seed, Brownian3};
use num::One;
use threadpool::ThreadPool;

use errors::{ChainErr, Result};
use gfx::{Camera, LevelOfDetail};
use gfx::lod::ChunkId;
use math::{GpuScalar, Matrix4f, Vec3f, ScalarField};
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
        position.normalize_mut();

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
        let mut alpha = (1.0 + mix.apply(&self.seed, (position * 3.0 + 10.0).as_ref())) / 2.0;
        let u = spec.landscape_deviation * spec.base_radius * 0.01;
        if alpha > 0.45 && alpha < 0.55 {
            alpha = (alpha - 0.45) * 10.0;
            perturbation = alpha * (mountains.apply(&self.seed, (position * 4.0).as_ref()) + u) +
                           (1.0 - alpha) * plains.apply(&self.seed, position.as_ref());
        } else if alpha < 0.45 {
            perturbation = plains.apply(&self.seed, position.as_ref());
        } else {
            perturbation = mountains.apply(&self.seed, (position * 4.0).as_ref()) + u;
        }

        let radius = spec.base_radius + spec.landscape_deviation * spec.base_radius * perturbation;
        // distance - radius
        y

        // y - (x * x + z * z).sqrt().sin()
    }
}

const COLLIDE_WORLD_GROUP: usize = 0;
const COLLIDE_PLAYER_GROUP: usize = 1;

pub struct PlanetRenderer<'a, 'b, Field: ScalarField> {
    lod: LevelOfDetail<'a, Field>,
    collision_world: CollisionWorld3<GpuScalar, ()>,
    collision_set: HashSet<usize>,
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
        let lod = LevelOfDetail::new(scalar_field.clone(), thread_pool, facade, 6, 16.0, 16.0, 10);

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        let mut collision_world = CollisionWorld3::new(0.01, false);
        let ball = ShapeHandle::new(Ball::new(4.0f32));
        let mut camera_group = CollisionGroups::new();
        camera_group.set_membership(&[COLLIDE_PLAYER_GROUP]);
        collision_world.deferred_add(0,
                                     Isometry3::one(),
                                     ball,
                                     camera_group,
                                     GeometricQueryType::Contacts(0.0),
                                     ());
        collision_world.update();

        Ok(PlanetRenderer {
            lod: lod,
            collision_world: collision_world,
            collision_set: HashSet::new(),
            draw_parameters: params,
            program: program,
            scalar_field: scalar_field,
        })
    }

    pub fn render(&mut self, frame: &mut Frame, camera: &mut Camera) -> Result<()> {
        let PlanetRenderer { ref program,
                             ref draw_parameters,
                             ref mut lod,
                             ref mut collision_world,
                             ref mut collision_set,
                             .. } = *self;

        collision_world.deferred_set_position(0, camera.position());

        let view = camera.view_matrix();
        let light = Vec3f::new(-40.0f32, 0.0, -60.0);
        let uniforms = uniform! {
            perspective: PlanetRenderer::<Field>::perspective_matrix(frame),
            model: PlanetRenderer::<Field>::model_matrix(),
            view: view,
            u_light: &light,
        };

        let screen_chunks = try!(lod.update(camera));
        let contacts_query = GeometricQueryType::Contacts(0.0);
        let mut world_group = CollisionGroups::new();
        world_group.set_membership(&[COLLIDE_WORLD_GROUP]);
        world_group.set_whitelist(&[COLLIDE_PLAYER_GROUP]);

        let mut remove_set = collision_set.clone();
        for chunk in screen_chunks.into_iter() {
            try!(frame.draw(&chunk.vertex_buffer,
                      &chunk.index_buffer,
                      program,
                      &uniforms,
                      draw_parameters)
                .chain_err(|| "Could not render frame."));

            if !collision_set.contains(&chunk.uid) {
                info!("adding collision chunk {}", chunk.uid);
                collision_world.deferred_add(chunk.uid,
                                             Isometry3::one(),
                                             chunk.tri_mesh.clone(),
                                             world_group,
                                             contacts_query,
                                             ());
                collision_set.insert(chunk.uid);
                remove_set.remove(&chunk.uid);
            }
        }
        // info!("collision_set {:?}", collision_set);
        // info!("remove_set {:?}", remove_set);

        // for uid in remove_set.into_iter() {
        //     info!("removing collision chunk {}", uid);
        //     collision_world.deferred_remove(uid);
        //     collision_set.remove(&uid);
        // }

        collision_world.update();
        for (p1, p2, c) in collision_world.contacts() {
            if p1.uid == 0 {
                camera.observer_mut().append_translation_mut(&(c.normal * c.depth));
            } else if p2.uid == 0 {
                camera.observer_mut().append_translation_mut(&(c.normal * c.depth));
            }

            // info!("p1.uid: {:?} | p2.uid: {:?}", p1.uid, p2.uid);
            // info!("c: {:?}", c);
        }

        info!("Camera: {:?}", camera.position().translation());

        Ok(())
    }

    fn model_matrix() -> Matrix4f {
        Matrix4f::from(Matrix4::new_identity(4))
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
