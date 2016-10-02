use glium::glutin::{Window, Event, ElementState, VirtualKeyCode};

use math::{Mat4f, Vec3f, Vec4f, Vector};

#[derive(Debug)]
pub struct Camera {
    pub position: Vec3f,
    pub direction: Vec3f,
    pub up: Vec3f,

    keyboard_speed: f32,
    mouse_speed: f32,
    horizontal_angle: f32,
    vertical_angle: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vec3f::new(0.0, 0.0, -54.0),
            direction: Vec3f::new(0.0, 0.0, 1.0),
            up: Vec3f::new(0.0, 1.0, 0.0),
            horizontal_angle: 0.0,
            vertical_angle: 0.0,
            keyboard_speed: 64.0,
            mouse_speed: 0.1,
        }
    }

    pub fn view_matrix(&self) -> Mat4f {
        let position = &self.position;
        let direction = &self.direction;
        let up = &self.up;

        let f = direction.normalized();
        let s =
            [up[1] * f[2] - up[2] * f[1], up[2] * f[0] - up[0] * f[2], up[0] * f[1] - up[1] * f[0]];

        let s_norm = {
            let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
            let len = len.sqrt();
            [s[0] / len, s[1] / len, s[2] / len]
        };

        let u = [f[1] * s_norm[2] - f[2] * s_norm[1],
                 f[2] * s_norm[0] - f[0] * s_norm[2],
                 f[0] * s_norm[1] - f[1] * s_norm[0]];

        let p = [-position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
                 -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
                 -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

        Mat4f::from([[s_norm[0], u[0], f[0], 0.0],
                     [s_norm[1], u[1], f[1], 0.0],
                     [s_norm[2], u[2], f[2], 0.0],
                     [p[0], p[1], p[2], 1.0]])
    }

    pub fn update(&mut self, delta_time: f32, window: &Window, event: Event) -> () {
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
            // Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Q)) => {
            //     let rot = Mat4f::new_axis_rotation(&self.direction, delta_time * 3.1415);
            //     let direction =
            //         Vec4f::new(self.direction[0], self.direction[1], self.direction[2], 1.0);
            //     let direction = Vec4f::new(self.up[0], self.up[1], self.up[2], 1.0);
            //     direction = rot * direction;
            //     let up = rot * direction;
            //     self.direction = Vec3f::new(direction[0], direction[1], direction[2]);
            //     info!("New direction: {:?}", self.direction);
            //     // self.keyboard_speed /= 0.5;
            // }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::W)) => {
                self.position += self.direction * self.keyboard_speed * delta_time;
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::S)) => {
                self.position -= self.direction * self.keyboard_speed * delta_time;
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::A)) => {
                self.position += self.right() * self.keyboard_speed * delta_time;
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::D)) => {
                self.position -= self.right() * self.keyboard_speed * delta_time;
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Space)) => {
                self.position += self.up * self.keyboard_speed * delta_time;
            }

            // Handle mouse
            Event::MouseMoved(x, y) => {
                let (width, height) = window.get_inner_size_pixels().unwrap();
                window.set_cursor_position((width as i32) / 2, (height as i32) / 2).unwrap();
                self.horizontal_angle -= self.mouse_speed * delta_time *
                                         ((width as f32) / 2.0 - x as f32);

                let vertical_diff = self.mouse_speed * delta_time *
                                    ((height as f32) / 2.0 - y as f32);
                self.vertical_angle += vertical_diff;
                // if (vertical_diff + self.vertical_angle).abs() < 3.14159 / 2.0 {
                //     self.vertical_angle += vertical_diff;
                // }

                self.direction =
                    Vec3f::new(self.vertical_angle.cos() * self.horizontal_angle.sin(),
                               self.vertical_angle.sin(),
                               self.vertical_angle.cos() * self.horizontal_angle.cos())
                        .normalized();

                // Right vector
                let right = Vec3f::new((self.horizontal_angle - 3.14159 / 2.0).sin(),
                                       0.0,
                                       (self.horizontal_angle - 3.14159 / 2.0).cos());
                self.up = right.cross(&self.direction).normalized();

                // println!("Mouse at {} {}: {} {}",
                //          x,
                //          y,
                //          self.horizontal_angle,
                //          self.vertical_angle)
            }
            _ => (),
        }
        //        println!("Camera Orientation: {:?}", self);
    }

    fn right(&self) -> Vec3f {
        Vec3f::new((self.horizontal_angle - 3.14159 / 2.0).sin(),
                   0.0,
                   (self.horizontal_angle - 3.14159 / 2.0).cos())
    }
}
