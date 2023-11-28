use std::{ffi::CStr, sync::mpsc};

use anyhow::bail;
use glfw::Context;
use tracing::{info, trace};

pub struct GlfwEventloop {
    glfw: glfw::Glfw,
    receiver: mpsc::Receiver<(f64, glfw::WindowEvent)>,
}

impl GlfwEventloop {
    pub fn poll_events(&mut self) -> glfw::FlushedMessages<(f64, glfw::WindowEvent)> {
        self.glfw.poll_events();

        glfw::flush_messages(&self.receiver)
    }
}

#[derive(Debug)]
pub struct GlfwWindow {
    glfw: glfw::Glfw,
    inner_win: glfw::Window,
}

impl GlfwWindow {
    pub fn new(
        title: &str,
        width: u32,
        height: u32,
        mode: glfw::WindowMode,
    ) -> anyhow::Result<(Self, GlfwEventloop)> {
        let mut glfw = match glfw::init(glfw::fail_on_errors) {
            Ok(glfw) => glfw,
            Err(e) => bail!("GLFW window init error: {e}"),
        };

        // Setup GL version : use OpenGL 3.3 with core profile
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
        let mut win = Self {
            glfw: glfw.clone(),
            inner_win,
        };

        // Setup window
        win.setup();

        Ok((
            win,
            GlfwEventloop {
                glfw,
                receiver: events,
            },
        ))
    }

    fn setup(&mut self) {
        // Setup OpenGL Context
        self.inner_win.make_current();
        self.load_gl();

        // Enable Vsync
        self.glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

        // Setup Viewport
        let (width, height) = self.inner_win.get_framebuffer_size();
        unsafe {
            gl::Viewport(0, 0, width, height);
        }

        // Start polling for all available events
        self.inner_win.set_all_polling(true);
    }

    /// Load Gl Functions from window
    fn load_gl(&mut self) {
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
            let gl_shading_language_version =
                CStr::from_ptr(gl::GetString(gl::SHADING_LANGUAGE_VERSION) as _)
                    .to_str()
                    .unwrap();

            info!(
                Vendor = gl_vendor,
                Renderer = gl_renderer,
                Version = gl_version,
                SlVersion = gl_shading_language_version,
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
                // TODO
            }
            _ => (),
        }

        false
    }
}
