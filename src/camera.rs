use nalgebra as na;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Camera {
    pos: na::Point3<f32>,
    target: na::Point3<f32>,
    up: na::Vector3<f32>,
    camera_speed: f32,
}

impl Camera {
    pub fn new(pos: na::Point3<f32>, target: na::Point3<f32>, up: na::Vector3<f32>) -> Self {
        Self {
            pos,
            target,
            up,
            camera_speed: 0.05,
        }
    }

    pub fn get_lookat_matrix(&self) -> na::Matrix4<f32> {
        // View Tranform Matrix (right-handed)
        // right-handed: camera always look at -z after transform
        // left-handed:  camera always look at +z after transform
        na::Matrix4::look_at_rh(&self.pos, &self.target, &self.up)
    }

    pub fn move_front(&mut self, distance: f32) {
        self.pos.z -= distance;
    }

    pub fn move_right(&mut self, distance: f32) {
        self.pos.x += distance;
    }

    pub fn move_up(&mut self, distance: f32) {
        self.pos.y += distance;
    }

    pub fn handle_glfw_event(&mut self, event: &glfw::WindowEvent) -> bool {
        if let glfw::WindowEvent::Key(key, _scancode, action, _modifier) = event {
            if (key == &glfw::Key::W)
                && (action == &glfw::Action::Press || action == &glfw::Action::Repeat)
            {
                self.move_front(self.camera_speed);
                return true;
            } else if (key == &glfw::Key::S)
                && (action == &glfw::Action::Press || action == &glfw::Action::Repeat)
            {
                self.move_front(-self.camera_speed);
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
                self.move_front(self.camera_speed);
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
                self.move_front(-self.camera_speed);
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
            _ => false,
        }
    }
}
