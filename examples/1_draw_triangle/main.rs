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
        // Load Gl Functions from window
        gl33::global_loader::load_global_gl(&(|func_name| win.get_proc_address(func_name)));

        // Specify clear color
        glClearColor(0.2, 0.3, 0.3, 1.0);
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
