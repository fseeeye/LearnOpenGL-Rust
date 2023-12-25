//! This example has more infos about deferred rendering.

// remove console window : https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ffi::CString, path::PathBuf};

use anyhow::bail;
use gl::types::*;
use rand::Rng;

use learn::{
    clear_color, set_clear_color, Buffer, BufferBit, BufferType, BufferUsage, Camera, Model,
    ShaderProgram, VertexArray, VertexDescription, WinitWindow,
};
use learn_opengl_rs as learn;

use nalgebra as na;
use nalgebra_glm as glm;
use tracing::error;
use winit::event::Event;

/* Screen info */
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const BACKGROUND_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WINDOW_TITLE: &str = "Deferred Rendering";
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
const LIGHT_NUM: usize = 32;

/* Object data */
const BACKPACK_POS: [[f32; 3]; 9] = [
    [-3.0, -0.5, -3.0],
    [0.0, -0.5, -3.0],
    [3.0, -0.5, -3.0],
    [-3.0, -0.5, 0.0],
    [0.0, -0.5, 0.0],
    [3.0, -0.5, 0.0],
    [-3.0, -0.5, 3.0],
    [0.0, -0.5, 3.0],
    [3.0, -0.5, 3.0],
];

struct Renderer {
    backpack_model: Model,
    g_buffer: u32,
    g_position: u32,
    g_normal: u32,
    g_albedo_specular: u32,
    gbuffer_shader: ShaderProgram,

    screen_vao: VertexArray,
    lighting_pass_shader: ShaderProgram,
    light_postions: Vec<glm::Vec3>,
    light_colors: Vec<glm::Vec3>,
    cube_model: Model,
    light_box_shader: ShaderProgram,
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

        /* Lights */

        let mut light_postions: Vec<glm::Vec3> = Vec::new();
        let mut light_colors: Vec<glm::Vec3> = Vec::new();
        for _ in 0..LIGHT_NUM {
            let mut rng = rand::thread_rng();
            let x = rng.gen_range(-3.0..=3.0);
            let y = rng.gen_range(-3.0..=3.0);
            let z = rng.gen_range(-3.0..=3.0);
            let r = rng.gen_range(0.5..=1.0);
            let g = rng.gen_range(0.5..=1.0);
            let b = rng.gen_range(0.5..=1.0);
            light_postions.push(glm::Vec3::new(x, y, z));
            light_colors.push(glm::Vec3::new(r, g, b));
        }

        /* Object Models */

        // backpack object
        let backpack_model = Model::new(PathBuf::from("assets/models/backpack/backpack.obj"))?;
        // // cube object
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

