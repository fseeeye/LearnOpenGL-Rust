use std::{ffi::CStr, sync::mpsc};

use anyhow::bail;
use glfw::Context;
use tracing::{info, trace};

use crate::Camera;

pub struct EventPump {
    glfw: glfw::Glfw,
    receiver: mpsc::Receiver<(f64, glfw::WindowEvent)>,
}

impl EventPump {
    pub fn poll_events(&mut self) -> glfw::FlushedMessages<(f64, glfw::WindowEvent)> {
        self.glfw.poll_events();

        glfw::flush_messages(&self.receiver)
    }
}

#[derive(Debug)]
pub struct Window {
    glfw: glfw::Glfw,
    inner_win: glfw::Window,
    pub camera: Option<Camera>,
}

impl Window {
    pub fn new(
        title: &str,
        width: u32,
        height: u32,
        mode: glfw::WindowMode,
    ) -> anyhow::Result<(Self, EventPump)> {
        let mut glfw = match glfw::init(glfw::FAIL_ON_ERRORS) {
            Ok(glfw) => glfw,
            Err(e) => bail!("GLFW window init error: {e}"),
        };

        // Setting up GL Context in window: use OpenGL 3.3 with core profile
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        #[cfg(target_os = "macos")]
        {
            glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        }

        // Make window
        let (inner_win, events) = glfw.create_window(width, height, title, mode).unwrap();

        Ok((
            Self {
                glfw: glfw.clone(),
                inner_win,
                camera: None,
            },
            EventPump {
                glfw,
                receiver: events,
            },
        ))
    }

    pub fn setup(&mut self, camera: Option<Camera>) {
        self.camera = camera;

        // Make OpenGL Context in inner window, wrapper for `glfwMakeContextCurrent`
        self.inner_win.make_current();

        // Enable Vsync
        self.glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

        // Start polling for all available events
        self.inner_win.set_all_polling(true);
    }

    pub fn get_view_matrix(&self) -> nalgebra::Matrix4<f32> {
        if let Some(camera) = &self.camera {
            camera.get_lookat_matrix()
        } else {
            nalgebra::Matrix4::identity()
        }
    }

    /// Load Gl Functions from window
    pub fn load_gl(&mut self) {
        gl::load_with(|symbol| self.inner_win.get_proc_address(symbol));

        unsafe {
            let gl_vendor = CStr::from_ptr(gl::GetString(gl::VENDOR) as _)
                .to_str()
                .unwrap();
            let gl_renderer = CStr::from_ptr(gl::GetString(gl::RENDERER) as _)
                .to_str()
                .unwrap();
            let gl_version = CStr::from_ptr(gl::GetString(gl::VERSION) as _)
                .to_str()
                .unwrap();

            info!(
                Vendor = gl_vendor,
                Renderer = gl_renderer,
                Version = gl_version,
                "Load OpenGL sucessfully!"
            );
        }
    }

    /// Wrapper of `glfw::Window::close()`
    pub fn close(self) {
        self.inner_win.close();
        drop(self.glfw);
    }

    /// Wrapper of `glfw::Window::should_close()`
    pub fn should_close(&self) -> bool {
        self.inner_win.should_close()
    }

    /// Wrapper of `glfw::Window::swap_buffers()`
    pub fn swap_buffers(&mut self) {
        self.inner_win.swap_buffers();
    }

    /// Wrapper of `glfw::get_time()`
    pub fn get_time(&self) -> f64 {
        self.glfw.get_time()
    }

    /// Wrapper of `glfw::poll_events()`
    pub fn poll_events(&mut self) {
        self.glfw.poll_events();
    }

    /// Default event handler
    pub fn handle_event_default(&mut self, event: &glfw::WindowEvent, _timestamp: f64) -> bool {
        // handle camera input events
        if let Some(camera) = self.camera.as_mut() {
            if camera.handle_event(event) {
                return true;
            }
        }

        match event {
            glfw::WindowEvent::Close => {
                self.inner_win.set_should_close(true);
                return true;
            }
            glfw::WindowEvent::Key(key, _scancode, action, _modifier) => {
                if key == &glfw::Key::Escape && action == &glfw::Action::Press {
                    self.inner_win.set_should_close(true);
                    return true;
                }
            }
            glfw::WindowEvent::Size(w, h) => {
                trace!("Resizing to ({}, {})", w, h);
            }
            _ => (),
        }

        false
    }
}
