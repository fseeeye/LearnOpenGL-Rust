#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use learn::{
    Buffer, BufferType, BufferUsage, ShaderProgram, VertexArray, VertexBufferLayout, Window,
};
/// This example is about how to abstract safe gl funcs.
/// TODO:
/// * Add glGetError() after any gl funcs calling.
use learn_opengl_rs as learn;

fn main() {
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

    Buffer::set_clear_color(0.2, 0.3, 0.3, 1.0);

    /* Vertex Array Object */
    let mut vao = VertexArray::new().expect("Failed to make a VAO.");
    vao.bind();

    /* Vertex Buffer Object */
    let vbo = Buffer::new(BufferType::Array).expect("Failed to make a VBO");
    vbo.bind();
    vbo.set_buffer_data(bytemuck::cast_slice(&VERTICES), BufferUsage::StaticDraw);

    // Set vbo and its layout to VAO
    let mut buffer_layout = VertexBufferLayout::new();
    buffer_layout.push(gl::FLOAT, 3); // Vertex is [f32; 3]
    vao.add_vertex_buffer(&vbo, &buffer_layout);

    /* Index Buffer Object */
    let ibo = Buffer::new(BufferType::ElementArray).expect("Failed to make a IBO");
    ibo.bind();
    ibo.set_buffer_data(bytemuck::cast_slice(&INDICES), BufferUsage::StaticDraw);

    /* Shader */
    let shader_program = ShaderProgram::create_from_source(
        include_str!("../../assets/shaders/solid.vert.glsl"),
        include_str!("../../assets/shaders/solid.frag.glsl"),
    )
    .unwrap();
    shader_program.bind();

    // Main Loop
    'main_loop: loop {
        if win.should_close() {
            break;
        }

        /* Handle events of this frame */
        for (_timestamp, event) in win.poll_events() {
            match event {
                glfw::WindowEvent::Close => break 'main_loop,
                glfw::WindowEvent::Size(w, h) => {
                    println!("Resizing to ({}, {})", w, h);
                }
                _ => (),
            }
        }

        /* On Update (Drawing) */
        unsafe {
            // Clear bits
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw call
            gl::DrawElements(
                gl::TRIANGLES,
                INDICES.len() as i32 * 3,
                gl::UNSIGNED_INT,
                0 as *const _,
            );
        }
        // Swap buffers of window
        win.swap_buffers();
    }

    shader_program.close();
    win.close();
}
