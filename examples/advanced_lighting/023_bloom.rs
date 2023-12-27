//! This example has more infos about blooming.

// remove console window : https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ffi::CString, path::PathBuf, sync::Mutex};

use anyhow::bail;
use gl::types::*;

use learn::{
    clear_color, set_clear_color, Buffer, BufferBit, BufferType, BufferUsage, Camera, Model,
    ShaderProgram, VertexArray, VertexDescription, WinitWindow,
};
use learn_opengl_rs as learn;

use nalgebra as na;
use nalgebra_glm as glm;
use tracing::error;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

/* Screen info */
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const BACKGROUND_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WINDOW_TITLE: &str = "Bloom";
const SCREEN_VERTICES: [[f32; 5]; 4] = [
    // vertex attributes for a quad that fills the entire screen in Normalized Device Coordinates.
    [-1.0, 1.0, 0.0, 0.0, 1.0],
    [-1.0, -1.0, 0.0, 0.0, 0.0],
    [1.0, 1.0, 0.0, 1.0, 1.0],
    [1.0, -1.0, 0.0, 1.0, 0.0],
];

/* Camera data */
const CAMERA_POS: [f32; 3] = [0.0, 0.0, 5.0];
const CAMERA_LOOK_AT: [f32; 3] = [0.0, 0.0, -1.0];
const CAMERA_UP: [f32; 3] = [0.0, 1.0, 0.0];
const PROJECTION_FOV: f32 = std::f32::consts::FRAC_PI_4;
const PROJECTION_NEAR: f32 = 0.1;
const PROJECTION_FAR: f32 = 100.0;

/* Scene data */
const LIGHT_POS: [[f32; 3]; 4] = [
    [0.0, 0.5, 1.5],   // back light
    [-4.0, 0.5, -3.0], // top light
    [3.0, 0.5, 1.0],   // front light
    [-0.8, 2.4, -1.0], // right light
];
const LIGHT_COLOR: [[f32; 3]; 4] = [
    [5.0, 5.0, 5.0],  // back light
    [10.0, 0.0, 0.0], // top light
    [0.0, 0.0, 15.0], // front light
    [0.0, 5.0, 0.0],  // right light
];

/* Tone Mapping data */
static ENABLE_BLOOM: Mutex<bool> = Mutex::new(true);
static EXPOSURE: Mutex<f32> = Mutex::new(1.0);

struct Renderer {
    cube_model: Model,

    color_textures: [u32; 2],
    hdr_fbo: u32,
    object_shader: ShaderProgram,
    light_shader: ShaderProgram,

    blur_fbos: [u32; 2],
    blur_color_textures: [u32; 2],
    blur_shader: ShaderProgram,

    screen_vao: VertexArray,
    tone_mapping_shader: ShaderProgram,
}

impl Renderer {
    pub fn new(win: &WinitWindow) -> anyhow::Result<Self> {
        /* Extra Settings */

        // Configure global opengl state
        unsafe {
            // Enable Depth Test
            gl::Enable(gl::DEPTH_TEST);
        };
        // Set clear color
        set_clear_color(
            BACKGROUND_COLOR[0],
            BACKGROUND_COLOR[1],
            BACKGROUND_COLOR[2],
            BACKGROUND_COLOR[3],
        );

        /* Object Models */

        // cube object
        let cube_model = Model::new(PathBuf::from("assets/models/cube_wood/cube.obj"))?;
        // screen quad
        let screen_vao = VertexArray::new()?;
        let screen_vbo = Buffer::new(BufferType::VertexBuffer)?;
        screen_vbo.bind();
        screen_vbo.set_buffer_data(SCREEN_VERTICES.as_slice(), BufferUsage::StaticDraw);
        screen_vao.bind();
        let mut screen_vertex_desc = VertexDescription::new();
        screen_vertex_desc.add_attribute(gl::FLOAT, 3); // set coords attribute
        screen_vertex_desc.add_attribute(gl::FLOAT, 2); // set Texture coord attribute
        screen_vertex_desc.bind_to(&screen_vbo, Some(&screen_vao));

        /* Shaders */

        // Create shader of object
        let object_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/advanced_lighting/023-object.vert"),
            include_str!("../../assets/shaders/advanced_lighting/023-object.frag"),
        )?;
        for i in 0..LIGHT_POS.len() {
            let light_name = format!("lights[{}].position", i);
            object_shader.set_uniform_3f(
                CString::new(light_name)?.as_c_str(),
                LIGHT_POS[i][0],
                LIGHT_POS[i][1],
                LIGHT_POS[i][2],
            );
            let light_color_name = format!("lights[{}].color", i);
            object_shader.set_uniform_3f(
                CString::new(light_color_name)?.as_c_str(),
                LIGHT_COLOR[i][0],
                LIGHT_COLOR[i][1],
                LIGHT_COLOR[i][2],
            );
        }

