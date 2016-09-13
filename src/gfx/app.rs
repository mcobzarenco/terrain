use std::time::Instant;

use glium::{self, DisplayBuild, IndexBuffer, Surface, VertexBuffer};
use glium::glutin::{CursorState, Event, WindowBuilder};
use glium::backend::glutin_backend::GlutinFacade;
use rand::{self, Rng};
use threadpool::ThreadPool;

use errors::{ChainErr, Result};
use utils::read_utf8_file;
use math::{Vector, Vec3f};
use super::camera::Camera;
use super::marching_cubes::marching_cubes;

use planet::{TerrainField, Planet};

pub struct App {
    facade: GlutinFacade,
    camera: Camera,
    thread_pool: ThreadPool,
}

impl App {
    pub fn new(num_workers: usize) -> Result<Self> {
        let facade = try!(WindowBuilder::new()
            .with_title("Rusty Terrain")
            .with_depth_buffer(24)
            .build_glium()
            .chain_err(|| "Could not create a Glutin window."));

        if let Some(win) = facade.get_window() {
            try!(win.set_cursor_state(CursorState::Hide));
        }

        Ok(App {
            facade: facade,
            camera: Camera::new(),
            thread_pool: ThreadPool::new(num_workers),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // let ref cube = try!(mesh::load_mesh_from_file("assets/teapot.obj")
        //     .chain_err(|| "Couldn't load asset."))[0];

        let mut rng = rand::thread_rng();
        let x: u32 = rng.gen();
        info!("world_seed = {}", x);
        let field = TerrainField::new(x);
        let mut planet = try!(Planet::new(field, &self.facade, &self.thread_pool));

        loop {
            let mut target = self.facade.draw();
            target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

            let time = Instant::now();
            try!(planet.render(&mut target, &self.camera));
            try!(target.finish()
                .chain_err(|| "Could not render frame."));

            let elapsed = time.elapsed();
            let delta = elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 * 1e-9;

            for event in self.facade.poll_events() {
                match event {
                    Event::Closed => return Ok(()),
                    _ => self.camera.update(delta, &self.facade.get_window().unwrap(), event),
                }
            }
        }
    }
}
