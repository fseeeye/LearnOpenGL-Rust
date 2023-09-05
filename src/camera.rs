use nalgebra as na;
use winit::event::{ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode, WindowEvent};

use tracing::trace;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Camera {
    // basic attributes
    pos: na::Point3<f32>,
    look_at: na::Unit<na::Vector3<f32>>,
    up: na::Unit<na::Vector3<f32>>,
    camera_speed: f32,
    // motion attributes
    first_move: bool,
    is_moving: bool,
    last_cursor_pos: na::Point2<f32>,
}

impl Camera {
    pub fn new(pos: na::Point3<f32>, look_at: na::Vector3<f32>, up: na::Vector3<f32>) -> Self {
        Self {
            pos,
            look_at: na::Unit::new_normalize(look_at),
            up: na::Unit::new_normalize(up),
            camera_speed: 0.1,

            first_move: false,
            is_moving: false,
            last_cursor_pos: na::Point2::new(0.0, 0.0),
        }
    }

    pub fn get_lookat_matrix(&self) -> na::Matrix4<f32> {
        let target_pos = self.pos + self.look_at.into_inner();

        // View Tranform Matrix (right-handed)
        // right-handed: camera always look at -z after transform
        // left-handed:  camera always look at +z after transform
        na::Matrix4::look_at_rh(&self.pos, &target_pos, &self.up)
    }

    pub fn get_lookat(&self) -> na::Unit<na::Vector3<f32>> {
        self.look_at
    }

    pub fn get_pos(&self) -> na::Point3<f32> {
        self.pos
    }

    #[inline]
    fn print_camera_pos(&self) {
        trace!("Camera pos: {:?}", self.pos);
    }

    #[inline]
    fn get_right_direction(&self) -> na::Unit<na::Vector3<f32>> {
        na::Unit::new_normalize(self.look_at.cross(&self.up))
    }

    pub fn move_front(&mut self, distance: f32) {
        self.pos += self.look_at.into_inner() * distance;

        self.print_camera_pos();
    }

    pub fn move_right(&mut self, distance: f32) {
        let right_vec = self.get_right_direction();

        self.pos += right_vec.into_inner() * distance;

        self.print_camera_pos();
    }

    pub fn move_up(&mut self, distance: f32) {
        self.pos += self.up.into_inner() * distance;

        self.print_camera_pos();
    }

    pub fn handle_glfw_event(&mut self, event: &glfw::WindowEvent) -> bool {
        if let glfw::WindowEvent::Key(key, _scancode, action, _modifier) = event {
            if (key == &glfw::Key::W)
                && (action == &glfw::Action::Press || action == &glfw::Action::Repeat)
            {
                self.move_up(self.camera_speed);
                return true;
            } else if (key == &glfw::Key::S)
                && (action == &glfw::Action::Press || action == &glfw::Action::Repeat)
            {
                self.move_up(-self.camera_speed);
                return true;
            } else if (key == &glfw::Key::D)
                && (action == &glfw::Action::Press || action == &glfw::Action::Repeat)
            {
                self.move_right(self.camera_speed);
                return true;
            } else if (key == &glfw::Key::A)
                && (action == &glfw::Action::Press || action == &glfw::Action::Repeat)
            {
                self.move_right(-self.camera_speed);
                return true;
            }
        }

        false
    }

    pub fn handle_winit_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::W),
                        ..
                    },
                ..
            } => {
                self.move_up(self.camera_speed);
                true
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::S),
                        ..
                    },
                ..
            } => {
                self.move_up(-self.camera_speed);
                true
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::D),
                        ..
                    },
                ..
            } => {
                self.move_right(self.camera_speed);
                true
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::A),
                        ..
                    },
                ..
            } => {
                self.move_right(-self.camera_speed);
                true
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                if let MouseScrollDelta::LineDelta(_, down) = delta {
                    if phase == &winit::event::TouchPhase::Started
                        || phase == &winit::event::TouchPhase::Moved
                    {
                        self.move_front(self.camera_speed * down);
                        return true;
                    }
                }

                false
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == winit::event::MouseButton::Left && *state == ElementState::Pressed {
                    self.is_moving = true;
                    true
                } else if *button == winit::event::MouseButton::Left
                    && *state == ElementState::Released
                {
                    self.is_moving = false;
                    true
                } else {
                    false
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.first_move {
                    self.last_cursor_pos = na::Point2::new(position.x as f32, position.y as f32);
                    self.first_move = false;
                    return true;
                } else if !self.is_moving {
                    self.last_cursor_pos = na::Point2::new(position.x as f32, position.y as f32);
                    return true;
                }

                // Calculate YAW (on y axis)
                let yaw_angle = (position.x as f32 - self.last_cursor_pos.x) * self.camera_speed;
                let yaw_rot =
                    na::Rotation3::from_axis_angle(&na::Vector3::y_axis(), yaw_angle.to_radians());

                self.look_at = yaw_rot * self.look_at;
                self.up = yaw_rot * self.up;

                // Calculate PITCH (on right direction)
                let right_vec = self.get_right_direction();

                let pitch_angle = (position.y as f32 - self.last_cursor_pos.y) * self.camera_speed;
                let pitch_rot =
                    na::Rotation3::from_axis_angle(&right_vec, pitch_angle.to_radians());

                self.look_at = pitch_rot * self.look_at;
                self.up = pitch_rot * self.up;

                // Reserve cursor position
                self.last_cursor_pos = na::Point2::new(position.x as f32, position.y as f32);

                true
            }
            _ => false,
        }
    }
}
