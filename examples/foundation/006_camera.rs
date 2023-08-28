#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::CString;

use anyhow::bail;
use gl::types::*;
/// This example is about how to impl a camera which decides view matrix dynamically.
use learn::{
    Buffer, BufferBit, BufferType, BufferUsage, Camera, ShaderProgram, Texture, TextureFormat,
    TextureUnit, VertexArray, VertexDescription, WinitWindow,
};
use learn_opengl_rs as learn;
use nalgebra as na;
use tracing::error;
use winit::event::Event;

/* Screen info */
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

/* Vertex data */
type Vertex = [f32; 3 + 2]; // NDC coords(3) + texture coords(2)
const VERTICES: [Vertex; 36] = [
    // panel 1
    [-0.5, -0.5, -0.5, 0.0, 0.0],
    [0.5, -0.5, -0.5, 1.0, 0.0],
    [0.5, 0.5, -0.5, 1.0, 1.0],
    [0.5, 0.5, -0.5, 1.0, 1.0],
    [-0.5, 0.5, -0.5, 0.0, 1.0],
    [-0.5, -0.5, -0.5, 0.0, 0.0],
    // panel 2
    [-0.5, -0.5, 0.5, 0.0, 0.0],
    [0.5, -0.5, 0.5, 1.0, 0.0],
    [0.5, 0.5, 0.5, 1.0, 1.0],
    [0.5, 0.5, 0.5, 1.0, 1.0],
    [-0.5, 0.5, 0.5, 0.0, 1.0],
    [-0.5, -0.5, 0.5, 0.0, 0.0],
    // panel 3
    [-0.5, 0.5, 0.5, 1.0, 0.0],
    [-0.5, 0.5, -0.5, 1.0, 1.0],
    [-0.5, -0.5, -0.5, 0.0, 1.0],
    [-0.5, -0.5, -0.5, 0.0, 1.0],
    [-0.5, -0.5, 0.5, 0.0, 0.0],
    [-0.5, 0.5, 0.5, 1.0, 0.0],
    // panel 4
    [0.5, 0.5, 0.5, 1.0, 0.0],
    [0.5, 0.5, -0.5, 1.0, 1.0],
    [0.5, -0.5, -0.5, 0.0, 1.0],
    [0.5, -0.5, -0.5, 0.0, 1.0],
    [0.5, -0.5, 0.5, 0.0, 0.0],
    [0.5, 0.5, 0.5, 1.0, 0.0],
    // panel 5
    [-0.5, -0.5, -0.5, 0.0, 1.0],
    [0.5, -0.5, -0.5, 1.0, 1.0],
    [0.5, -0.5, 0.5, 1.0, 0.0],
    [0.5, -0.5, 0.5, 1.0, 0.0],
    [-0.5, -0.5, 0.5, 0.0, 0.0],
    [-0.5, -0.5, -0.5, 0.0, 1.0],
    // panel 6
    [-0.5, 0.5, -0.5, 0.0, 1.0],
    [0.5, 0.5, -0.5, 1.0, 1.0],
    [0.5, 0.5, 0.5, 1.0, 0.0],
    [0.5, 0.5, 0.5, 1.0, 0.0],
    [-0.5, 0.5, 0.5, 0.0, 0.0],
    [-0.5, 0.5, -0.5, 0.0, 1.0],
];
const CUBE_POSTIONS: [na::Vector3<f32>; 10] = [
    na::Vector3::new(0.0, 0.0, 0.0),
    na::Vector3::new(2.0, 5.0, -15.0),
    na::Vector3::new(-1.5, -2.2, -2.5),
    na::Vector3::new(-3.8, -2.0, -12.3),
    na::Vector3::new(2.4, -0.4, -3.5),
    na::Vector3::new(-1.7, 3.0, -7.5),
    na::Vector3::new(1.3, -2.0, -2.5),
    na::Vector3::new(1.5, 2.0, -2.5),
    na::Vector3::new(1.5, 0.2, -1.5),
    na::Vector3::new(-1.3, 1.0, -1.5),
];

