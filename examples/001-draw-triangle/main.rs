#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Ok;
/// This example is only about how to draw a simple triangle.
/// It is involved about:
/// * Vertex Array Object
/// * Vertex Buffer Object
/// * Shader and `in` & `out` keyword
/// * Draw call: `glDrawArrays()`
/// It isn't involved about "Index Buffer" and "uniform" keyword in shader.
use learn_opengl_rs as learn;

use gl::types::*;
use glfw::Context;
use tracing::debug;

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

fn main() -> anyhow::Result<()> {
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
    const VERTICES: [Vertex; 3] = [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];

    /* Vertex Array Object */
    // Generate VAO
    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
    }
    assert_ne!(vao, 0);
    // Bind VAO
    unsafe { gl::BindVertexArray(vao) }

    /* Vertex Buffer Object */
    // Generate VBO
    let mut vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
    }
    assert_ne!(vbo, 0);
    unsafe {
        // Bind VBO as ARRAY_BUFFER
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        // Set buffer data
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
            // attribute index 0 is the target
            0,
            // attribute is : 3 * float
            3,
            gl::FLOAT,
            // coordinate already normalized
            gl::FALSE,
            // TODO: handle overflow
            core::mem::size_of::<Vertex>().try_into()?,
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
    const VERTEX_SHADER: &str = include_str!("../../assets/shaders/001-solid.vert");
    const FRAGMENT_SHADER: &str = include_str!("../../assets/shaders/001-solid.frag");

    // Make vertex & fragment shader
    let vertex_shader = unsafe { gl::CreateShader(gl::VERTEX_SHADER) };
    assert_ne!(vertex_shader, 0);
    unsafe {
        gl::ShaderSource(
            vertex_shader,
            1,
            &(VERTEX_SHADER.as_bytes().as_ptr().cast()),
            &(VERTEX_SHADER.len().try_into()?),
        );
    }
    let fragment_shader = unsafe { gl::CreateShader(gl::FRAGMENT_SHADER) };
    assert_ne!(fragment_shader, 0);
    unsafe {
        gl::ShaderSource(
            fragment_shader,
            1,
            &(FRAGMENT_SHADER.as_bytes().as_ptr().cast()),
            &(FRAGMENT_SHADER.len().try_into()?),
        );
    }

    unsafe {
        // Compile vertex & fragment shader
        gl::CompileShader(vertex_shader);
        gl::CompileShader(fragment_shader);
    }
    // Check shader object compile result
    check_shader_compile(vertex_shader);
    check_shader_compile(fragment_shader);

    /* Shader Program */
    // Create/Attach/Link shader program
    let shader_program = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
    }
    // Check shader program link result
    check_shader_link(shader_program);

    unsafe {
        // Delete shader object after link
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        gl::UseProgram(shader_program);
    }

    /* Extra Settings */
    // Specify clear color
    unsafe { gl::ClearColor(0.2, 0.3, 0.3, 1.0) }

    /* Main Loop */
    'main_loop: loop {
        if win.inner_win.should_close() {
            break;
        }

        /* Handle events of this frame */
        if !win.handle_events() {
            break 'main_loop;
        };

        /* On Update (Drawing) */
        unsafe {
            // Clear bits
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Bind shader program
            gl::UseProgram(shader_program);

            // Bind VAO
            gl::BindVertexArray(vao);

            // Draw call
            gl::DrawArrays(gl::TRIANGLES, 0, VERTICES.len().try_into()?);
        }

        win.inner_win.swap_buffers();
    }

    unsafe { gl::DeleteProgram(shader_program) }
    win.close();

    Ok(())
}
