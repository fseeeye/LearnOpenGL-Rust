//! This example is about how to use MVP Transform in OpenGL.

// remove console window : https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::CString;

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
        "Transform Texture",
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
    let vbo = Buffer::new(BufferType::VertexBuffer)?;
    vbo.set_buffer_data(bytemuck::cast_slice(&VERTICES), BufferUsage::StaticDraw);

    /* Vertex Attribute description */
    let mut vertex_desc = VertexDescription::new();
    vertex_desc.add_attribute(gl::FLOAT, 3); // push NDC coords
    vertex_desc.add_attribute(gl::FLOAT, 2); // push texture coords
    vertex_desc.bind_to(&vbo, Some(&vao));

    /* Index Buffer Object */
    let ibo = Buffer::new(BufferType::IndexBuffer)?;
    ibo.set_buffer_data(bytemuck::cast_slice(&INDICES), BufferUsage::StaticDraw);

    /* Shader */
    let shader_program = ShaderProgram::create_from_source(
        include_str!("../../assets/shaders/foundation/004-transform.vert"),
        include_str!("../../assets/shaders/foundation/004-transform.frag"),
    )?;

    /* Transform Matrixes */
    shader_program.bind();
    // Model Matrix
    let model_matrix = na::Rotation3::from_axis_angle(
        &na::Vector3::x_axis(),
        -std::f32::consts::PI * (55.0 / 180.0),
    )
    .to_homogeneous();
    let model_loc = unsafe {
        gl::GetUniformLocation(
            shader_program.id,
            CString::new("model")?.as_c_str().as_ptr(),
        )
    };
    unsafe { gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model_matrix.as_ptr()) };

    // View Matrix
    let view_matrix = na::Translation3::new(0.0, 0.0, -3.0).to_homogeneous();
    let view_loc =
        unsafe { gl::GetUniformLocation(shader_program.id, CString::new("view")?.as_ptr().cast()) };
    unsafe { gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view_matrix.as_ptr()) };

    // Projection Matrix
    let projection_matrix = na::Perspective3::new(
        (SCREEN_WIDTH as f32) / (SCREEN_HEIGHT as f32),
        std::f32::consts::FRAC_PI_4,
        0.1,
        100.0,
    )
    .to_homogeneous(); // Perspective projection
    let projection_loc = unsafe {
        gl::GetUniformLocation(
            shader_program.id,
            CString::new("projection")?.as_ptr().cast(),
        )
    };
    unsafe { gl::UniformMatrix4fv(projection_loc, 1, gl::FALSE, projection_matrix.as_ptr()) };

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
    shader_program.set_texture_unit(&CString::new("t_container")?, &texture_container);
    shader_program.set_texture_unit(&CString::new("t_face")?, &texture_face);

    /* Extra Settings */
    Buffer::set_clear_color(0.2, 0.3, 0.3, 1.0);

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
        win.swap_buffers();
    }

    shader_program.close();
    win.close();

    Ok(())
}
