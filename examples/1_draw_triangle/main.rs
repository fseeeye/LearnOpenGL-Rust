#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::CString;

/// This example is only about how to draw a simple triangle.
/// It is involved about:
/// * Vertex Array Object
/// * Vertex Buffer Object
/// * Shader and `in` & `out` keyword
/// * Draw call: `glDrawArrays()`
/// It isn't involved about "Index Buffer" and "uniform" keyword in shader.

use gl::types::*;

use beryllium::events::Event;
use beryllium::init::InitFlags;
#[cfg(target_os = "macos")]
use beryllium::video::GlContextFlags;
use beryllium::video::{CreateWinArgs, GlProfile, GlSwapInterval};

const SHADER_INFO_BUFF_SIZE: usize = 1024;

fn check_shader_compile(shader_obj: u32, info_buff_size: usize) {
    let mut compile_success = 0;

    unsafe {
        gl::GetShaderiv(shader_obj, gl::COMPILE_STATUS, &mut compile_success);
        if compile_success == 0 {
            let mut log_buf: Vec<u8> = Vec::with_capacity(info_buff_size);
            let mut log_len = 0;

            gl::GetShaderInfoLog(
                shader_obj,
                info_buff_size as i32,
                &mut log_len,
                log_buf.as_mut_ptr() as *mut GLchar,
            );
            log_buf.set_len(log_len.try_into().unwrap());

            panic!(
                "Vertex Shader Compile Error: {}",
                String::from_utf8_lossy(&log_buf)
            );
        }
    }
}

fn check_shader_link(shader_program: u32) {
    let mut link_success = 0;

    unsafe {
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut link_success);
        if link_success == 0 {
            let mut log_buf: Vec<u8> = Vec::with_capacity(SHADER_INFO_BUFF_SIZE);
            let mut log_len = 0;
            gl::GetProgramInfoLog(
                shader_program,
                SHADER_INFO_BUFF_SIZE as i32,
                &mut log_len,
                log_buf.as_mut_ptr() as *mut GLchar,
            );
            log_buf.set_len(log_len.try_into().unwrap());
            panic!(
                "Fragment Shader Compile Error: {}",
                String::from_utf8_lossy(&log_buf)
            );
        }
    }
}

fn main() {
    let sdl = beryllium::Sdl::init(InitFlags::EVERYTHING);

    // Setting up GL Context in window: use OpenGL 3.3 with core profile
    sdl.set_gl_context_major_version(3).unwrap();
    sdl.set_gl_context_minor_version(3).unwrap();
    sdl.set_gl_profile(GlProfile::Core).unwrap();
    #[cfg(target_os = "macos")]
    {
        sdl.set_gl_context_flags(GlContextFlags::FORWARD_COMPATIBLE)
            .unwrap();
    }

    // Make Window
    let win_args = CreateWinArgs {
        title: "Hello World",
        width: 800,
        height: 600,
        allow_high_dpi: true,
        borderless: false,
        resizable: true,
    };
    let win = sdl
        .create_gl_window(win_args)
        .expect("Failed to create window & OpenGL Context.");

    // Enable Vsync
    win.set_swap_interval(GlSwapInterval::Vsync).unwrap();

    type Vertex = [f32; 3]; // x, y, z in Normalized Device Context (NDC) coordinates
    const TRIANGLE: [Vertex; 3] = [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];
    unsafe {
        // Load Gl Functions from window
        gl::load_with(|symbol| {
            // ref: https://stackoverflow.com/questions/49203561/how-do-i-convert-a-str-to-a-const-u8
            let c_str = CString::new(symbol).unwrap(); // with NUL-terminated string
            win.get_proc_address(c_str.as_ptr() as *const u8)
    });

        // Specify clear color
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);

        /* Vertex Array Object */
        // Generate VAO
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        assert_ne!(vao, 0);
        // Bind VAO
        gl::BindVertexArray(vao);

        /* Vertex Buffer Object */
        // Generate VBO
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        assert_ne!(vbo, 0);
        // Bind VBO
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        // Set buffer data
        gl::BufferData(
            gl::ARRAY_BUFFER,
            core::mem::size_of_val(&TRIANGLE) as isize,
            TRIANGLE.as_ptr().cast(),
            gl::STATIC_DRAW,
        );

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

        // Make vertex & fragment shader
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

        // Compile vertex & fragment shader
        gl::CompileShader(vertex_shader);
        gl::CompileShader(fragment_shader);

        // Check shader object compile result
        check_shader_compile(vertex_shader, SHADER_INFO_BUFF_SIZE);
        check_shader_compile(fragment_shader, SHADER_INFO_BUFF_SIZE);

        // Create/Attach/Link shader program
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        // Check shader program link result
        check_shader_link(shader_program);

        // Delete shader object after link
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        // Bind shader program
        gl::UseProgram(shader_program);
    }

    // Main Loop
    'main_loop: loop {
        // Handle events of this frame
        while let Some((event, _timestamp)) = sdl.poll_events() {
            match event {
                Event::Quit => break 'main_loop,
                _ => (),
            }
        }

        // On Update
        unsafe {
            // Clear bits
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw call
            gl::DrawArrays(gl::TRIANGLES, 0, TRIANGLE.len().try_into().unwrap());
        }
        // Swap buffers of window
        win.swap_window();
    }
}
