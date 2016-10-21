#![recursion_limit = "1024"]

#[macro_use]
extern crate chan;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate custom_derive;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate glium;
extern crate image;
#[macro_use]
extern crate log;
extern crate lru_time_cache;
extern crate itertools;
extern crate nalgebra;
extern crate ncollide;
#[macro_use]
extern crate newtype_derive;
extern crate noise;
extern crate nphysics3d;
extern crate num;
extern crate rand;
extern crate rayon;
extern crate threadpool;
extern crate wavefront_obj;

mod errors;
mod game;
mod gfx;
mod math;
mod utils;
mod planet;

use std::error::Error;
use clap::Arg;
use rand::Rng;

use errors::Result;
use gfx::App;
use planet::{PlanetField, PlanetSpec};

fn start_app() -> Result<()> {
    let matches = clap::App::new("Rusty Terrain.")
        .version("0.1.0")
        .author("Marius C. <marius@reinfer.io>")
        .about("A voxel based planet generator.")
        .arg(Arg::with_name("base_radius")
            .long("base-radius")
            .value_name("f32")
            .takes_value(true))
        .arg(Arg::with_name("deviation")
            .long("deviation")
            .value_name("f32")
            .takes_value(true))
        .arg(Arg::with_name("num_octaves")
            .long("num-octaves")
            .value_name("usize")
            .takes_value(true))
        .arg(Arg::with_name("persistence")
            .long("persistence")
            .value_name("f32")
            .takes_value(true))
        .arg(Arg::with_name("wavelength")
            .long("wavelength")
            .value_name("f32")
            .takes_value(true))
        .arg(Arg::with_name("lacunarity")
            .long("lacunarity")
            .value_name("f32")
            .takes_value(true))
        .arg(Arg::with_name("width")
            .long("width")
            .value_name("u32")
            .takes_value(true))
        .arg(Arg::with_name("height")
            .long("height")
            .value_name("u32")
            .takes_value(true))
        .get_matches();

    let mut planet_spec = PlanetSpec::default();
    if matches.is_present("base_radius") {
        value_t!(matches, "base_radius", f32).map(|v| planet_spec.base_radius = v).unwrap();
    }
    if matches.is_present("deviation") {
        value_t!(matches, "deviation", f32).map(|v| planet_spec.landscape_deviation = v).unwrap();
    }
    if matches.is_present("num_octaves") {
        value_t!(matches, "num_octaves", usize).map(|v| planet_spec.num_octaves = v).unwrap();
    }
    if matches.is_present("persistence") {
        value_t!(matches, "persistence", f32).map(|v| planet_spec.persistence = v).unwrap();
    }
    if matches.is_present("wavelength") {
        value_t!(matches, "wavelength", f32).map(|v| planet_spec.wavelength = v).unwrap();
    }
    if matches.is_present("lacunarity") {
        value_t!(matches, "lacunarity", f32).map(|v| planet_spec.lacunarity = v).unwrap();
    }

    let mut width = 1024;
    let mut height = 768;
    if matches.is_present("width") {
        value_t!(matches, "width", u32).map(|v| width = v).unwrap();
    }
    if matches.is_present("height") {
        value_t!(matches, "height", u32).map(|v| height = v).unwrap();
    }

    let mut rng = rand::thread_rng();
    let seed: u32 = rng.gen();
    info!("The world seed is {}", seed);
    info!("Generating planet with params {:?}", planet_spec);
    let field = PlanetField::new(seed, planet_spec);

    info!("Creating app");
    let mut app = try!(App::new(width, height, 3));
    app.run(field)
}

fn main() {
    if let Err(err) = env_logger::init() {
        println!("Could not initialize logger, exiting: {}",
                 err.description());
    } else {
        start_app().unwrap();
    }
}
