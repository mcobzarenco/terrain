use std::collections::{HashSet, HashMap};
use std::sync::Arc;

use glium::{self, Frame, DrawParameters, Program, Surface};
use nalgebra::{Eye, Norm, Matrix4, Isometry3, Translation, Point3, Rotation, Vector3};
use ncollide::shape::{Ball, ShapeHandle};
use nphysics3d::object::{RigidBody, RigidBodyHandle};
use nphysics3d::volumetric::Volumetric;
use nphysics3d::world::World;
use noise::{self, Seed, Brownian3};
use threadpool::ThreadPool;

use errors::{ChainErr, Result};
use game::Player;
use gfx::{Camera, LevelOfDetail, Window};
use math::{CpuScalar, Matrix4f, Vec3f, ScalarField};
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
            base_radius: 0.5e4,
            landscape_deviation: 0.15,
            num_octaves: 5,
            persistence: 0.8,
            wavelength: 1.7,
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
    fn value_at(&self, x: CpuScalar, y: CpuScalar, z: CpuScalar) -> CpuScalar {
        assert!(x.is_finite() && y.is_finite() && z.is_finite(),
                format!("{} {} {}", x, y, z));
        let PlanetField { ref seed, ref spec } = *self;

        let mut position = Vec3f::new(x, y, z);
        let distance = position.norm();
        position.normalize_mut();
        // info!("pos: {:?}", position);

        let mountains = Brownian3::new(noise::open_simplex3, spec.num_octaves)
            .persistence(spec.persistence)
            .wavelength(spec.wavelength)
            .lacunarity(spec.lacunarity);
        let plains = Brownian3::new(noise::open_simplex3, 3)
            .persistence(0.9)
            .wavelength(1.9)
            .lacunarity(1.8);
        let mix = Brownian3::new(noise::open_simplex3, 2).wavelength(2.0);

        let mut perturbation = 0.0;
        let mut alpha = (1.0 + mix.apply(&self.seed, (position * 3.0 + 10.0).as_ref())) / 2.0;
        if alpha > 0.45 && alpha < 0.55 {
            alpha = (alpha - 0.45) * 10.0;
            perturbation = alpha * mountains.apply(&self.seed, (position * 4.0).as_ref()) +
                           (1.0 - alpha) * plains.apply(&self.seed, (position * 2.0).as_ref());
        } else if alpha < 0.45 {
            perturbation = plains.apply(&self.seed, (position * 2.0).as_ref());
        } else {
            perturbation = mountains.apply(&self.seed, (position * 4.0).as_ref());
        }

        let radius = spec.base_radius + spec.landscape_deviation * spec.base_radius * perturbation;
        distance - radius
        // y

        // y - (x * x + z * z).sqrt().sin()
    }
}

pub struct PlanetRenderer<'a, 'b, Field: ScalarField> {
    lod: LevelOfDetail<'a, Field>,
    physics_world: World<CpuScalar>,
    physics_chunks: HashMap<usize, RigidBodyHandle<CpuScalar>>,
    draw_parameters: DrawParameters<'b>,
    program: Program,
    scalar_field: Arc<Field>,
    pub player: Player,
}

