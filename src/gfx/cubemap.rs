use std::path::Path;
use std::fmt::Debug;
use glium::texture::RawImage2d;
use image;

use errors::*;

pub struct CubemapRenderer {}

impl CubemapRenderer {
    pub fn new() -> Self {
        CubemapRenderer {}
    }

    pub fn load<P>(path: &P) -> Result<()>
        where P: AsRef<Path> + Debug
    {
        let image = try!(image::open(path)
                .chain_err(|| format!("Could not load image at {:?}", path)))
            .to_rgba();
        let image_dimensions = image.dimensions();
        let image = RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
        Ok(())
    }
}
