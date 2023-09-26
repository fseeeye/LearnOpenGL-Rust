//! This example has more infos about depth test.

// remove console window : https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ffi::CString, path::PathBuf};

use anyhow::bail;
use gl::types::*;

use learn::{clear_color, set_clear_color, BufferBit, Camera, Model, ShaderProgram, WinitWindow};
use learn_opengl_rs as learn;

use nalgebra as na;
use tracing::error;
use winit::event::Event;

/* Screen info */
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const BACKGROUND_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/* Camera data */
const CAMERA_POS: [f32; 3] = [0.0, 0.5, 2.0];

struct Renderer {
    cube_model: Model,
    plane_model: Model,
    object_shader: ShaderProgram,
}

impl Renderer {
    pub fn new() -> anyhow::Result<Self> {
        /* Extra Settings */

        // Set clear color
        set_clear_color(
            BACKGROUND_COLOR[0],
            BACKGROUND_COLOR[1],
            BACKGROUND_COLOR[2],
            BACKGROUND_COLOR[3],
        );
        unsafe {
            // Enable Depth Test
            gl::Enable(gl::DEPTH_TEST);
            // Set depth function
            gl::DepthFunc(gl::LESS);
        };

        /* Object Vertices & Shader */

        // Prepare model of object
        let cube_model = Model::new(PathBuf::from("assets/models/cube/cube.obj"))?;
        let plane_model = Model::new(PathBuf::from("assets/models/plane/plane.obj"))?;

        // Prepare shader of object
        let object_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/advanced_opengl/012-object.vert"),
            include_str!("../../assets/shaders/advanced_opengl/012-object.frag"),
        )?;

        Ok(Self {
            cube_model,
            plane_model,
            object_shader,
        })
    }

    pub fn redraw(
        &self,
        win: &WinitWindow,
        camera: &Camera,
        _delta_time: f32,
    ) -> anyhow::Result<()> {
        clear_color(
            (BufferBit::ColorBufferBit as GLenum | BufferBit::DepthBufferBit as GLenum)
                as gl::types::GLbitfield,
        );

        // Model Matrix
        let model_name = CString::new("model")?;
        let normal_matrix_name = CString::new("normal_matrix")?;
        let object_model_matrix = na::Matrix4::identity();
        let object_normal_matrix = object_model_matrix
            .fixed_view::<3, 3>(0, 0)
            .try_inverse()
            .unwrap()
            .transpose();

        // View Matrix
        let view_name = CString::new("view")?;
        let object_view_matrix = camera.get_lookat_matrix();

        // Projection Matrix
        let (window_width, window_height) = win.get_window_size();
        let projection_matrix = na::Perspective3::new(
            (window_width as f32) / (window_height as f32),
            std::f32::consts::FRAC_PI_4,
            0.1,
            100.0,
        )
        .to_homogeneous(); // Perspective projection
        let projection_name = CString::new("projection")?;

        /* Draw object */

        self.object_shader.bind();

        self.object_shader
            .set_uniform_mat4fv(model_name.as_c_str(), &object_model_matrix);
        self.object_shader
            .set_uniform_mat3fv(normal_matrix_name.as_c_str(), &object_normal_matrix);
        self.object_shader
            .set_uniform_mat4fv(view_name.as_c_str(), &object_view_matrix);
        self.object_shader
            .set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);
        self.object_shader.set_uniform_3f(
            CString::new("camera_pos")?.as_c_str(),
            camera.get_pos().x,
            camera.get_pos().y,
            camera.get_pos().z,
        );

        self.cube_model.draw(&self.object_shader, "material")?;
        self.plane_model.draw(&self.object_shader, "material")?;

        // Swap buffers of window
        win.swap_buffers()?;

        Ok(())
    }

    pub fn close(self) {
        self.object_shader.close();
    }
}

#[allow(unreachable_code)]
fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    /* Camera */
    // Init camera at pos(0,0,3) look-at(0,0,0) up(0,1,0)
    let camera_pos = na::Point3::new(CAMERA_POS[0], CAMERA_POS[1], CAMERA_POS[2]);
    let camera_look_at = na::Vector3::new(0.0, 0.0, -1.0);
    let camera_up = na::Vector3::new(0.0, 1.0, 0.0);
    let mut camera = learn::Camera::new(camera_pos, camera_look_at, camera_up);

    /* Window */
    let (win, event_loop) = match WinitWindow::new("Simple Triangle", SCREEN_WIDTH, SCREEN_HEIGHT) {
        Ok((win, event_loop)) => (win, event_loop),
        Err(e) => {
            error!("Failed to create window: {}", e);
            bail!(e);
        }
    };

    /* Renderer */
    let renderer = match Renderer::new() {
        Ok(renderer) => renderer,
        Err(e) => {
            bail!("Failed to create renderer: {}", e);
        }
    };

    /* Main Loop */
    let mut last_time = std::time::SystemTime::now();
    event_loop.run(move |event, _window_target, control_flow| {
        // Set ControlFlow::Poll: when the current loop iteration finishes, immediately begin a new iteration regardless
        // of whether or not new events are available to process. This is ideal for games and similar applications.
        control_flow.set_poll();

        if win.handle_event_default(&event, control_flow) {
            return;
        }

        match event {
            // Event "RedrawRequested" : Emitted after MainEventsCleared when a window should be redrawn.
            Event::RedrawRequested(_window_id) => {
                let current_time = std::time::SystemTime::now();
                let delta_time = current_time
                    .duration_since(last_time)
                    .unwrap()
                    .as_secs_f32();
                last_time = current_time;

                /* Do REDRAW */
                if let Err(e) = renderer.redraw(&win, &camera, delta_time) {
                    error!("Failed to redraw: {}", e);
                    control_flow.set_exit();
                }
            }
            Event::WindowEvent { event, .. } => if !camera.handle_winit_event(&event) {},
            _ => (),
        }
    });

    renderer.close();
    Ok(())
}
