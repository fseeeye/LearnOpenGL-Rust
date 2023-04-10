// ref: https://github.com/rust-windowing/glutin/blob/8e0960d7aa8c67ee709897001def551fc1d868bb/glutin_examples/src/lib.rs

use std::ffi::{CStr, CString};

use anyhow::bail;
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin_winit::GlWindow;
use raw_window_handle::HasRawWindowHandle;
use tracing::info;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

#[allow(dead_code)]
pub struct WinitWindow {
    inner_window: winit::window::Window,
    gl_context: glutin::context::PossiblyCurrentContext,
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl WinitWindow {
    /// Create window based on `winit` and `glutin`.
    pub fn new(
        title: &str,
        width: u32,
        height: u32,
    ) -> anyhow::Result<(Self, winit::event_loop::EventLoop<()>)> {
        // Create Event Loop
        let event_loop = winit::event_loop::EventLoopBuilder::new().build();

        // Create Window
        let (window, gl_config) = Self::create_gl_window(title, width, height, &event_loop)?;

        // Create the OpenGL context
        let (not_current_gl_context, gl_surface) = Self::create_gl_context(&window, &gl_config)?;

        // Setup context
        let gl_context = Self::setup_gl_context(not_current_gl_context, &gl_surface)?;

        Ok((
            Self {
                inner_window: window,
                gl_context,
                gl_surface,
            },
            event_loop,
        ))
    }

    fn create_gl_window(
        title: &str,
        width: u32,
        height: u32,
        event_loop: &winit::event_loop::EventLoop<()>,
    ) -> anyhow::Result<(winit::window::Window, glutin::config::Config)> {
        // Create window builder.
        //
        // Only WINDOWS requires the window to be present before creating the display.
        // Other platforms don't really need one. Current project don't care about running on android,
        // so we can always pass the window builder.
        let window_builder = Some(
            winit::window::WindowBuilder::new()
                .with_resizable(true)
                .with_title(title)
                .with_inner_size(winit::dpi::LogicalSize::new(width, height)),
        );

        // Build window and GlConfig using DisplayBuilder
        let display_builder = glutin_winit::DisplayBuilder::new()
            .with_preference(glutin_winit::ApiPrefence::FallbackEgl)
            .with_window_builder(window_builder);
        let (window, gl_config) =
            match display_builder.build(event_loop, <_>::default(), |configs| {
                // All normal platforms will return multiple configs, so we should find the config with the maximum number
                // of samples & SRGB capable, so that our triangle will be smooth.
                configs
                    .filter(|config| config.srgb_capable())
                    .max_by_key(|config| config.num_samples())
                    .unwrap()
            }) {
                Ok((window, gl_config)) => (window, gl_config),
                Err(e) => bail!("Failed to create window: {}", e),
            };
        let window = window.unwrap();

        info!(
            "Picked OpenGL config with {} samples",
            gl_config.num_samples()
        );

        Ok((window, gl_config))
    }

    fn create_gl_context(
        window: &winit::window::Window,
        gl_config: &glutin::config::Config,
    ) -> anyhow::Result<(
        glutin::context::NotCurrentContext,
        glutin::surface::Surface<glutin::surface::WindowSurface>,
    )> {
        // Get the raw window handle
        let raw_window_handle = window.raw_window_handle();

        // Create OpenGL context attributes (OpenGL 3.3 - Core)
        //
        // The context creation part. It can be created before surface and that's how
        // it's expected in multithreaded + multiwindow operation mode, since you
        // can send NotCurrentContext, but not Surface.
        let context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::OpenGl(Some(
                glutin::context::Version::new(3, 3),
            )))
            .with_profile(glutin::context::GlProfile::Core)
            .build(Some(raw_window_handle));

        // Create gl surface.
        let surface_attrs = window.build_surface_attributes(<_>::default());
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(gl_config, &surface_attrs)
        }?;

        // Create the OpenGL Context, but don't make it current.
        let not_current_gl_context = unsafe {
            gl_config
                .display()
                .create_context(gl_config, &context_attributes)?
        };

        Ok((not_current_gl_context, gl_surface))
    }

    fn setup_gl_context(
        not_current_gl_context: glutin::context::NotCurrentContext,
        gl_surface: &glutin::surface::Surface<glutin::surface::WindowSurface>,
    ) -> anyhow::Result<glutin::context::PossiblyCurrentContext> {
        // Make GlContext current
        //
        // The context needs to be current for OpenGL function pointers loading.
        //
        // I do `make_current()` here since there's not need to change context during runtime.
        let gl_context = not_current_gl_context.make_current(gl_surface)?;

        // Load the OpenGL function pointers
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            // The gl display could be obtained from the any object created by it, so we
            // can query it from the config.
            gl_surface
                .display()
                .get_proc_address(symbol.as_c_str())
                .cast()
        });

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

        // Try enable vsync.
        if let Err(e) = gl_surface.set_swap_interval(
            &gl_context,
            glutin::surface::SwapInterval::Wait(std::num::NonZeroU32::new(1).unwrap()),
        ) {
            bail!("Failed to set vsync: {e:?}");
        }

        Ok(gl_context)
    }

    /// Swap buffers of gl surface.
    pub fn swap_buffers(&self) -> anyhow::Result<()> {
        self.gl_surface.swap_buffers(&self.gl_context)?;

        Ok(())
    }

    /// Handle window events with some default processing logical.
    pub fn handle_event_default(&self, event: &Event<()>, control_flow: &mut ControlFlow) -> bool {
        match event {
            // Emitted when all of the event loop’s input events have been processed and redraw processing
            // is about to begin.
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually request it.
                self.inner_window.request_redraw();
                true
            }
            // A Window object can generate WindowEvents when certain input events occur,
            // such as a cursor moving over the window or a key getting pressed while the window is focused.
            Event::WindowEvent { event, .. } => match event {
                // Resize the window and redraw it.
                WindowEvent::Resized(physical_size) => {
                    self.gl_surface.resize(
                        &self.gl_context,
                        std::num::NonZeroU32::new(physical_size.width).unwrap(),
                        std::num::NonZeroU32::new(physical_size.height).unwrap(),
                    );
                    self.inner_window.request_redraw();
                    true
                }
                // Exit the program when the window should be closed.
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    is_synthetic: false,
                    ..
                } => {
                    control_flow.set_exit();
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }
}
