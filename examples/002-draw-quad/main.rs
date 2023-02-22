#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// This example is only about how to draw a simple triangle.
/// It is involved about:
/// * Vertex Array Object
/// * Vertex Buffer Object
/// * Shader and `in` & `out` keyword
/// * Draw call: `glDrawArrays()`
/// It isn't involved about "Index Buffer" and "uniform" keyword in shader.
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
    type TriIndexes = [u32; 3]; // vertex indexes for a triangle primitive
    const VERTICES: [Vertex; 4] = [
        [0.5, 0.5, 0.0],
        [0.5, -0.5, 0.0],
        [-0.5, -0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ];
    const INDICES: [TriIndexes; 2] = [[1, 2, 3], [0, 1, 3]];
    let shader_program: u32;

    unsafe {
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
        // Bind VBO as ARRAY_BUFFER
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        // Set buffer data
        gl::BufferData(
            gl::ARRAY_BUFFER,
            core::mem::size_of_val(&VERTICES) as isize,
            VERTICES.as_ptr().cast(),
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
        const VERTEX_SHADER: &str = include_str!("../../assets/shaders/solid.vert");
        const FRAGMENT_SHADER: &str = include_str!("../../assets/shaders/solid.frag");

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
        check_shader_compile(vertex_shader);
        check_shader_compile(fragment_shader);

        // Create/Attach/Link shader program
        shader_program = gl::CreateProgram();
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
        if win.should_close() {
            break;
        }

        /* Handle events of this frame */
        for (_timestamp, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Close => break 'main_loop,
                glfw::WindowEvent::Key(key, _scancode, action, _modifier) => {
                    if key == glfw::Key::Escape && action == glfw::Action::Press {
                        win.set_should_close(true);
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

            // Draw call
            gl::DrawElements(
                gl::TRIANGLES,
                INDICES.len() as i32 * 3,
                gl::UNSIGNED_INT,
                0 as *const _,
            );
        }
        
        // check and call events
        glfw.poll_events();
        // Swap buffers of window
        win.swap_buffers();
    }

    unsafe {
        gl::DeleteProgram(shader_program);
    }
    win.close();
    drop(glfw); // this will call `glfwTerminate`
}