struct Renderer {
    shader_program: ShaderProgram,
    vao: VertexArray,
}

impl Renderer {
    pub fn new() -> anyhow::Result<Self> {
        /* Vertex Array Object */
        let vao = VertexArray::new()?;

        /* Vertex Buffer Object */
        let mut vbo = Buffer::new(BufferType::VertexBuffer)?;
        vbo.set_buffer_data(bytemuck::cast_slice(&VERTICES), BufferUsage::StaticDraw);

        /* Vertex Attribute description */
        let mut vertex_desc = VertexDescription::new();
        vertex_desc.push(gl::FLOAT, 3); // push NDC coords
        vertex_desc.push(gl::FLOAT, 2); // push texture coords
        vbo.set_vertex_description(&vertex_desc, Some(&vao));

        /* Shader */
        let shader_program = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/foundation/004-transform.vert"),
            include_str!("../../assets/shaders/foundation/004-transform.frag"),
        )?;

        /* Texture */
        let texture_container = Texture::create(
            "assets/textures/container.jpg",
            TextureFormat::RGB,
            TextureUnit::TEXTURE0,
        )?;
        let texture_face = Texture::create(
            "assets/textures/awesomeface.png",
            TextureFormat::RGBA,
            TextureUnit::TEXTURE1,
        )?;
        texture_container.active();
        texture_face.active();
        texture_face.bind_texture_unit("t_face", &shader_program);
        texture_container.bind_texture_unit("t_container", &shader_program);

        /* Extra Settings */
        Buffer::set_clear_color(0.2, 0.3, 0.3, 1.0);
        // Enable Depth Test
        unsafe { gl::Enable(gl::DEPTH_TEST) };

        Ok(Self {
            shader_program,
            vao,
        })
    }

    pub fn redraw(
        &self,
        win: &WinitWindow,
        camera: &Camera,
        delta_time: f32,
    ) -> anyhow::Result<()> {
        Buffer::clear(
            (BufferBit::ColorBufferBit as GLenum | BufferBit::DepthBufferBit as GLenum)
                as gl::types::GLbitfield,
        );

        self.shader_program.bind();

        self.vao.bind();

        // View Matrix: Send to shader
        let view_name = CString::new("view")?;
        self.shader_program
            .set_uniform_mat4fv(view_name.as_c_str(), &camera.get_lookat_matrix());

        // Projection Matrix: Create and Send to shader
        let (window_width, window_height) = win.get_window_size();
        let projection_matrix = na::Perspective3::new(
            (window_width as f32) / (window_height as f32),
            std::f32::consts::FRAC_PI_4,
            0.1,
            100.0,
        )
        .to_homogeneous(); // Perspective projection
        let projection_name = CString::new("projection")?;
        self.shader_program
            .set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);

        for cube_position in CUBE_POSTIONS {
            // Model Matrix: Create and Send to shader
            let model_matrix_rotation = na::Rotation3::from_axis_angle(
                &na::Unit::new_normalize(na::Vector3::new(0.5, 1.0, 0.0)),
                -std::f32::consts::PI / 3.0 * delta_time,
            )
            .to_homogeneous();
            let model_matrix_transform = na::Translation3::from(cube_position).to_homogeneous();
            let model_matrix = model_matrix_transform * model_matrix_rotation;
            let model_name = CString::new("model")?;
            self.shader_program
                .set_uniform_mat4fv(model_name.as_c_str(), &model_matrix);

            // Draw
            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }
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
    let camera_up = na::Vector3::y();
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

    let start_time = std::time::SystemTime::now();

    /* Main Loop */
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
                let delta_time = std::time::SystemTime::now()
                    .duration_since(start_time)
                    .unwrap()
                    .as_secs_f32();

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