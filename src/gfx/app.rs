use std::time::Instant;

use glium::{DisplayBuild, Surface};
use glium::glutin::{CursorState, Event, WindowBuilder};
use glium::backend::glutin_backend::GlutinFacade;
use threadpool::ThreadPool;

use errors::{ChainErr, Result};
use super::camera::Camera;

use planet::{PlanetField, PlanetRenderer};


pub struct App {
    facade: GlutinFacade,
    camera: Camera,
    thread_pool: ThreadPool,
}

impl App {
    pub fn new(width: u32, height: u32, num_workers: usize) -> Result<Self> {
        let facade = try!(WindowBuilder::new()
            .with_title("Rusty Terrain")
            .with_dimensions(width, height)
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

    pub fn run(&mut self, planet_field: PlanetField) -> Result<()> {
        let mut planet = try!(PlanetRenderer::new(planet_field, &self.facade, &self.thread_pool));
        // let mut cubemap_renderer = try!(CubemapRenderer::new());

        loop {
            let time = Instant::now();

            let mut target = self.facade.draw();
            target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

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
