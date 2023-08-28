//! This example is about how to impl a camera which decides view matrix dynamically.

// remove console window : https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::CString;

use anyhow::bail;
use gl::types::*;

use learn::{
    Buffer, BufferBit, BufferType, BufferUsage, Camera, ShaderProgram, VertexArray, VertexDescription, WinitWindow,
};
use learn_opengl_rs as learn;

use nalgebra as na;
use tracing::error;
use winit::event::Event;

/* Screen info */
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

/* Vertex data */
type Vertex = [f32; 3]; // NDC coords(3)
const CUBE_VERTICES: [Vertex; 36] = [
    // panel 1
    [-0.5, -0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, 0.5, -0.5],
    [0.5, 0.5, -0.5],
    [-0.5, 0.5, -0.5],
    [-0.5, -0.5, -0.5],
    // panel 2
    [-0.5, -0.5, 0.5],
    [0.5, -0.5, 0.5],
    [0.5, 0.5, 0.5],
    [0.5, 0.5, 0.5],
    [-0.5, 0.5, 0.5],
    [-0.5, -0.5, 0.5],
    // panel 3
    [-0.5, 0.5, 0.5],
    [-0.5, 0.5, -0.5],
    [-0.5, -0.5, -0.5],
    [-0.5, -0.5, -0.5],
    [-0.5, -0.5, 0.5],
    [-0.5, 0.5, 0.5],
    // panel 4
    [0.5, 0.5, 0.5],
    [0.5, 0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, -0.5, 0.5],
    [0.5, 0.5, 0.5],
    // panel 5
    [-0.5, -0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, -0.5, 0.5],
    [0.5, -0.5, 0.5],
    [-0.5, -0.5, 0.5],
    [-0.5, -0.5, -0.5],
    // panel 6
    [-0.5, 0.5, -0.5],
    [0.5, 0.5, -0.5],
    [0.5, 0.5, 0.5],
    [0.5, 0.5, 0.5],
    [-0.5, 0.5, 0.5],
    [-0.5, 0.5, -0.5],
];

struct Renderer {
    shader_program: ShaderProgram,
    vao: VertexArray,
}

impl Renderer {
    pub fn new() -> anyhow::Result<Self> {
        /* Extra Settings */
        // Set clear color
        Buffer::set_clear_color(0.0, 0.0, 0.0, 1.0);
        // Enable Depth Test
        unsafe { gl::Enable(gl::DEPTH_TEST) };

        /* Vertex Array Object (VAO) */
        let vao = VertexArray::new()?;

        /* Vertex Buffer Object (VBO) */
        let vbo = Buffer::new(BufferType::VertexBuffer)?;
        vbo.bind();
        vbo.set_buffer_data(bytemuck::cast_slice(&CUBE_VERTICES), BufferUsage::StaticDraw);

        /* Vertex Attribute description */
        vao.bind();
        let mut vertex_desc = VertexDescription::new();
        vertex_desc.add_attribute(gl::FLOAT, 3); // push NDC coords
        vertex_desc.bind_to(&vbo, Some(&vao));

        /* Shader */
        let shader_program = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/lighting/007-color.vert"),
            include_str!("../../assets/shaders/lighting/007-color.frag"),
        )?;

        Ok(Self {
            shader_program,
            vao,
        })
    }

    pub fn redraw(
        &self,
        win: &WinitWindow,
        camera: &Camera,
        _delta_time: f32,
    ) -> anyhow::Result<()> {
        Buffer::clear(
            (BufferBit::ColorBufferBit as GLenum | BufferBit::DepthBufferBit as GLenum)
                as gl::types::GLbitfield,
        );

        self.vao.bind();

        self.shader_program.bind();

        // View Matrix: Send to shader
        let view_name = CString::new("view")?;
        self.shader_program
            .set_uniform_mat4fv(view_name.as_c_str(), &camera.get_lookat_matrix());

        // Projection Matrix: Create and Send to shader
        let projection_matrix = na::Perspective3::new(
            (SCREEN_WIDTH as f32) / (SCREEN_HEIGHT as f32),
            std::f32::consts::FRAC_PI_4,
            0.1,
            100.0,
        )
        .to_homogeneous(); // Perspective projection
        let projection_name = CString::new("projection")?;
        self.shader_program
            .set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);
        
        // Model Matrix
        let model_matrix = na::Matrix3::identity().to_homogeneous();
        let model_name = CString::new("model")?;
        self.shader_program.set_uniform_mat4fv(model_name.as_c_str(), &model_matrix);

        // Draw
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }

        // Swap buffers of window
        win.swap_buffers()?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn close(self) {
        self.shader_program.close();
    }
}

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    /* Camera */
    // Init camera at pos(0,0,3) look-at(0,0,0) up(0,1,0)
    let camera_pos = na::Point3::new(0.0, 0.0, 3.0);
    let camera_target = na::Point3::new(0.0, 0.0, 0.0);
    let camera_up = na::Vector3::new(0.0, 1.0, 0.0);
    let mut camera = learn::Camera::new(camera_pos, camera_target, camera_up);

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
            error!("Failed to create renderer: {}", e);
            bail!(e);
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
            // Emitted after MainEventsCleared when a window should be redrawn.
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
}
