#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use beryllium::events::Event;
use beryllium::init::InitFlags;
#[cfg(target_os = "macos")]
use beryllium::video::GlContextFlags;
use beryllium::video::{CreateWinArgs, GlProfile};

use gl33::global_loader::*;

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

    unsafe {
        type Vertex = [f32; 3]; // x, y, z in Normalized Device Context (NDC) coordinates
        const TRIANGLE: [Vertex; 3] = [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];

        // Load Gl Functions from window
        gl33::global_loader::load_global_gl(&(|func_name| win.get_proc_address(func_name)));

        // Specify clear color
        glClearColor(0.2, 0.3, 0.3, 1.0);

        /* Vertex Array Object */
        // Generate VAO
        let mut vao = 0;
        glGenVertexArrays(1, &mut vao);
        assert_ne!(vao, 0);
        // Bind VAO
        glBindVertexArray(vao);

        /* Vertex Buffer Object */
        // Generate VBO
        let mut vbo = 0;
        glGenBuffers(1, &mut vbo);
        assert_ne!(vbo, 0);
        // Bind VBO
        glBindBuffer(gl33::GL_ARRAY_BUFFER, vbo);
        // Set buffer data
        glBufferData(
            gl33::GL_ARRAY_BUFFER,
            core::mem::size_of_val(&TRIANGLE) as isize,
            TRIANGLE.as_ptr().cast(),
            gl33::GL_STATIC_DRAW,
        );

        /* Vertex Attribute */
        glVertexAttribPointer(
            // attribute index 0 is the target
            0, 
            // attribute is : 3 * float
            3,
            gl33::GL_FLOAT,
             // coordinate already normalized
            gl33::GL_FALSE.0 as u8,
            // TODO: handle overflow
            core::mem::size_of::<Vertex>().try_into().unwrap(), 
            // We have to convert the pointer location using usize values and then cast to a const pointer 
            // once we have our usize. We do not want to make a null pointer and then offset it with the `offset` 
            // method. That's gonna generate an out of bounds pointer, which is UB. We could try to remember to use the 
            // `wrapping_offset` method, or we could just do all the math in usize and then cast at the end.
            // I prefer the latter option.
            0 as *const _,
        );
        glEnableVertexAttribArray(0);
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
    }
}
