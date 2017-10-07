use glium::{DisplayBuild, Frame, Program, Surface};
use glium::glutin::{CursorState, WindowBuilder};
use glium::backend::glutin_backend::{GlutinFacade, WinRef as GlutinWindow};

use errors::{Result, ChainErr, ErrorKind};
use math::GpuScalar;
use utils::read_utf8_file;

pub const GLSL_VERSION_STRING: &'static str = "330 core";

pub struct Window {
    facade: GlutinFacade,
}

impl Window {
    pub fn new<'a>(width: u32, height: u32, title: &str) -> Result<Window> {
        let facade = try!(
            WindowBuilder::new()
                .with_title(title)
                .with_dimensions(width, height)
                .with_depth_buffer(24)
                .build_glium()
                .chain_err(|| "Could not create a Glutin window.")
        );

        Ok(Window { facade: facade })
    }

    pub fn size(&self) -> WindowInnerSize {
        let (width, height) = self.facade
            .get_window()
            .expect(
                "Could not get a reference to the current window; no window?",
            )
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

    pub fn set_cursor_state(&mut self, cursor_state: CursorState) -> Result<()> {
        let glutin_window = try!(self.glutin_window());
        try!(glutin_window.set_cursor_state(cursor_state));
        Ok(())
    }

    pub fn set_cursor_position(&mut self, x: i32, y: i32) -> Result<()> {
        let glutin_window = try!(self.glutin_window());
        if let Err(_) = glutin_window.set_cursor_position(x, y) {
            Err(ErrorKind::SetCursorPositionError(x, y).into())
        } else {
            Ok(())
        }
    }

    pub fn glutin_window(&self) -> Result<GlutinWindow> {
        if let Some(window) = self.facade.get_window() {
            Ok(window)
        } else {
            Err(ErrorKind::MissingGlutinWindow.into())
        }
    }

    pub fn program(&self, vertex_src: &str, fragment_src: &str) -> Result<Program> {
        Program::from_source(
            &self.facade,
            &format!(
                "#version {}\n{}",
                GLSL_VERSION_STRING,
                try!(read_utf8_file(vertex_src).chain_err(
                    || "Failed to read vertex shader.",
                ))
            ),
            &format!(
                "#version {}\n{}",
                GLSL_VERSION_STRING,
                try!(read_utf8_file(fragment_src).chain_err(
                    || "Failed to read fragment shader.",
                ))
            ),
            None,
        ).chain_err(|| "Failed to build program.")
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
