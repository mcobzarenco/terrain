use nphysics3d::object::RigidBodyHandle;
use num::Zero;

use gfx::{Analog2d, Gesture, Input, KeyCode};
use math::{GpuScalar, Matrix4f};
use nalgebra::{Isometry3, Translation, Point3, Rotation, Vector2, Vector3, Inverse, ToHomogeneous};

pub struct ControllerBindings {
    pub movement: Analog2d,
    pub look: Analog2d,
    pub jump: Gesture,
}

impl Default for ControllerBindings {
    fn default() -> Self {
        ControllerBindings {
            movement: Analog2d::Gestures {
                x_positive: Gesture::KeyHold(KeyCode::D),
                x_negative: Gesture::KeyHold(KeyCode::A),
                y_positive: Gesture::KeyHold(KeyCode::W),
                y_negative: Gesture::KeyHold(KeyCode::S),
                step: 1.0,
            },
            look: Analog2d::Sum {
                analogs: vec![
                    Analog2d::Gestures {
                        x_positive: Gesture::KeyHold(KeyCode::Right),
                        x_negative: Gesture::KeyHold(KeyCode::Left),
                        y_positive: Gesture::KeyHold(KeyCode::Down),
                        y_negative: Gesture::KeyHold(KeyCode::Up),
                        step: 0.05,
                    },
                    Analog2d::Mouse { sensitivity: 0.008 },
                ],
            },
            jump: Gesture::KeyHold(KeyCode::Space),
        }
    }
}

pub struct Player {
    player: RigidBodyHandle<GpuScalar>,
    keyboard_speed: GpuScalar,
    mouse_speed: GpuScalar,
    pub observer: Isometry3<GpuScalar>,
}

impl Player {
    pub fn new(
        player: RigidBodyHandle<GpuScalar>,
        position: &Point3<GpuScalar>,
        target: &Point3<GpuScalar>,
        up: &Vector3<GpuScalar>,
    ) -> Self {
        player.borrow_mut().set_translation(position.to_vector());
        player.borrow_mut().set_deactivation_threshold(None);

        player.borrow_mut().set_margin(0.01);
        let observer = Isometry3::new_observer_frame(position, &target, &up);
        Player {
            player: player,
            keyboard_speed: 500.0,
            mouse_speed: 0.04,
            observer: observer,
        }
    }

    pub fn view_matrix(&self) -> Matrix4f {
        Matrix4f::from(self.observer.inverse().unwrap().to_homogeneous())
    }

    pub fn update_position(&mut self) -> Isometry3<GpuScalar> {
        let player = self.player.borrow();
        let position = player.position();
        self.observer.set_translation(position.translation());
        self.observer
    }

    pub fn update(&mut self, delta_time: f32, input: &Input) -> () {
        self.update_position();
        let mut player = self.player.borrow_mut();
        if input.poll_gesture(&Gesture::AnyOf(vec![
            Gesture::KeyUpTrigger(KeyCode::W),
            Gesture::KeyUpTrigger(KeyCode::A),
            Gesture::KeyUpTrigger(KeyCode::S),
            Gesture::KeyUpTrigger(KeyCode::D),
        ]))
        {
            player.clear_forces();
        }

        if input.poll_gesture(&Gesture::KeyHold(KeyCode::W)) {
            let movement = self.observer.rotation * Vector3::z() * self.keyboard_speed;
            player.append_lin_force(movement);
        }
        if input.poll_gesture(&Gesture::KeyHold(KeyCode::S)) {
            let movement = self.observer.rotation * Vector3::z() * self.keyboard_speed * -1.0;
            player.append_lin_force(movement);
        }
        if input.poll_gesture(&Gesture::KeyHold(KeyCode::A)) {
            let movement = self.observer.rotation * Vector3::x() * self.keyboard_speed * -1.0;
            player.append_lin_force(movement);
        }

        if input.poll_gesture(&Gesture::KeyHold(KeyCode::D)) {
            let movement = self.observer.rotation * Vector3::x() * self.keyboard_speed;
            player.append_lin_force(movement);
        }
        if input.poll_gesture(&Gesture::KeyHold(KeyCode::Space)) {
            let movement = self.observer.rotation * Vector3::y() * self.keyboard_speed * 0.1;
            player.apply_central_impulse(movement);
        }
        if input.poll_gesture(&Gesture::KeyHold(KeyCode::Q)) {
            let angle = self.observer.rotation * Vector3::z() * delta_time;
            self.observer.rotation.append_rotation_mut(&angle);
        }
        if input.poll_gesture(&Gesture::KeyHold(KeyCode::E)) {
            let angle = self.observer.rotation * Vector3::z() * delta_time * -1.0;
            self.observer.rotation.append_rotation_mut(&angle);
        }

        let mut mouse_rel = input.poll_analog2d(&Analog2d::Sum {
            analogs: vec![
                Analog2d::Gestures {
                    x_positive: Gesture::KeyHold(KeyCode::Right),
                    x_negative: Gesture::KeyHold(KeyCode::Left),
                    y_positive: Gesture::KeyHold(KeyCode::Down),
                    y_negative: Gesture::KeyHold(KeyCode::Up),
                    step: 0.5,
                },
                Analog2d::Mouse { sensitivity: 0.8 },
            ],
        });

        if mouse_rel != Vector2::zero() {
            mouse_rel *= self.mouse_speed * delta_time;
            let horizontal_angle = mouse_rel[0];
            let vertical_angle = mouse_rel[1];

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
    }
}
