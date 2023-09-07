//! This example is about how to draw a simple quad.
//! It is involved about:
//! * Index Buffer Object
//! * Shader uniform
//! * Draw call: `DrawElements()`

// remove console window : https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::CString;

use anyhow::Ok;
use learn::{
    clear_color, set_clear_color, Buffer, BufferBit, BufferType, BufferUsage, ShaderProgram,
    VertexArray, VertexDescription,
};
use learn_opengl_rs as learn;

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    /* Window */
    let (mut win, mut event_loop) =
        learn::GlfwWindow::new("Simple Quad", 800, 600, glfw::WindowMode::Windowed)?;

    /* Vertex data */
    type Vertex = [f32; 3]; // x, y, z in Normalized Device Context (NDC) coordinates
    type TriIndexes = [u32; 3]; // vertex indexes for a triangle primitive
    const VERTICES: [Vertex; 4] = [
        [0.5, 0.5, 0.0],
        [0.5, -0.5, 0.0],
        [-0.5, -0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ];
    const INDICES: [TriIndexes; 2] = [[1, 2, 3], [0, 1, 3]];

    /* Vertex Array Object */
    let vao = VertexArray::new()?;

    /* Vertex Buffer Object */
    let vbo = Buffer::new(BufferType::VertexBuffer)?;
    vbo.set_buffer_data(VERTICES.as_slice(), BufferUsage::StaticDraw);

    /* Vertex Attribute description */
    let mut vertex_desc = VertexDescription::new();
    vertex_desc.add_attribute(gl::FLOAT, 3); // Vertex is [f32; 3]
    vertex_desc.bind_to(&vbo, Some(&vao));

    /* Index Buffer Object */
    // Generate a buffer
    let mut ibo = 0;
    unsafe {
        gl::GenBuffers(1, &mut ibo);
    }
    assert_ne!(ibo, 0);
    unsafe {
        // Bind Buffer as ELEMENT_ARRAY_BUFFER(IBO)
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        // Set buffer data
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            core::mem::size_of_val(&INDICES) as isize,
            INDICES.as_ptr().cast(),
            gl::STATIC_DRAW,
        );
    }

    /* Shader */
    let shader_program = ShaderProgram::create_from_source(
        include_str!("../../assets/shaders/foundation/002-uniform.vert"),
        include_str!("../../assets/shaders/foundation/002-uniform.frag"),
    )?;

    // Get uniform var
    let uniform_color_name = CString::new("dyn_color")?;
    let uniform_color_location =
        unsafe { gl::GetUniformLocation(shader_program.id, uniform_color_name.as_ptr()) };

    /* Extra Settings */
    set_clear_color(0.2, 0.3, 0.3, 1.0);

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
        clear_color(BufferBit::ColorBufferBit as gl::types::GLbitfield);

        shader_program.bind();

        // Send uniform value - 'dynamic color'
        let time = win.get_time() as f32;
        let color = (time.sin() / 2.0) + 0.5;
        unsafe {
            gl::Uniform4f(uniform_color_location, color, color, color, color);
        }

        vao.bind();

        unsafe {
            // Bind IBO
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);

            // Draw call
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