        // Create shader of lights
        let light_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/advanced_lighting/023-light.vert"),
            include_str!("../../assets/shaders/advanced_lighting/023-light.frag"),
        )?;

        // Create shader of Gaussian blur
        let blur_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/advanced_lighting/023-gaussian-blur.vert"),
            include_str!("../../assets/shaders/advanced_lighting/023-gaussian-blur.frag"),
        )?;

        // Create shader of Tone Mapping
        let tone_mapping_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/advanced_lighting/023-bloom-final.vert"),
            include_str!("../../assets/shaders/advanced_lighting/023-bloom-final.frag"),
        )?;

        /* Floating-point Framebuffer for HDR rendering */

        // Create framebuffer
        let mut hdr_fbo = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut hdr_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, hdr_fbo);
        }
        // Create texture as color attachment
        let mut color_textures: [u32; 2] = [0; 2];
        unsafe {
            // Create two color textures for color and brightness space result
            gl::GenTextures(2, color_textures.as_mut_ptr());

            for (i, color_texture) in color_textures.iter().enumerate() {
                gl::BindTexture(gl::TEXTURE_2D, *color_texture);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGB16F as GLint, // use RGB16F to restore HDR content
                    win.get_window_size().0.try_into()?,
                    win.get_window_size().1.try_into()?,
                    0,
                    gl::RGB,
                    gl::FLOAT,
                    core::ptr::null(),
                );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                // Attach color texture to framebuffer
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0 + i as GLenum,
                    gl::TEXTURE_2D,
                    *color_texture,
                    0,
                );
            }
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        // Create renderbuffer as depth attachment
        let mut depth_rbo = 0;
        unsafe {
            gl::GenRenderbuffers(1, &mut depth_rbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, depth_rbo);
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH_COMPONENT,
                win.get_window_size().0.try_into()?,
                win.get_window_size().1.try_into()?,
            );
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
            // Attach depth rbo to framebuffer
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::RENDERBUFFER,
                depth_rbo,
            );
        }
        // Tell OpenGL which color attachments we'll use (of this framebuffer) for rendering
        let attachments: [u32; 2] = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1];
        unsafe {
            gl::DrawBuffers(2, attachments.as_ptr());
        }
        // Check framebuffer status
        let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };
        if status != gl::FRAMEBUFFER_COMPLETE {
            bail!("Framebuffer is not complete!");
        }
        // Bind framebuffer to default
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        /* Framebuffers for Gaussian blur rendering */
        let mut blur_fbos = [0; 2];
        let mut blur_color_textures = [0; 2];
        unsafe {
            gl::GenFramebuffers(2, blur_fbos.as_mut_ptr());
            gl::GenTextures(2, blur_color_textures.as_mut_ptr());

            for i in 0..2 {
                gl::BindFramebuffer(gl::FRAMEBUFFER, blur_fbos[i]);
                gl::BindTexture(gl::TEXTURE_2D, blur_color_textures[i]);

                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGB16F as GLint,
                    win.get_window_size().0.try_into()?,
                    win.get_window_size().1.try_into()?,
                    0,
                    gl::RGB,
                    gl::FLOAT,
                    core::ptr::null(),
                );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_WRAP_S,
                    gl::CLAMP_TO_EDGE as GLint,
                );
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_WRAP_T,
                    gl::CLAMP_TO_EDGE as GLint,
                );

                // Attach color texture to framebuffer
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0 as GLenum,
                    gl::TEXTURE_2D,
                    blur_color_textures[i],
                    0,
                );

                // Check framebuffer status
                let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
                if status != gl::FRAMEBUFFER_COMPLETE {
                    bail!("Gaussian Blur Framebuffer is not complete!");
                }

                gl::BindTexture(gl::TEXTURE_2D, 0);
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            }
        }

        Ok(Self {
            cube_model,
            color_textures,
            hdr_fbo,
            object_shader,
            light_shader,
            blur_fbos,
            blur_color_textures,
            blur_shader,
            screen_vao,
            tone_mapping_shader,
        })
    }

    pub fn redraw(
        &self,
        win: &WinitWindow,
        camera: &Camera,
        _delta_time: f32,
    ) -> anyhow::Result<()> {
        clear_color(
            (BufferBit::ColorBufferBit as GLenum | BufferBit::DepthBufferBit as GLenum)
                as gl::types::GLbitfield,
        );

        // View Matrix
        let view_name = CString::new("view")?;
        let object_view_matrix = camera.get_lookat_matrix();

        // Projection Matrix
        let (window_width, window_height) = win.get_window_size();
        let projection_matrix = na::Perspective3::new(
            (window_width as f32) / (window_height as f32),
            PROJECTION_FOV,
            PROJECTION_NEAR,
            PROJECTION_FAR,
        )
        .to_homogeneous(); // Perspective projection
        let projection_name = CString::new("projection")?;

        /* Pass 1 : Render scene */
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.hdr_fbo);
        }

        clear_color(
            (BufferBit::ColorBufferBit as GLenum | BufferBit::DepthBufferBit as GLenum)
                as gl::types::GLbitfield,
        );

        self.object_shader.bind();
        self.object_shader
            .set_uniform_mat4fv(view_name.as_c_str(), &object_view_matrix);
        self.object_shader
            .set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);
        self.object_shader.set_uniform_3f(
            CString::new("camera_pos")?.as_c_str(),
            camera.get_pos().x,
            camera.get_pos().y,
            camera.get_pos().z,
        );

        self.render_scence(&self.object_shader)?;

        /* Pass 2 : Draw lights */

        self.light_shader.bind();
        self.light_shader
            .set_uniform_mat4fv(view_name.as_c_str(), &object_view_matrix);
        self.light_shader
            .set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);

        for i in 0..LIGHT_POS.len() {
            let mut light_model_matrix = na::Matrix4::identity();
            light_model_matrix = glm::translate(
                &light_model_matrix,
                &glm::vec3(LIGHT_POS[i][0], LIGHT_POS[i][1], LIGHT_POS[i][2]),
            );
            light_model_matrix = glm::scale(&light_model_matrix, &glm::vec3(0.1, 0.1, 0.1));
            self.light_shader
                .set_uniform_mat4fv(CString::new("model")?.as_c_str(), &light_model_matrix);

            self.light_shader.set_uniform_3f(
                CString::new("light_color")?.as_c_str(),
                LIGHT_COLOR[i][0],
                LIGHT_COLOR[i][1],
                LIGHT_COLOR[i][2],
            );

            self.cube_model.draw(&self.light_shader, "material")?;
        }

        /* Pass 3 : Gaussian Blur */

        let mut horizontal_blur = true;
        let mut first_iteration = true;
        let blur_amount = 5;

        self.blur_shader.bind();

        for _ in 0..(blur_amount * 2) {
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.blur_fbos[horizontal_blur as usize]);
            }
            // Prepare uniforms
            self.blur_shader.set_uniform_1i(
                CString::new("horizontal_blur")?.as_c_str(),
                horizontal_blur as i32,
            );
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(
                    gl::TEXTURE_2D,
                    if first_iteration {
                        self.color_textures[1]
                    } else {
                        self.blur_color_textures[!horizontal_blur as usize]
                    },
                );
            }
            // Draw to color texture as a quad
            self.screen_vao.bind();
            unsafe {
                gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            }
            self.screen_vao.unbind();
            // Update vars
            horizontal_blur = !horizontal_blur;
            if first_iteration {
                first_iteration = false;
            }
        }

        /* Pass 4 : Draw to quad */
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        clear_color(
            (BufferBit::ColorBufferBit as GLenum | BufferBit::DepthBufferBit as GLenum)
                as gl::types::GLbitfield,
        );

        // Prepare uniforms
        self.tone_mapping_shader.bind();
        unsafe {
            // Set base color texture
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.color_textures[0]);
            // Set gaussian blur texture
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(
                gl::TEXTURE_2D,
                self.blur_color_textures[!horizontal_blur as usize],
            );
            let location: i32 = gl::GetUniformLocation(
                self.tone_mapping_shader.id,
                CString::new("bloom_blur_buffer")?
                    .as_c_str()
                    .as_ptr()
                    .cast(),
            );
            gl::Uniform1i(location, 1);
        }
        let tm_lock: std::sync::MutexGuard<'_, bool> = ENABLE_BLOOM.lock().unwrap();
        let ex_lock: std::sync::MutexGuard<'_, f32> = EXPOSURE.lock().unwrap();
        self.tone_mapping_shader
            .set_uniform_1i(CString::new("enable_bloom")?.as_c_str(), *tm_lock as i32);
        self.tone_mapping_shader
            .set_uniform_1f(CString::new("exposure")?.as_c_str(), *ex_lock);

        // Draw final image
        self.screen_vao.bind();
        unsafe {
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
        self.screen_vao.unbind();

        // Swap buffers of window
        win.swap_buffers()?;

        Ok(())
    }

    pub fn render_scence(&self, shader: &ShaderProgram) -> anyhow::Result<()> {
        let model_name = CString::new("model")?;

        /* Draw cubes */
        // Cube1
        let mut object_model_matrix = na::Matrix4::identity();
        object_model_matrix = glm::translate(&object_model_matrix, &glm::vec3(0.0, -1.0, 0.0));
        object_model_matrix = glm::scale(&object_model_matrix, &glm::vec3(12.5, 0.5, 12.5));
        shader.set_uniform_mat4fv(model_name.as_c_str(), &object_model_matrix);
        self.cube_model.draw(shader, "material")?;
        // Cube2
        let mut object_model_matrix = na::Matrix4::identity();
        object_model_matrix = glm::translate(&object_model_matrix, &glm::vec3(0.0, 1.5, 0.0));
        object_model_matrix = glm::scale(&object_model_matrix, &glm::vec3(0.5, 0.5, 0.5));
        shader.set_uniform_mat4fv(model_name.as_c_str(), &object_model_matrix);
        self.cube_model.draw(shader, "material")?;
        // Cube3
        let mut object_model_matrix = na::Matrix4::identity();
        object_model_matrix = glm::translate(&object_model_matrix, &glm::vec3(2.0, 0.0, 1.0));
        object_model_matrix = glm::scale(&object_model_matrix, &glm::vec3(0.5, 0.5, 0.5));
        shader.set_uniform_mat4fv(model_name.as_c_str(), &object_model_matrix);
        self.cube_model.draw(shader, "material")?;
        // Cube4
        let mut object_model_matrix = na::Matrix4::identity();
        object_model_matrix = glm::translate(&object_model_matrix, &glm::vec3(-1.0, -1.0, 2.0));
        object_model_matrix = glm::rotate(
            &object_model_matrix,
            glm::radians(&glm::Vec1::new(60.0))[0],
            &glm::vec3(1.0, 0.0, 1.0),
        );
        shader.set_uniform_mat4fv(model_name.as_c_str(), &object_model_matrix);
        self.cube_model.draw(shader, "material")?;
        // Cube5
        let mut object_model_matrix = na::Matrix4::identity();
        object_model_matrix = glm::translate(&object_model_matrix, &glm::vec3(0.0, 2.7, 4.0));
        object_model_matrix = glm::rotate(
            &object_model_matrix,
            glm::radians(&glm::Vec1::new(23.0))[0],
            &glm::vec3(1.0, 0.0, 1.0),
        );
        object_model_matrix = glm::scale(&object_model_matrix, &glm::vec3(1.25, 1.25, 1.25));
        shader.set_uniform_mat4fv(model_name.as_c_str(), &object_model_matrix);
        self.cube_model.draw(shader, "material")?;
        // Cube6
        let mut object_model_matrix = na::Matrix4::identity();
        object_model_matrix = glm::translate(&object_model_matrix, &glm::vec3(-2.0, 1.0, -3.0));
        object_model_matrix = glm::rotate(
            &object_model_matrix,
            glm::radians(&glm::Vec1::new(124.0))[0],
            &glm::vec3(1.0, 0.0, 1.0),
        );
        shader.set_uniform_mat4fv(model_name.as_c_str(), &object_model_matrix);
        self.cube_model.draw(shader, "material")?;
        // Cube7
        let mut object_model_matrix = na::Matrix4::identity();
        object_model_matrix = glm::translate(&object_model_matrix, &glm::vec3(-3.0, 0.0, 0.0));
        object_model_matrix = glm::scale(&object_model_matrix, &glm::vec3(0.5, 0.5, 0.5));
        shader.set_uniform_mat4fv(model_name.as_c_str(), &object_model_matrix);
        self.cube_model.draw(shader, "material")?;

        Ok(())
    }

    pub fn close(self) {
        self.object_shader.close();
    }
}

