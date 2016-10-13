use std::collections::{HashSet, HashMap};
use std::sync::Arc;

use glium::{self, Frame, DrawParameters, Program, Surface};
use glium::backend::glutin_backend::GlutinFacade;
use glium::glutin::{Window, Event, ElementState, VirtualKeyCode};
use nalgebra::{Eye, Norm, Matrix4, Isometry3, Translation, Point3, Rotation, Vector3, Inverse,
               ToHomogeneous};
use ncollide::shape::{Ball, ShapeHandle};
use ncollide::world::{CollisionWorld3, CollisionGroups, GeometricQueryType, CollisionObject3};
use nphysics3d::object::{RigidBody, RigidBodyHandle};
use nphysics3d::volumetric::Volumetric;
use nphysics3d::world::World;
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
            base_radius: 64.0,
            landscape_deviation: 0.4,
            num_octaves: 5,
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
        distance - radius
        // y

        // y - (x * x + z * z).sqrt().sin()
    }
}

pub struct Player {
    player: RigidBodyHandle<GpuScalar>,
    keyboard_speed: GpuScalar,
    mouse_speed: GpuScalar,
    observer: Isometry3<GpuScalar>,
}

impl Player {
    fn new(mut player: RigidBodyHandle<GpuScalar>,
           position: &Point3<GpuScalar>,
           target: &Point3<GpuScalar>,
           up: &Vector3<GpuScalar>)
           -> Self {
        player.borrow_mut().set_translation(position.to_vector());
        player.borrow_mut().set_deactivation_threshold(None);

        player.borrow_mut().set_margin(0.01);
        let observer = Isometry3::new_observer_frame(position, &target, &up);
        Player {
            player: player,
            keyboard_speed: 32.0,
            mouse_speed: 0.04,
            observer: observer,
        }
    }

    pub fn view_matrix(&self) -> Matrix4f {
        Matrix4f::from(self.observer.inverse().unwrap().to_homogeneous())
    }

    pub fn update_position(&mut self) {
        self.observer.set_translation(self.player.borrow().position().translation());
    }

    pub fn update(&mut self, delta_time: f32, window: &Window, event: Event) -> () {
        self.update_position();
        let mut player = self.player.borrow_mut();
        info!("Player's (active={} | {:?}) lin velocity {:?}",
              player.is_active(),
              player.deactivation_threshold(),
              player.lin_vel());

        match event {
            // Handle keyboard
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Key1)) => {
                self.keyboard_speed /= 0.5;
                info!("New keyboard speed: {:?}", self.keyboard_speed);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Key2)) => {
                self.keyboard_speed *= 0.5;
                info!("New keyboard speed: {:?}", self.keyboard_speed);
            }
            Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::W)) |
            Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::S)) |
            Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::A)) |
            Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::D)) => {
                player.clear_forces();
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::W)) => {
                let movement = self.observer.rotation * Vector3::z() * self.keyboard_speed;
                info!("m: {:?}", movement);
                player.append_lin_force(movement);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::S)) => {
                let movement = self.observer.rotation * Vector3::z() * self.keyboard_speed * -1.0;
                player.append_lin_force(movement);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::A)) => {
                let movement = self.observer.rotation * Vector3::x() * self.keyboard_speed * -1.0;
                player.append_lin_force(movement);
            }

            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::D)) => {
                let movement = self.observer.rotation * Vector3::x() * self.keyboard_speed;
                player.append_lin_force(movement);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Space)) => {
                let movement = self.observer.rotation * Vector3::y() * self.keyboard_speed;
                info!("m: {:?}", movement);
                // player.append_lin_force(movement);
                player.apply_central_impulse(movement);
                // player.set_lin_vel(movement);
            }

            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Q)) => {
                let angle = self.observer.rotation * Vector3::z() * delta_time;
                self.observer
                    .rotation
                    .append_rotation_mut(&angle);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::E)) => {
                let angle = self.observer.rotation * Vector3::z() * delta_time * -1.0;
                self.observer
                    .rotation
                    .append_rotation_mut(&angle);
            }

            // Handle mouse
            Event::MouseMoved(x, y) => {
                let (width, height) = window.get_inner_size_pixels().unwrap();
                window.set_cursor_position((width as i32) / 2, (height as i32) / 2).unwrap();

                let horizontal_angle = self.mouse_speed * delta_time *
                                       ((width as f32) / 2.0 - x as f32);
                let vertical_angle = self.mouse_speed * delta_time *
                                     ((height as f32) / 2.0 - y as f32);

                let rotation = self.observer.rotation;

                self.observer
                    .rotation
                    .append_rotation_mut(&(rotation * (Vector3::x() * -1.0) * vertical_angle));
                self.observer
                    .rotation
                    .append_rotation_mut(&(rotation * (Vector3::y() * -1.0) * horizontal_angle));

            }
            _ => (),
        }
    }

    pub fn position(&self) -> Isometry3<GpuScalar> {
        self.observer
    }
}

