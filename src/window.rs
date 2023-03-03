use std::{ffi::CStr, sync::mpsc};

use anyhow::bail;
use glfw::Context;
use tracing::info;

#[derive(Debug)]
pub struct Window {
    pub glfw: glfw::Glfw,
    pub inner_win: glfw::Window,
    pub events: mpsc::Receiver<(f64, glfw::WindowEvent)>,
}

impl Window {
    pub fn new(
        title: &str,
        width: u32,
        height: u32,
        mode: glfw::WindowMode,
    ) -> anyhow::Result<Self> {
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
        let (win, events) = glfw.create_window(width, height, title, mode).unwrap();

        Ok(Self {
            glfw,
            inner_win: win,
            events,
        })
    }

    /// TODO: add option params
    pub fn setup(&mut self) {
        // Make OpenGL Context, wrapper for `glfwMakeContextCurrent`
        self.inner_win.make_current();

        // Enable Vsync
        self.glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

        // Start polling for all available events
        self.inner_win.set_all_polling(true);
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

    pub fn close(self) {
        self.inner_win.close();
        drop(self.glfw);
    }
}
