#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Ok;
use learn::{
    Buffer, BufferBit, BufferType, BufferUsage, ShaderProgram, Texture, TextureFormat, TextureUnit,
    VertexArray, VertexDescription,
};
/// This example is about how to use `Texture` in OpenGL.
use learn_opengl_rs as learn;

use glfw::Context;
use tracing::trace;

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    /* Window */
    let mut win = learn::Window::new("Simple Triangle", 800, 600, glfw::WindowMode::Windowed)?;
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
        include_str!("../../assets/shaders/003-texture.vert"),
        include_str!("../../assets/shaders/003-texture.frag"),
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
    texture_container.bind_texture_unit("t_container", &shader_program);
    texture_face.bind_texture_unit("t_face", &shader_program);

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
