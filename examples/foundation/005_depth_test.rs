#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::CString;

/// This example is about how to enable Depth Test. It will show multiple cubes.
use gl::types::*;
use learn::{
    Buffer, BufferBit, BufferType, BufferUsage, ShaderProgram, Texture, TextureFormat, TextureUnit,
    VertexArray, VertexDescription,
};
use learn_opengl_rs as learn;
use nalgebra as na;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    /* Window */
    let (mut win, mut event_loop) = learn::GlfwWindow::new(
        "Rolling Box - Depth Test",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        glfw::WindowMode::Windowed,
    )?;
    win.setup();
    win.load_gl();

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

    /* Main Loop */
    'main_loop: loop {
        if win.should_close() {
            break 'main_loop;
        }

        /* Handle events of this frame */
        for (timestamp, event) in event_loop.poll_events() {
            if !win.handle_event_default(&event, timestamp) {}
        }

        /* On Update (Drawing) */
        Buffer::clear(
            (BufferBit::ColorBufferBit as GLenum | BufferBit::DepthBufferBit as GLenum)
                as gl::types::GLbitfield,
        );

        shader_program.bind();

        vao.bind();

        // View Matrix
        let view_matrix = na::Translation3::new(0.0, 0.0, -3.0).to_homogeneous();
        let view_name = CString::new("view")?;
        shader_program.set_uniform_mat4fv(view_name.as_c_str(), &view_matrix);

        // Projection Matrix
        let projection_matrix = na::Perspective3::new(
            (SCREEN_WIDTH as f32) / (SCREEN_HEIGHT as f32),
            std::f32::consts::FRAC_PI_4,
            0.1,
            100.0,
        )
        .to_homogeneous(); // Perspective projection
        let projection_name = CString::new("projection")?;
        shader_program.set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);

        for cube_position in CUBE_POSTIONS {
            // Model Matrix
            let model_matrix_rotation = na::Rotation3::from_axis_angle(
                &na::Unit::new_normalize(na::Vector3::new(0.5, 1.0, 0.0)),
                -std::f32::consts::PI / 3.0 * (win.get_time() as f32),
            )
            .to_homogeneous();
            let model_matrix_transform = na::Translation3::from(cube_position).to_homogeneous();
            let model_matrix = model_matrix_transform * model_matrix_rotation;
            let model_name = CString::new("model")?;
            shader_program.set_uniform_mat4fv(model_name.as_c_str(), &model_matrix);

            // Draw
            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }
        }

        // Swap buffers of window
        win.swap_buffers();
    }

    shader_program.close();
    win.close();

    Ok(())
}
