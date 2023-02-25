#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// This example is about how to draw a simple quad.
/// It is involved about:
/// * Index Buffer Object
/// * Shader uniform
/// * Draw call: `DrawElements()`
use learn_opengl_rs as learn;

use std::ffi::CString;

use glfw::Context;
use tracing::trace;

fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set default subscriber");

    /* Window */
    let mut win = learn::Window::new("Simple Triangle", 800, 600, glfw::WindowMode::Windowed)
        .expect("Failed to create window.");
    win.setup();
    win.load_gl();

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
    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
    }
    assert_ne!(vao, 0);
    unsafe { gl::BindVertexArray(vao) }

    /* Vertex Buffer Object */
    let mut vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
    }
    assert_ne!(vbo, 0);
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            core::mem::size_of_val(&VERTICES) as isize,
            VERTICES.as_ptr().cast(),
            gl::STATIC_DRAW,
        );
    }

    /* Vertex Attribute */
    unsafe {
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            core::mem::size_of::<Vertex>().try_into().unwrap(),
            0 as _,
        );
        gl::EnableVertexAttribArray(0);
    }

    /* Index Buffer Object */
    // Generate IBO
    let mut ibo = 0;
    unsafe {
        gl::GenBuffers(1, &mut ibo);
    }
    assert_ne!(ibo, 0);
    unsafe {
        // Bind IBO as ELEMENT_ARRAY_BUFFER
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
    const VERTEX_SHADER: &str = include_str!("../../assets/shaders/002-uniform.vert");
    const FRAGMENT_SHADER: &str = include_str!("../../assets/shaders/002-uniform.frag");

    let vertex_shader = unsafe { gl::CreateShader(gl::VERTEX_SHADER) };
    assert_ne!(vertex_shader, 0);
    unsafe {
        gl::ShaderSource(
            vertex_shader,
            1,
            &(VERTEX_SHADER.as_bytes().as_ptr().cast()),
            &(VERTEX_SHADER.len().try_into().unwrap()),
        );
    }
    let fragment_shader = unsafe { gl::CreateShader(gl::FRAGMENT_SHADER) };
    assert_ne!(fragment_shader, 0);
    unsafe {
        gl::ShaderSource(
            fragment_shader,
            1,
            &(FRAGMENT_SHADER.as_bytes().as_ptr().cast()),
            &(FRAGMENT_SHADER.len().try_into().unwrap()),
        );
    }

    unsafe {
        gl::CompileShader(vertex_shader);
        gl::CompileShader(fragment_shader);
    }
    match learn::Shader::check_compile_result(vertex_shader) {
        Ok(_) => {}
        Err(e) => {
            panic!("Vertex Shader({}) compile error: {}", vertex_shader, e)
        }
    }
    match learn::Shader::check_compile_result(fragment_shader) {
        Ok(_) => {}
        Err(e) => {
            panic!("Fragment Shader({}) compile error: {}", fragment_shader, e)
        }
    }

    /* Shader Program */
    let shader_program = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
    }
    match learn::ShaderProgram::check_link_result(shader_program) {
        Ok(_) => {}
        Err(e) => {
            panic!("Shader Program({}) link error: {}", shader_program, e)
        }
    }

    unsafe {
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
    }

    // Get uniform location
    let uniform_color_name = CString::new("dyn_color").unwrap();
    let uniform_color_location =
        unsafe { gl::GetUniformLocation(shader_program, uniform_color_name.as_ptr()) };

    unsafe { gl::UseProgram(shader_program) }

    unsafe { gl::ClearColor(0.2, 0.3, 0.3, 1.0) }

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
        let time = win.glfw.get_time() as f32;
        let color = (time.sin() / 2.0) + 0.5;
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(shader_program);
            // Send uniform value - 'dynamic color'
            gl::Uniform4f(uniform_color_location, color, color, color, color);

            gl::BindVertexArray(vao);

            // Bind IBO
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);

            gl::DrawElements(
                gl::TRIANGLES,
                INDICES.len() as i32 * 3,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }

        win.inner_win.swap_buffers();
    }

    unsafe { gl::DeleteProgram(shader_program) }
    win.close();
}