pub struct PlanetRenderer<'a, 'b, Field: ScalarField> {
    lod: LevelOfDetail<'a, Field>,
    physics_world: World<GpuScalar>,
    physics_chunks: HashMap<usize, RigidBodyHandle<GpuScalar>>,
    draw_parameters: DrawParameters<'b>,
    program: Program,
    scalar_field: Arc<Field>,
    pub player: Player,
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
        let lod = LevelOfDetail::new(scalar_field.clone(),
                                     thread_pool,
                                     facade,
                                     10,
                                     16.0,
                                     32.0,
                                     10);

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
        let ball = ShapeHandle::new(Ball::new(0.01f32));
        let ball_mass = 80.0;
        let props = Some((ball_mass, ball.center_of_mass(), ball.angular_inertia(ball_mass)));
        let player_handle = physics_world.add_rigid_body(RigidBody::new(ball, props, 0.4, 1.0));
        let player = Player::new(player_handle,
                                 &(Point3::new(1.0, 1.0, 1.0) * 40.0),
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

    pub fn render(&mut self, frame: &mut Frame, camera: &mut Camera) -> Result<()> {
        let PlanetRenderer { ref program,
                             ref draw_parameters,
                             ref mut lod,
                             ref mut physics_world,
                             ref mut physics_chunks,
                             ref mut player,
                             .. } = *self;

        physics_world.set_gravity(player.observer.translation().normalize() * -2.60);
        let new_camera = camera.position().translation() + player.position().translation() / 2.0;
        camera.observer_mut().set_translation(new_camera);

        // let speed = player.player.borrow().lin_vel();
        // if speed.norm() > 6.0 {
        //     player.player.borrow_mut().set_lin_vel(speed.normalize());
        // }

        // player.borrow_mut().set_rotation(camera.position().rotation());
        // physics_world.deferred_set_position(0, camera.position());
        player.update_position();

        let view = player.view_matrix();
        let light = Vec3f::new(-40.0f32, 0.0, -60.0);
        let uniforms = uniform! {
            perspective: PlanetRenderer::<Field>::perspective_matrix(frame),
            model: PlanetRenderer::<Field>::model_matrix(),
            view: view,
            u_light: &light,
        };

        let screen_chunks = try!(lod.update(camera));

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
                    0.5,
                    1.0));
                physics_chunks.insert(chunk.uid, handle);
            }
            remove_set.remove(&chunk.uid);
        }
        for uid in remove_set.into_iter() {
            physics_world.remove_rigid_body(&physics_chunks[&uid]);
            physics_chunks.remove(&uid);
        }

        // camera.observer_mut().set_rotation(player.borrow().position().rotation());
        // for (p1, p2, c) in physics_world.contacts() {
        //     if p1.uid == 0 {
        //         camera.observer_mut().append_translation_mut(&(c.normal * c.depth));
        //     } else if p2.uid == 0 {
        //         camera.observer_mut().append_translation_mut(&(c.normal * c.depth));
        //     }

        //     // info!("p1.uid: {:?} | p2.uid: {:?}", p1.uid, p2.uid);
        //     // info!("c: {:?}", c);
        // }

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
