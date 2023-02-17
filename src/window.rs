use std::sync::mpsc;

use glfw::Context;

pub struct Window {
    glfw: glfw::Glfw,
    inner_win: glfw::Window,
    events: mpsc::Receiver<(f64, glfw::WindowEvent)>,
}

impl Window {
    pub fn new(title: &str, width: u32, height: u32, mode: glfw::WindowMode) -> Result<Self, ()> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).map_err(|_a| ())?;

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

    pub fn load_gl(&mut self) {
        // Load Gl Functions from window
        gl::load_with(|symbol| self.inner_win.get_proc_address(symbol));
    }

    pub fn swap_buffers(&mut self) {
        self.inner_win.swap_buffers();
    }

    pub fn poll_events(&mut self) -> glfw::FlushedMessages<(f64, glfw::WindowEvent)> {
        self.glfw.poll_events();

        glfw::flush_messages(&self.events)
    }

    pub fn should_close(&self) -> bool {
        self.inner_win.should_close()
    }

    pub fn close(self) {
        self.inner_win.close();
        drop(self.glfw);
    }
}