#[allow(unreachable_code)]
fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    /* Camera */
    // Init camera
    let camera_pos = na::Point3::new(CAMERA_POS[0], CAMERA_POS[1], CAMERA_POS[2]);
    let camera_look_at = na::Vector3::new(CAMERA_LOOK_AT[0], CAMERA_LOOK_AT[1], CAMERA_LOOK_AT[2]);
    let camera_up = na::Vector3::new(CAMERA_UP[0], CAMERA_UP[1], CAMERA_UP[2]);
    let mut camera = learn::Camera::new(camera_pos, camera_look_at, camera_up);

    /* Window */
    let (win, event_loop) = match WinitWindow::new(WINDOW_TITLE, SCREEN_WIDTH, SCREEN_HEIGHT) {
        Ok((win, event_loop)) => (win, event_loop),
        Err(e) => {
            error!("Failed to create window: {}", e);
            bail!(e);
        }
    };

    /* Renderer */
    let renderer = match Renderer::new(&win) {
        Ok(renderer) => renderer,
        Err(e) => {
            bail!("Failed to create renderer: {}", e);
        }
    };

    /* Main Loop */
    let mut last_time = std::time::SystemTime::now();
    event_loop.run(move |event, _window_target, control_flow| {
        // Set ControlFlow::Poll: when the current loop iteration finishes, immediately begin a new iteration regardless
        // of whether or not new events are available to process. This is ideal for games and similar applications.
        control_flow.set_poll();

        if win.handle_event_default(&event, control_flow) {
            return;
        }

        match event {
            // Event "RedrawRequested" : Emitted after MainEventsCleared when a window should be redrawn.
            Event::RedrawRequested(_window_id) => {
                let current_time = std::time::SystemTime::now();
                let delta_time = current_time
                    .duration_since(last_time)
                    .unwrap()
                    .as_secs_f32();
                last_time = current_time;

                /* Do REDRAW */
                if let Err(e) = renderer.redraw(&win, &camera, delta_time) {
                    error!("Failed to redraw: {}", e);
                    control_flow.set_exit();
                }
            }
            Event::WindowEvent { event, .. } => {
                if !camera.handle_winit_event(&event) {
                    match event {
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Space),
                                    ..
                                },
                            is_synthetic: false,
                            ..
                        } => {
                            let mut lock: std::sync::MutexGuard<'_, bool> =
                                ENABLE_BLOOM.lock().unwrap();
                            *lock = !(*lock);
                        }
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Up),
                                    ..
                                },
                            is_synthetic: false,
                            ..
                        } => {
                            let mut lock: std::sync::MutexGuard<'_, f32> = EXPOSURE.lock().unwrap();
                            *lock += 0.5;
                        }
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Down),
                                    ..
                                },
                            is_synthetic: false,
                            ..
                        } => {
                            let mut lock: std::sync::MutexGuard<'_, f32> = EXPOSURE.lock().unwrap();
                            if *lock >= 0.5 {
                                *lock -= 0.5;
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    });

    renderer.close();
    Ok(())
}
