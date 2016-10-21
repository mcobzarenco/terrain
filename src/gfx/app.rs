use std::time::Instant;

use nalgebra::{Rotation, Translation};
use threadpool::ThreadPool;

use errors::{ChainErr, Result};
use gfx::{Camera, Gesture, Input, KeyCode, SkyboxRenderer, Window};
use math::{Point3f, Vec3f};
use planet::{PlanetField, PlanetRenderer};

pub struct App {
    window: Window,
    input: Input,
    camera: Camera,
    thread_pool: ThreadPool,
}

impl App {
    pub fn new(width: u32, height: u32, num_workers: usize) -> Result<Self> {
        let mut window = try!(Window::new(width, height, "Rusty Terrain"));
        let input = try!(Input::new(&mut window));
        Ok(App {
            window: window,
            input: input,
            camera: Camera::new(Point3f::new(0.0, 0.0, 0.0),
                                Point3f::new(0.0, 0.0, 1.0),
                                Vec3f::new(0.0, 1.0, 0.0)),
            thread_pool: ThreadPool::new(num_workers),
        })
    }

    pub fn run(&mut self, planet_field: PlanetField) -> Result<()> {
        let App { ref mut input, ref thread_pool, ref mut window, .. } = *self;
        let mut planet = try!(PlanetRenderer::new(planet_field, window, thread_pool));
        let mut skybox = try!(SkyboxRenderer::new(window));
        try!(skybox.load(window, "/home/marius/w/terrain/assets/skybox-galaxy.jpg"));
        info!("Loaded the skybox.");

        let quit_gesture = Gesture::AnyOf(vec![Gesture::QuitTrigger,
                                               Gesture::KeyDownTrigger(KeyCode::Escape)]);

        info!("Entering main loop.");
        let mut running = true;
        while running {
            let time = Instant::now();

            let mut target = window.draw();

            let player_pos = planet.player.update_position();
            self.camera.observer_mut().set_translation(player_pos.translation());
            self.camera.observer_mut().set_rotation(player_pos.rotation());

            try!(skybox.render(&mut target, &mut self.camera));
            try!(planet.render(window, &mut target, &mut self.camera));
            try!(target.finish()
                .chain_err(|| "Could not render frame."));

            let elapsed = time.elapsed();
            let delta = elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 * 1e-9;
            planet.update_physics(delta);

            try!(input.update(window));
            if input.poll_gesture(&quit_gesture) {
                info!("Quit gesture detected, exiting...");
                running = false;
            }
            planet.player.update(delta, input);
        }
        Ok(())
    }
}
