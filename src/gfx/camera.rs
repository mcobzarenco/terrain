use glium::glutin::{Window, Event, ElementState, VirtualKeyCode};
use nalgebra::{Isometry3, Rotation, ToHomogeneous, Translation, Vector3, Inverse};

use math::{Matrix4f, Vec3f, Point3f, GpuScalar};

#[derive(Debug)]
pub struct Camera {
    keyboard_speed: GpuScalar,
    mouse_speed: GpuScalar,
    observer: Isometry3<GpuScalar>,
}

impl Camera {
    pub fn new(position: Point3f, target: Point3f, up: Vec3f) -> Self {
        let observer = Isometry3::new_observer_frame(&position, &target, &up);
        Camera {
            keyboard_speed: 64.0,
            mouse_speed: 0.04,
            observer: observer,
        }
    }

    pub fn view_matrix(&self) -> Matrix4f {
        Matrix4f::from(self.observer.inverse().unwrap().to_homogeneous())
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
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::W)) => {
                let movement = self.observer.rotation * Vector3::z() * self.keyboard_speed *
                    delta_time;
                self.observer.append_translation_mut(&movement);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::S)) => {
                let movement = self.observer.rotation * Vector3::z() * self.keyboard_speed *
                    delta_time * -1.0;
                self.observer.append_translation_mut(&movement);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::A)) => {
                let movement = self.observer.rotation * Vector3::x() * self.keyboard_speed *
                    delta_time * -1.0;
                self.observer.append_translation_mut(&movement);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::D)) => {
                let movement = self.observer.rotation * Vector3::x() * self.keyboard_speed *
                    delta_time;
                self.observer.append_translation_mut(&movement);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Space)) => {
                let movement = self.observer.rotation * Vector3::y() * self.keyboard_speed *
                    delta_time;
                self.observer.append_translation_mut(&movement);
            }

            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Q)) => {
                let angle = self.observer.rotation * Vector3::z() * delta_time;
                self.observer.rotation.append_rotation_mut(&angle);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::E)) => {
                let angle = self.observer.rotation * Vector3::z() * delta_time * -1.0;
                self.observer.rotation.append_rotation_mut(&angle);
            }

            // Handle mouse
            Event::MouseMoved(x, y) => {
                let (width, height) = window.get_inner_size_pixels().unwrap();
                window
                    .set_cursor_position((width as i32) / 2, (height as i32) / 2)
                    .unwrap();

                let horizontal_angle = self.mouse_speed * delta_time *
                    ((width as f32) / 2.0 - x as f32);
                let vertical_angle = self.mouse_speed * delta_time *
                    ((height as f32) / 2.0 - y as f32);

                let rotation = self.observer.rotation;

                self.observer.rotation.append_rotation_mut(
                    &(rotation * (Vector3::x() * -1.0) *
                          vertical_angle),
                );
                self.observer.rotation.append_rotation_mut(
                    &(rotation * (Vector3::y() * -1.0) *
                          horizontal_angle),
                );

            }
            _ => (),
        }
    }

    pub fn position(&self) -> Isometry3<GpuScalar> {
        self.observer
    }

    pub fn observer_mut(&mut self) -> &mut Isometry3<GpuScalar> {
        &mut self.observer
    }
}
