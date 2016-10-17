use std::time::Instant;

use nalgebra::{Rotation, Translation};
use glium::glutin::{CursorState, Event};
use threadpool::ThreadPool;

use errors::{ChainErr, Result};
use gfx::{Camera, SkyboxRenderer, Window};
use math::{Point3f, Vec3f};
use planet::{PlanetField, PlanetRenderer};

pub struct App {
    window: Window,
    camera: Camera,
    thread_pool: ThreadPool,
}

impl App {
    pub fn new(width: u32, height: u32, num_workers: usize) -> Result<Self> {
        let window = try!(Window::new(width, height, "Rusty Terrain"));
        if let Some(win) = window.facade().get_window() {
            try!(win.set_cursor_state(CursorState::Hide));
        }

        Ok(App {
            window: window,
            camera: Camera::new(Point3f::new(0.0, 0.0, 0.0),
                                Point3f::new(0.0, 0.0, 1.0),
                                Vec3f::new(0.0, 1.0, 0.0)),
            thread_pool: ThreadPool::new(num_workers),
        })
    }

    pub fn run(&mut self, planet_field: PlanetField) -> Result<()> {
        let App { ref thread_pool, ref window, .. } = *self;
        let mut planet = try!(PlanetRenderer::new(planet_field, window, thread_pool));
        let mut skybox = try!(SkyboxRenderer::new(window));
        try!(skybox.load("/home/marius/w/terrain/assets/skybox-galaxy.jpg"));
        info!("loaded assests");


        let mut num = 0;
        let mut cum_delta = 0.0;
        loop {
            let time = Instant::now();

            let mut target = window.draw();

            let player_pos = planet.player.update_position();
            self.camera.observer_mut().set_translation(player_pos.translation());
            self.camera.observer_mut().set_rotation(player_pos.rotation());

            try!(skybox.render(&mut target, &mut self.camera));
            try!(planet.render(&mut target, &mut self.camera));
            try!(target.finish()
                .chain_err(|| "Could not render frame."));

            let elapsed = time.elapsed();
            let delta = elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 * 1e-9;
            planet.update_physics(delta);

            num += 1;
            cum_delta += delta;
            if num % 100 == 0 {
                warn!("FPS: {:?}", num as f32 / cum_delta);
                let mut num = 0;
                let mut cum_delta = 0.0;
            }

            for event in window.facade().poll_events() {
                match event {
                    Event::Closed => return Ok(()),
                    _ => planet.player.update(delta, window, event),
                }
                // _ => self.camera.update(delta, &window.facade().get_window().unwrap(), event),

            }
            // info!("camera position : {:?}", self.camera.position());
        }
    }
}