        // Create shader of G-Buffer (Geometry Pass)
        let gbuffer_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/advanced_lighting/024-gbuffer.vert"),
            include_str!("../../assets/shaders/advanced_lighting/024-gbuffer.frag"),
        )?;

        // Create shader of Lighting Pass
        let lighting_pass_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/advanced_lighting/024-lighting-pass.vert"),
            include_str!("../../assets/shaders/advanced_lighting/024-lighting-pass.frag"),
        )?;
        for i in 0..LIGHT_NUM {
            let light_name = format!("lights[{}].position", i);
            lighting_pass_shader.set_uniform_3f(
                CString::new(light_name)?.as_c_str(),
                light_postions[i][0],
                light_postions[i][1],
                light_postions[i][2],
            );
            let light_name = format!("lights[{}].color", i);
            lighting_pass_shader.set_uniform_3f(
                CString::new(light_name)?.as_c_str(),
                light_colors[i][0],
                light_colors[i][1],
                light_colors[i][2],
            );
        }

        // Create shader of light box
        let light_box_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/advanced_lighting/024-light-box.vert"),
            include_str!("../../assets/shaders/advanced_lighting/024-light-box.frag"),
        )?;

        /* GBuffer */

        // Create framebuffer
        let mut g_buffer = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut g_buffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, g_buffer);
        }
        // Create color texture to restore postion in GBuffer
        let mut g_position = 0;
        unsafe {
            gl::GenTextures(1, &mut g_position);
            gl::BindTexture(gl::TEXTURE_2D, g_position);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA16F as GLint, // use RGBA16F to restore position
                win.get_window_size().0.try_into()?,
                win.get_window_size().1.try_into()?,
                0,
                gl::RGB,
                gl::FLOAT,
                core::ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            // Attach color texture to framebuffer
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                g_position,
                0,
            );
        }
        // Create color texture to restore normal in GBuffer
        let mut g_normal = 0;
        unsafe {
            gl::GenTextures(1, &mut g_normal);
            gl::BindTexture(gl::TEXTURE_2D, g_normal);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA16F as GLint, // use RGBA16F to restore normal vector
                win.get_window_size().0.try_into()?,
                win.get_window_size().1.try_into()?,
                0,
                gl::RGB,
                gl::FLOAT,
                core::ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            // Attach color texture to framebuffer
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT1,
                gl::TEXTURE_2D,
                g_normal,
                0,
            );
        }
        // Create color texture to restore albedo & specular in GBuffer
        let mut g_albedo_specular = 0;
        unsafe {
            gl::GenTextures(1, &mut g_albedo_specular);
            gl::BindTexture(gl::TEXTURE_2D, g_albedo_specular);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint, // use RGBA8 to restore albedo and specular
                win.get_window_size().0.try_into()?,
                win.get_window_size().1.try_into()?,
                0,
                gl::RGBA,
                gl::FLOAT,
                core::ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            // Attach color texture to framebuffer
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT2,
                gl::TEXTURE_2D,
                g_albedo_specular,
                0,
            );
        }
        // Tell OpenGL which color attachments we'll use (of this framebuffer) for rendering
        let attachments: [u32; 3] = [
            gl::COLOR_ATTACHMENT0,
            gl::COLOR_ATTACHMENT1,
            gl::COLOR_ATTACHMENT2,
        ];
        unsafe {
            gl::DrawBuffers(3, attachments.as_ptr());
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
        // Check framebuffer status
        let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };
        if status != gl::FRAMEBUFFER_COMPLETE {
            bail!("Framebuffer is not complete!");
        }
        // Bind framebuffer to default
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Ok(Self {
            backpack_model,
            g_buffer,
            g_position,
            g_normal,
            g_albedo_specular,
            gbuffer_shader,
            screen_vao,
            lighting_pass_shader,
            light_postions,
            light_colors,
            cube_model,
            light_box_shader,
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

        /* Pass 1 : G-Buffer */
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.g_buffer);
        }

        clear_color(
            (BufferBit::ColorBufferBit as GLenum | BufferBit::DepthBufferBit as GLenum)
                as gl::types::GLbitfield,
        );

        self.gbuffer_shader.bind();
        self.gbuffer_shader
            .set_uniform_mat4fv(view_name.as_c_str(), &object_view_matrix);
        self.gbuffer_shader
            .set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);

        self.render_scence(&self.gbuffer_shader)?;

        /* Pass 2 : Lighting Pass */

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        clear_color(
            (BufferBit::ColorBufferBit as GLenum | BufferBit::DepthBufferBit as GLenum)
                as gl::types::GLbitfield,
        );

        self.lighting_pass_shader.bind();
        self.lighting_pass_shader.set_uniform_3f(
            CString::new("camera_pos")?.as_c_str(),
            camera.get_pos().x,
            camera.get_pos().y,
            camera.get_pos().z,
        );
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.g_position);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.g_normal);
            let location: i32 = gl::GetUniformLocation(
                self.lighting_pass_shader.id,
                CString::new("g_normal")?.as_c_str().as_ptr().cast(),
            );
            gl::Uniform1i(location, 1);
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, self.g_albedo_specular);
            let location: i32 = gl::GetUniformLocation(
                self.lighting_pass_shader.id,
                CString::new("g_albedo_spec")?.as_c_str().as_ptr().cast(),
            );
            gl::Uniform1i(location, 2);
        }

        self.screen_vao.bind();
        unsafe {
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
        self.screen_vao.unbind();

        /* Pass 3 : Draw light box with Forward Rendering */

        // Copy content of geometry's depth buffer to default framebuffer's depth buffer
        unsafe {
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.g_buffer);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0); // write to default framebuffer
            gl::BlitFramebuffer(
                0,
                0,
                window_width as i32,
                window_height as i32,
                0,
                0,
                window_width as i32,
                window_height as i32,
                gl::DEPTH_BUFFER_BIT,
                gl::NEAREST,
            ); // Blit to default framebuffer.
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        self.light_box_shader.bind();
        self.light_box_shader
            .set_uniform_mat4fv(view_name.as_c_str(), &object_view_matrix);
        self.light_box_shader
            .set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);

        for i in 0..LIGHT_NUM {
            let mut light_model_matrix =
                glm::translate(&na::Matrix4::identity(), &self.light_postions[i]);
            light_model_matrix = glm::scale(&light_model_matrix, &glm::vec3(0.1, 0.1, 0.1));
            self.light_box_shader
                .set_uniform_mat4fv(CString::new("model")?.as_c_str(), &light_model_matrix);

            self.light_box_shader.set_uniform_3f(
                CString::new("light_color")?.as_c_str(),
                self.light_colors[i].x,
                self.light_colors[i].y,
                self.light_colors[i].z,
            );

            self.cube_model.draw(&self.light_box_shader, "")?;
        }

        // Swap buffers of window
        win.swap_buffers()?;

        Ok(())
    }

    pub fn render_scence(&self, shader: &ShaderProgram) -> anyhow::Result<()> {
        /* Draw backpacks */
        let model_name = CString::new("model")?;
        for i in 0..BACKPACK_POS.len() {
            let mut object_model_matrix = na::Matrix4::identity();
            object_model_matrix = glm::translate(
                &object_model_matrix,
                &glm::Vec3::new(BACKPACK_POS[i][0], BACKPACK_POS[i][1], BACKPACK_POS[i][2]),
            );
            object_model_matrix = glm::scale(&object_model_matrix, &glm::vec3(0.5, 0.5, 0.5));
            shader.set_uniform_mat4fv(model_name.as_c_str(), &object_model_matrix);
            self.backpack_model.draw(shader, "material")?;
        }

        Ok(())
    }

    pub fn close(self) {
        self.gbuffer_shader.close();
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
