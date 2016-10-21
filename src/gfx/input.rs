use glium::glutin::{CursorState, Event, ElementState};
use nalgebra::Vector2;
use num::Zero;

use math::CpuScalar;
use gfx::Window;
use errors::Result;

pub use glium::glutin::MouseButton;
pub use glium::glutin::VirtualKeyCode as KeyCode;

pub enum Gesture {
    NoGesture,
    KeyHold(KeyCode),
    KeyDownTrigger(KeyCode),
    KeyUpTrigger(KeyCode),
    ButtonHold(MouseButton),
    ButtonDownTrigger(MouseButton),
    ButtonUpTrigger(MouseButton),
    AnyOf(Vec<Gesture>),
    AllOf(Vec<Gesture>),
    QuitTrigger,
}

pub enum Analog2d {
    NoAnalog2d,

    Mouse { sensitivity: CpuScalar },

    Gestures {
        x_positive: Gesture,
        x_negative: Gesture,
        y_positive: Gesture,
        y_negative: Gesture,
        step: CpuScalar,
    },

    Sum { analogs: Vec<Analog2d> },
}

pub struct Input {
    current_update_index: UpdateIndex,

    keyboard_state: [ButtonState; NUM_KEY_CODES],
    mouse_button_state: [ButtonState; NUM_MOUSE_BUTTONS],
    quit_requested_index: UpdateIndex,

    mouse_rel: Vector2<CpuScalar>,
}

impl Input {
    pub fn new(window: &mut Window) -> Result<Input> {
        try!(window.set_cursor_state(CursorState::Hide));
        try!(center_cursor(window));

        Ok(Input {
            current_update_index: 1,
            keyboard_state: [ButtonState::Up(0); NUM_KEY_CODES],
            mouse_button_state: [ButtonState::Up(0); NUM_MOUSE_BUTTONS],
            quit_requested_index: 0,
            mouse_rel: Vector2::zero(),
        })
    }

    pub fn update(&mut self, window: &mut Window) -> Result<()> {
        self.current_update_index += 1;
        self.mouse_rel = Vector2::zero();
        for event in window.facade().poll_events() {
            match event {
                Event::Closed { .. } => {
                    self.quit_requested_index = self.current_update_index;
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(key_code)) => {
                    self.keyboard_state[key_code as usize] =
                        ButtonState::Down(self.current_update_index);
                }
                Event::KeyboardInput(ElementState::Released, _, Some(key_code)) => {
                    self.keyboard_state[key_code as usize] =
                        ButtonState::Up(self.current_update_index);
                }
                Event::MouseMoved(x, y) => {
                    let size = window.size();
                    let x_relative = (size.width as CpuScalar) / 2.0 - x as CpuScalar;
                    let y_relative = (size.height as CpuScalar) / 2.0 - y as CpuScalar;

                    self.mouse_rel = Vector2::new(x_relative, y_relative);
                }
                Event::MouseInput(ElementState::Pressed, mouse_button) => {
                    if let Some(index) = mouse_button_to_index(mouse_button) {
                        self.mouse_button_state[index] =
                            ButtonState::Down(self.current_update_index);
                    }
                }
                Event::MouseInput(ElementState::Released, mouse_button) => {
                    if let Some(index) = mouse_button_to_index(mouse_button) {
                        self.mouse_button_state[index] = ButtonState::Up(self.current_update_index);
                    }
                }
                _ => {}
            }
        }
        if self.mouse_rel != Vector2::zero() {
            try!(center_cursor(window));
        }
        Ok(())
    }

    pub fn poll_gesture(&self, gesture: &Gesture) -> bool {
        match *gesture {
            Gesture::QuitTrigger => self.quit_requested_index == self.current_update_index,
            Gesture::KeyHold(code) => {
                match self.keyboard_state[code as usize] {
                    ButtonState::Down(_) => true,
                    ButtonState::Up(_) => false,
                }
            }
            Gesture::KeyDownTrigger(code) => {
                match self.keyboard_state[code as usize] {
                    ButtonState::Down(index) => self.current_update_index == index,
                    ButtonState::Up(_) => false,
                }
            }
            Gesture::KeyUpTrigger(code) => {
                match self.keyboard_state[code as usize] {
                    ButtonState::Down(_) => false,
                    ButtonState::Up(index) => self.current_update_index == index,
                }
            }
            Gesture::ButtonHold(button) => {
                match mouse_button_to_index(button) {
                    Some(index) => {
                        match self.mouse_button_state[index] {
                            ButtonState::Down(_) => true,
                            ButtonState::Up(_) => false,
                        }
                    }
                    None => false,
                }
            }
            Gesture::ButtonDownTrigger(button) => {
                match mouse_button_to_index(button) {
                    Some(index) => {
                        match self.mouse_button_state[index] {
                            ButtonState::Down(index) => self.current_update_index == index,
                            ButtonState::Up(_) => false,
                        }
                    }
                    None => false,
                }
            }
            Gesture::ButtonUpTrigger(button) => {
                match mouse_button_to_index(button) {
                    Some(index) => {
                        match self.mouse_button_state[index] {
                            ButtonState::Down(_) => false,
                            ButtonState::Up(index) => self.current_update_index == index,
                        }
                    }
                    None => false,
                }
            }
            Gesture::AnyOf(ref subgestures) => {
                subgestures.iter().any(|subgesture| self.poll_gesture(subgesture))
            }
            Gesture::AllOf(ref subgestures) => {
                subgestures.iter().all(|subgesture| self.poll_gesture(subgesture))
            }
            Gesture::NoGesture => false,
        }
    }

    pub fn poll_analog2d(&self, motion: &Analog2d) -> Vector2<CpuScalar> {
        match *motion {
            Analog2d::Sum { ref analogs } => {
                analogs.iter()
                    .map(|analog| self.poll_analog2d(analog))
                    .fold(Vector2::zero(), |x, y| x + y)
            }
            Analog2d::Mouse { sensitivity } => self.mouse_rel * sensitivity,
            Analog2d::Gestures { ref x_positive,
                                 ref x_negative,
                                 ref y_positive,
                                 ref y_negative,
                                 step } => {
                Vector2::new(if self.poll_gesture(x_positive) {
                                 step
                             } else if self.poll_gesture(x_negative) {
                                 -step
                             } else {
                                 0.0
                             },
                             if self.poll_gesture(y_positive) {
                                 step
                             } else if self.poll_gesture(y_negative) {
                                 -step
                             } else {
                                 0.0
                             })
            }
            Analog2d::NoAnalog2d => Vector2::zero(),
        }
    }
}

const NUM_KEY_CODES: usize = 256;
const NUM_MOUSE_BUTTONS: usize = 256;

type UpdateIndex = u32;

#[derive(Copy, Clone)]
enum ButtonState {
    Up(UpdateIndex),
    Down(UpdateIndex),
}

fn mouse_button_to_index(button: MouseButton) -> Option<usize> {
    Some(match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::Other(index) => {
            let index: usize = index as usize + 3;
            if index > NUM_MOUSE_BUTTONS {
                warn!("Unsupported mouse button: {}", index);
                return None;
            }
            index
        }
    })
}

fn center_cursor(window: &mut Window) -> Result<()> {
    let size = window.size();
    window.set_cursor_position((size.width as i32) / 2, (size.height as i32) / 2)
}
