use glium::{DisplayBuild, Frame, Program, Surface};
use glium::glutin::{CursorState, WindowBuilder};
use glium::backend::glutin_backend::GlutinFacade;

use errors::{Result, ChainErr};
use math::GpuScalar;
use utils::read_utf8_file;

pub const GLSL_VERSION_STRING: &'static str = "330 core";

pub struct Window {
    facade: GlutinFacade,
    width: u32,
    height: u32,
}

impl Window {
    pub fn new<'a>(width: u32, height: u32, title: &str) -> Result<Window> {
        let facade = try!(WindowBuilder::new()
            .with_title(title)
            .with_dimensions(width, height)
            .with_depth_buffer(24)
            .build_glium()
            .chain_err(|| "Could not create a Glutin window."));

        if let Some(win) = facade.get_window() {
            try!(win.set_cursor_state(CursorState::Hide));
        }

        Ok(Window {
            facade: facade,
            width: width,
            height: height,
        })
    }

    pub fn size(&self) -> WindowInnerSize {
        let (width, height) = self.facade
            .get_window()
            .expect("Could not get a reference to the current window; no window?")
            .get_inner_size_pixels()
            .expect("Could not get the size of the window.");
        WindowInnerSize {
            width: width,
            height: height,
        }
    }

    pub fn aspect(&self) -> GpuScalar {
        let WindowInnerSize { width, height } = self.size();
        height as GpuScalar / width as GpuScalar
    }

    pub fn draw(&self) -> Frame {
        let mut frame = self.facade.draw();
        frame.clear_all(BACKGROUND_COLOR, 1.0, 0);
        frame
    }

    pub fn facade(&self) -> &GlutinFacade {
        &self.facade
    }

    pub fn program(&self, vertex_src: &str, fragment_src: &str) -> Result<Program> {
        Program::from_source(&self.facade,
                             &format!("#version {}\n{}",
                                      GLSL_VERSION_STRING,
                                      try!(read_utf8_file(vertex_src)
                                          .chain_err(|| "Failed to read vertex shader."))),
                             &format!("#version {}\n{}",
                                      GLSL_VERSION_STRING,
                                      try!(read_utf8_file(fragment_src)
                                          .chain_err(|| "Failed to read fragment shader."))),
                             None)
            .chain_err(|| "Failed to build program.")
    }
}

pub struct WindowInnerSize {
    pub width: u32,
    pub height: u32,
}

impl WindowInnerSize {
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

const BACKGROUND_COLOR: (f32, f32, f32, f32) = (0.0, 0.0, 0.0, 1.0);
