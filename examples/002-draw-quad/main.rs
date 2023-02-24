#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// This example is only about how to draw a simple triangle.
/// It is involved about:
/// * Vertex Array Object
/// * Vertex Buffer Object
/// * Shader and `in` & `out` keyword
/// * Draw call: `glDrawArrays()`
/// It isn't involved about "Index Buffer" and "uniform" keyword in shader.
use learn::Window;
use learn_opengl_rs as learn;

use std::ffi::CString;

use gl::types::*;
use glfw::Context;
use tracing::{debug, trace};

fn check_shader_compile(shader_obj: u32) {
    let mut is_success = gl::FALSE as GLint;
    unsafe { gl::GetShaderiv(shader_obj, gl::COMPILE_STATUS, &mut is_success) }

    if is_success == gl::FALSE as GLint {
        let mut log_cap = 0;
        unsafe { gl::GetShaderiv(shader_obj, gl::INFO_LOG_LENGTH, &mut log_cap) }
        let mut log_buf: Vec<u8> = Vec::with_capacity(log_cap as usize);

        let mut log_len = 0i32;
        unsafe {
            gl::GetShaderInfoLog(
                shader_obj,
                log_buf.capacity() as i32,
                &mut log_len,
                log_buf.as_mut_ptr() as *mut GLchar,
            );
            log_buf.set_len(log_len as usize);
        }

        panic!(
            "Shader compile error: {}",
            String::from_utf8_lossy(&log_buf)
        );
    } else {
        debug!("Create shader({}) successfully!", shader_obj);
    }
}

fn check_shader_link(shader_program: u32) {
    let mut is_success = gl::FALSE as GLint;
    unsafe { gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut is_success) }

    if is_success == gl::FALSE as GLint {
        let mut log_cap = 0;
        unsafe { gl::GetProgramiv(shader_program, gl::INFO_LOG_LENGTH, &mut log_cap) }
        let mut log_buf: Vec<u8> = Vec::with_capacity(log_cap as usize);

        let mut log_len = 0i32;
        unsafe {
            gl::GetProgramInfoLog(
                shader_program,
                log_buf.capacity() as i32,
                &mut log_len,
                log_buf.as_mut_ptr() as *mut GLchar,
            );
            log_buf.set_len(log_len as usize);
        }

        panic!(
            "Shader Program link error: {}",
            String::from_utf8_lossy(&log_buf)
        );
    } else {
        debug!("Create shader program({}) successfully!", shader_program);
    }
}

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

    // Prepare vars
    type Vertex = [f32; 3]; // x, y, z in Normalized Device Context (NDC) coordinates
    type TriIndexes = [u32; 3]; // vertex indexes for a triangle primitive
    const VERTICES: [Vertex; 4] = [
        [0.5, 0.5, 0.0],
        [0.5, -0.5, 0.0],
        [-0.5, -0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ];
    const INDICES: [TriIndexes; 2] = [[1, 2, 3], [0, 1, 3]];
    let shader_program: u32;
    let uniform_color_name = CString::new("dyn_color").unwrap();
    let uniform_color_location: i32;

    unsafe {
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);

        /* Vertex Array Object */
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        assert_ne!(vao, 0);
        gl::BindVertexArray(vao);

        /* Vertex Buffer Object */
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        assert_ne!(vbo, 0);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            core::mem::size_of_val(&VERTICES) as isize,
            VERTICES.as_ptr().cast(),
            gl::STATIC_DRAW,
        );

        /* Vertex Attribute */
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            core::mem::size_of::<Vertex>().try_into().unwrap(),
            0 as _,
        );
        gl::EnableVertexAttribArray(0);

        /* Index Buffer Object */
        // Generate IBO
        let mut ibo = 0;
        gl::GenBuffers(1, &mut ibo);
        assert_ne!(ibo, 0);
        // Bind IBO as ELEMENT_ARRAY_BUFFER
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        // Set buffer data
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            core::mem::size_of_val(&INDICES) as isize,
            INDICES.as_ptr().cast(),
            gl::STATIC_DRAW,
        );

        /* Shader */
        const VERTEX_SHADER: &str = include_str!("../../assets/shaders/002-uniform.vert");
        const FRAGMENT_SHADER: &str = include_str!("../../assets/shaders/002-uniform.frag");

        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        assert_ne!(vertex_shader, 0);
        gl::ShaderSource(
            vertex_shader,
            1,
            &(VERTEX_SHADER.as_bytes().as_ptr().cast()),
            &(VERTEX_SHADER.len().try_into().unwrap()),
        );
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        assert_ne!(fragment_shader, 0);
        gl::ShaderSource(
            fragment_shader,
            1,
            &(FRAGMENT_SHADER.as_bytes().as_ptr().cast()),
            &(FRAGMENT_SHADER.len().try_into().unwrap()),
        );

        gl::CompileShader(vertex_shader);
        gl::CompileShader(fragment_shader);

        check_shader_compile(vertex_shader);
        check_shader_compile(fragment_shader);

        shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        check_shader_link(shader_program);

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        // Get uniform location
        uniform_color_location =
            gl::GetUniformLocation(shader_program, uniform_color_name.as_ptr());

        gl::UseProgram(shader_program);
    }

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
        let time = win.glfw.get_time() as f32;
        let color = (time.sin() / 2.0) + 0.5;
        unsafe {
            // Send uniform value - 'dynamic color'
            gl::Uniform4f(uniform_color_location, color, color, color, color);

            // Clear bits
            gl::Clear(gl::COLOR_BUFFER_BIT);

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

    unsafe {
        gl::DeleteProgram(shader_program);
    }
    win.close();
}
