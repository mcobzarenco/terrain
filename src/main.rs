#![recursion_limit = "1024"]

extern crate env_logger;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate chan;
#[macro_use]
extern crate glium;
#[macro_use]
extern crate log;
extern crate lru_time_cache;
extern crate itertools;
extern crate noise;
extern crate num;
extern crate rand;
extern crate rayon;
extern crate wavefront_obj;
extern crate threadpool;

mod errors;
mod gfx;
mod math;
mod utils;
mod planet;

use std::error::Error;

use gfx::App;

fn main() {
    if let Err(err) = env_logger::init() {
        println!("Could not initialize logger, exiting: {}",
                 err.description());
    } else {
        let mut app = App::new(4).unwrap();
        app.run().unwrap();
    }
}
