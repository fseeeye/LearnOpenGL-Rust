#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// This example is about how to abstract safe gl funcs.
/// TODO:
/// * Add glGetError() after any gl funcs calling.
use learn_opengl_rs as learn;
use learn::{Buffer, BufferType, BufferUsage, VertexArray, ShaderProgram};

use glfw::Context;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // Setting up GL Context in window: use OpenGL 3.3 with core profile
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    #[cfg(target_os = "macos")]
    {
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    }

    // Make window
    let (mut win, events) = glfw
        .create_window(800, 600, "Simple Triangle", glfw::WindowMode::Windowed)
        .unwrap();

    // Setup window
    win.make_current(); // `glfwMakeContextCurrent`
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1)); // Enable Vsync
    win.set_all_polling(true); // start polling

    // Load Gl Functions from window
    gl::load_with(|symbol| win.get_proc_address(symbol));

    type Vertex = [f32; 3]; // x, y, z in Normalized Device Context (NDC) coordinates
    const TRIANGLE: [Vertex; 3] = [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];
    unsafe {
        // Specify clear color
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
    }

    /* Vertex Array Object */
    let vao = VertexArray::new().expect("Failed to make a VAO.");
    vao.bind();

    /* Vertex Buffer Object */
    let vbo = Buffer::new().expect("Failed to make a VBO");
    vbo.bind(BufferType::Array);
    vbo.set_buffer_data(
        bytemuck::cast_slice(&TRIANGLE),
        BufferType::Array,
        BufferUsage::StaticDraw,
    );

    unsafe {
        /* Vertex Attribute */
        gl::VertexAttribPointer(
            // attribute index 0 is the target
            0,
            // attribute is : 3 * float
            3,
            gl::FLOAT,
            // coordinate already normalized
            gl::FALSE,
            // TODO: handle overflow
            core::mem::size_of::<Vertex>().try_into().unwrap(),
            // We have to convert the pointer location using usize values and then cast to a const pointer
            // once we have our usize. We do not want to make a null pointer and then offset it with the `offset`
            // method. That's gonna generate an out of bounds pointer, which is UB. We could try to remember to use the
            // `wrapping_offset` method, or we could just do all the math in usize and then cast at the end.
            // I prefer the latter option.
            0 as _,
        );
        gl::EnableVertexAttribArray(0);
    }

    /* Shader */
    const VERTEX_SHADER: &str = r#"
    #version 330 core

    layout (location = 0) in vec3 pos;

    void main() {
        gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
    }"#;
    const FRAGMENT_SHADER: &str = r#"
    #version 330 core

    out vec4 final_color;

    void main() {
        final_color = vec4(1.0, 0.5, 0.2, 1.0);
    }"#;

    let program = ShaderProgram::from_vert_frag(VERTEX_SHADER, FRAGMENT_SHADER).unwrap();
    program.bind();

    // Main Loop
    'main_loop: loop {
        if win.should_close() {
            break;
        }

        /* Handle events of this frame */
        glfw.poll_events();
        for (_timestamp, event) in glfw::flush_messages(&events) {
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
            gl::DrawArrays(gl::TRIANGLES, 0, TRIANGLE.len().try_into().unwrap());
        }
        // Swap buffers of window
        win.swap_buffers();
    }

    program.delete();
}
