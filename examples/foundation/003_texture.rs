#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Ok;
use learn::{
    Buffer, BufferBit, BufferType, BufferUsage, ShaderProgram, VertexArray, VertexDescription,
};
/// This example is about how to use `Texture` in OpenGL.
use learn_opengl_rs as learn;

use gl::types::*;
use image::GenericImageView;

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    /* Window */
    let (mut win, mut event_loop) =
        learn::GlfwWindow::new("Simple Texture", 800, 600, glfw::WindowMode::Windowed)?;
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
    let mut vbo = Buffer::new(BufferType::VertexBuffer)?;
    vbo.set_buffer_data(bytemuck::cast_slice(&VERTICES), BufferUsage::StaticDraw);

    /* Vertex Attribute description */
    let mut vertex_desc = VertexDescription::new();
    vertex_desc.push(gl::FLOAT, 3); // push NDC coords
    vertex_desc.push(gl::FLOAT, 2); // push texture coords
    vbo.set_vertex_description(&vertex_desc, Some(&vao));

    /* Index Buffer Object */
    let ibo = Buffer::new(BufferType::IndexBuffer)?;
    ibo.set_buffer_data(bytemuck::cast_slice(&INDICES), BufferUsage::StaticDraw);

    /* Shader */
    let shader_program = ShaderProgram::create_from_source(
        include_str!("../../assets/shaders/foundation/003-texture.vert"),
        include_str!("../../assets/shaders/foundation/003-texture.frag"),
    )?;

    /* Texture */
    let mut texture_container = 0;
    {
        // Load Texture image
        let img = image::open("assets/textures/container.jpg")
            .unwrap()
            .flipv();
        let (width, height) = img.dimensions();
        let pixels = img.into_bytes();

        // Generate Texture
        unsafe { gl::GenTextures(1, &mut texture_container) }
        assert_ne!(texture_container, 0);
        // Active Texture Unit0, but it's unnecessary for TEXTURE 0
        unsafe { gl::ActiveTexture(gl::TEXTURE0) }
        // Bind Texture
        unsafe { gl::BindTexture(gl::TEXTURE_2D, texture_container) }
        // Set Texture wrapping & filtering
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        }
        // Send Texture image data
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as GLint,
                width.try_into()?,
                height.try_into()?,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                pixels.as_ptr().cast(),
            );
        }
        // Generate mipmap
        unsafe { gl::GenerateMipmap(gl::TEXTURE_2D) }
    }

    let mut texture_face = 0;
    {
        // Load Texture
        let img = image::open("assets/textures/awesomeface.png")
            .unwrap()
            .flipv();
        let (width, height) = img.dimensions();
        let pixels = img.into_bytes();

        // Generate Texture
        unsafe { gl::GenTextures(1, &mut texture_face) }
        assert_ne!(texture_face, 0);
        // Active Texture Unit1
        unsafe { gl::ActiveTexture(gl::TEXTURE1) }
        // Bind Texture
        unsafe { gl::BindTexture(gl::TEXTURE_2D, texture_face) }
        // Set Texture wrapping & filtering
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        }
        // Send Texture image data
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                width.try_into()?,
                height.try_into()?,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_ptr().cast(),
            );
        }
        // Generate mipmap
        unsafe { gl::GenerateMipmap(gl::TEXTURE_2D) }
    }

    unsafe {
        shader_program.bind();
        // Bind sampler uniform var to spec texture unit
        gl::Uniform1i(
            gl::GetUniformLocation(shader_program.id, "t_container".as_ptr().cast()),
            0,
        ); // unnecessary for TEXTURE 0
        gl::Uniform1i(
            gl::GetUniformLocation(shader_program.id, "t_face".as_ptr().cast()),
            1,
        );
    }

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