impl<'a, 'b, Field> PlanetRenderer<'a, 'b, Field>
    where Field: 'static + ScalarField + Send + Sync
{
    pub fn new(scalar_field: Field, window: &Window, thread_pool: &'a ThreadPool) -> Result<Self> {

        let vertex_shader = try!(read_utf8_file(VERTEX_SHADER));
        let fragment_shader = try!(read_utf8_file(FRAGMENT_SHADER));
        let program = try!(glium::Program::from_source(window.facade(),
                                                       &vertex_shader,
                                                       &fragment_shader,
                                                       None)
            .chain_err(|| "Could not compile the shaders."));

        let scalar_field = Arc::new(scalar_field);
        let lod = LevelOfDetail::new(scalar_field.clone(), thread_pool, 10, 16.0, 32768.0, 10);

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        let mut physics_world = World::new();
        let ball = ShapeHandle::new(Ball::new(3.0f32));
        let ball_mass = 80.0;
        let props = Some((ball_mass, ball.center_of_mass(), ball.angular_inertia(ball_mass)));
        let player_handle = physics_world.add_rigid_body(RigidBody::new(ball, props, 0.01, 2.0));
        let player = Player::new(player_handle,
                                 &(Point3::new(1.0, 1.0, 1.0) * 0.5e4),
                                 &Point3::new(0.0, 0.0, 0.0),
                                 &Vector3::y());

        Ok(PlanetRenderer {
            lod: lod,
            physics_world: physics_world,
            physics_chunks: HashMap::new(),
            draw_parameters: params,
            program: program,
            scalar_field: scalar_field,
            player: player,
        })
    }

    pub fn render(&mut self,
                  window: &Window,
                  frame: &mut Frame,
                  camera: &mut Camera)
                  -> Result<()> {
        let PlanetRenderer { ref program,
                             ref draw_parameters,
                             ref mut lod,
                             ref mut physics_world,
                             ref mut physics_chunks,
                             ref mut player,
                             .. } = *self;

        physics_world.set_gravity(player.observer.translation().normalize() * -9.60);
        // let new_camera = camera.position().translation() + player.position().translation() / 2.0;
        // camera.observer_mut().set_translation(new_camera);

        // let speed = player.player.borrow().lin_vel();
        // if speed.norm() > 6.0 {
        //     player.player.borrow_mut().set_lin_vel(speed.normalize());
        // }

        // player.borrow_mut().set_rotation(camera.position().rotation());
        // physics_world.deferred_set_position(0, camera.position());
        player.update_position();

        let view = player.view_matrix();
        let light = Vec3f::new(-40.0f32, 0.0, -1.1e4);
        let uniforms = uniform! {
            perspective: PlanetRenderer::<Field>::perspective_matrix(frame),
            model: PlanetRenderer::<Field>::model_matrix(),
            view: view,
            u_light: &light,
        };

        let screen_chunks = try!(lod.update(window, camera));

        let mut remove_set: HashSet<usize> = physics_chunks.keys().map(|x| *x).collect();

        // {
        //     let c1: HashSet<_> = physics_chunks.keys().collect();
        //     let c2: HashSet<_> = screen_chunks.iter().map(|x| x.uid).collect();

        //     info!("initial physics_chunks {:?}", c1);
        //     info!("screen chunks {:?}", c2);
        // }

        for chunk in screen_chunks.into_iter() {
            try!(frame.draw(&chunk.vertex_buffer,
                      &chunk.index_buffer,
                      program,
                      &uniforms,
                      draw_parameters)
                .chain_err(|| "Could not render frame."));

            if !physics_chunks.contains_key(&chunk.uid) {
                let handle = physics_world.add_rigid_body(RigidBody::new(
                    chunk.tri_mesh.clone(),
                    None,
                    0.1,
                    1.0));
                physics_chunks.insert(chunk.uid, handle);
            }
            remove_set.remove(&chunk.uid);
        }
        for uid in remove_set.into_iter() {
            physics_world.remove_rigid_body(&physics_chunks[&uid]);
            physics_chunks.remove(&uid);
        }

        // info!("Camera: {:?}", camera.position().translation());

        Ok(())
    }

    pub fn update_physics(&mut self, delta_time: f32) {
        self.physics_world.step(delta_time);
    }

    fn model_matrix() -> Matrix4f {
        Matrix4f::from(Matrix4::new_identity(4))
    }

    fn perspective_matrix(frame: &Frame) -> [[f32; 4]; 4] {
        let (width, height) = frame.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;

        let fov: f32 = 3.141592 / 3.0;
        let zfar = 1e4;
        let znear = 0.1;

        let f = 1.0 / (fov / 2.0).tan();

        [[f * aspect_ratio, 0.0, 0.0, 0.0],
         [0.0, f, 0.0, 0.0],
         [0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
         [0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0]]
    }
}

const VERTEX_SHADER: &'static str = "src/gfx/shaders/planet.vert";
const FRAGMENT_SHADER: &'static str = "src/gfx/shaders/planet.frag";
