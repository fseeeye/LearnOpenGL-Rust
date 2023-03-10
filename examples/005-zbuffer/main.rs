#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::CString;

use anyhow::Ok;
use learn::{
    Buffer, BufferBit, BufferType, BufferUsage, ShaderProgram, Texture, TextureFormat, TextureUnit,
    VertexArray, VertexDescription,
};
/// This example is about how to use `Texture` in OpenGL.
use learn_opengl_rs as learn;
use nalgebra as na;

use glfw::Context;
use tracing::trace;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    /* Window */
    let mut win = learn::Window::new(
        "Simple Triangle",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        glfw::WindowMode::Windowed,
    )?;
    win.setup();
    win.load_gl();

    /* Vertex data */
    type Vertex = [f32; 3 + 2]; // NDC coords(3) + texture coords(3)
    type TriIndexes = [u32; 3]; // vertex indexes for a triangle primitive
    const VERTICES: [Vertex; 4] = [
        [0.5, 0.5, 0.0, 1.0, 1.0],
        [0.5, -0.5, 0.0, 1.0, 0.0],
        [-0.5, -0.5, 0.0, 0.0, 0.0],
        [-0.5, 0.5, 0.0, 0.0, 1.0],
    ];
    const INDICES: [TriIndexes; 2] = [[1, 2, 3], [0, 1, 3]];

    /* Vertex Array Object */
    let vao = VertexArray::new()?;

    /* Vertex Buffer Object */
    let mut vbo = Buffer::new(BufferType::Array)?;
    vbo.set_buffer_data(bytemuck::cast_slice(&VERTICES), BufferUsage::StaticDraw);

    /* Vertex Attribute description */
    let mut vertex_desc = VertexDescription::new();
    vertex_desc.push(gl::FLOAT, 3); // push NDC coords
    vertex_desc.push(gl::FLOAT, 2); // push texture coords
    vbo.set_vertex_description(&vertex_desc, Some(&vao));

    /* Index Buffer Object */
    let ibo = Buffer::new(BufferType::ElementArray)?;
    ibo.set_buffer_data(bytemuck::cast_slice(&INDICES), BufferUsage::StaticDraw);

    /* Shader */
    let shader_program = ShaderProgram::create_from_source(
        include_str!("../../assets/shaders/004-transform.vert"),
        include_str!("../../assets/shaders/004-transform.frag"),
    )?;

    /* Transform Matrixes */
    shader_program.bind();
    // Model Matrix
    let model_matrix = na::Rotation3::from_axis_angle(
        &na::Vector3::x_axis(),
        -std::f32::consts::PI / 180.0 * 55.0,
    )
    .to_homogeneous();
    let model_name = CString::new("model")?;
    shader_program.set_uniform_mat4fv(model_name.as_c_str(), model_matrix.as_ptr());

    // View Matrix
    let view_matrix = na::Translation3::new(0.0, 0.0, -3.0).to_homogeneous();
    let view_name = CString::new("view")?;
    shader_program.set_uniform_mat4fv(view_name.as_c_str(), view_matrix.as_ptr());

    // Projection Matrix
    let projection_matrix = na::Perspective3::new(
        (SCREEN_WIDTH as f32) / (SCREEN_HEIGHT as f32),
        std::f32::consts::FRAC_PI_4,
        0.1,
        100.0,
    )
    .to_homogeneous(); // Perspective projection
    let projection_name = CString::new("projection")?;
    shader_program.set_uniform_mat4fv(projection_name.as_c_str(), projection_matrix.as_ptr());

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

    /* Main Loop */
    'main_loop: loop {
        if win.inner_win.should_close() {
            break;
        }

        /* Handle events of this frame */
        win.glfw.poll_events();
        for (_timestamp, event) in glfw::flush_messages(&win.events) {
            match event {
                glfw::WindowEvent::Close => break 'main_loop,
                glfw::WindowEvent::Key(key, _scancode, action, _modifier) => {
                    if key == glfw::Key::Escape && action == glfw::Action::Press {
                        win.inner_win.set_should_close(true);
                    }
                }
                glfw::WindowEvent::Size(w, h) => {
                    trace!("Resizing to ({}, {})", w, h);
                }
                _ => (),
            }
        }

        /* On Update (Drawing) */
        Buffer::clear(BufferBit::ColorBufferBit as gl::types::GLbitfield);

        shader_program.bind();

        vao.bind();

        ibo.bind();

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                INDICES.len() as i32 * 3,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }

        // Swap buffers of window
        win.inner_win.swap_buffers();
    }

    shader_program.close();
    win.close();

    Ok(())
}
