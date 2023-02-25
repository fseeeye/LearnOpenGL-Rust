#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::CString;

use glfw::Context;
use learn::{
    Buffer, BufferType, BufferUsage, ShaderProgram, VertexArray, VertexDescription, Window,
};
use tracing::trace;

/// This example is about how to abstract safe gl funcs.
/// TODO:
/// * Add glGetError() after any gl funcs calling.
use learn_opengl_rs as learn;

fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set default subscriber");

    // Create Window
    let mut win = Window::new("Simple Triangle", 800, 600, glfw::WindowMode::Windowed)
        .expect("Failed to create window.");
    win.setup();
    win.load_gl();

    // Vertex data
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
    let vao = VertexArray::new().expect("Failed to make a VAO.");
    vao.bind();

    /* Vertex Buffer Object */
    let mut vbo = Buffer::new(BufferType::Array).expect("Failed to make a VBO");
    vbo.bind();
    vbo.set_buffer_data(bytemuck::cast_slice(&VERTICES), BufferUsage::StaticDraw);

    /* Vertex Attribute description */
    let mut vertex_desc = VertexDescription::new();
    vertex_desc.push(gl::FLOAT, 3); // Vertex is [f32; 3]
    vbo.set_vertex_description(&vertex_desc, Some(&vao));

    /* Index Buffer Object */
    let ibo = Buffer::new(BufferType::ElementArray).expect("Failed to make a IBO");
    ibo.bind();
    ibo.set_buffer_data(bytemuck::cast_slice(&INDICES), BufferUsage::StaticDraw);

    /* Shader */
    let shader_program = ShaderProgram::create_from_source(
        include_str!("../../assets/shaders/002-uniform.vert"),
        include_str!("../../assets/shaders/002-uniform.frag"),
    )
    .unwrap();

    let uniform_color_name = CString::new("dyn_color").unwrap();
    let uniform_color_location = shader_program.get_uniform_location(&uniform_color_name);

    Buffer::set_clear_color(0.2, 0.3, 0.3, 1.0);

    // Main Loop
    'main_loop: loop {
        if win.inner_win.should_close() {
            break;
        }

        /* Handle events of this frame */
        win.glfw.poll_events(); // check and call events
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
        unsafe {
            // Clear bits
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        shader_program.bind();
        let time = win.glfw.get_time() as f32;
        let color = (time.sin() / 2.0) + 0.5;
        shader_program.set_uniform_4f(uniform_color_location, color, color, color, color);

        unsafe {
            // Draw call
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
}
