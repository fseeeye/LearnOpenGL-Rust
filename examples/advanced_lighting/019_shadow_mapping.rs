//! This example has more infos about shadow mapping.

// remove console window : https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ffi::CString, path::PathBuf};

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
use winit::event::Event;

/* Screen info */
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const BACKGROUND_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 0.1];

/* Camera data */
const CAMERA_POS: [f32; 3] = [0.0, 0.5, 2.0];

/* Shaodw Map data */
const SHADOW_MAP_WIDTH: i32 = 1024;
const SHADOW_MAP_HEIGHT: i32 = 1024;
const SHADOW_MAP_NEAR: f32 = 1.0;
const SHADOW_MAP_FAR: f32 = 7.5;
const LIGHT_POS: [f32; 3] = [-2.0, 4.0, -1.0];

/* Screen data */
const SCREEN_VERTICES: [[f32; 4]; 6] = [
    // vertex attributes for a quad that fills the entire screen in Normalized Device Coordinates.
    [-1.0, 1.0, 0.0, 1.0],
    [-1.0, -1.0, 0.0, 0.0],
    [1.0, -1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0, 1.0],
    [1.0, -1.0, 1.0, 0.0],
    [1.0, 1.0, 1.0, 1.0],
];

struct Renderer {
    cube_model: Model,
    plane_model: Model,
    object_shader: ShaderProgram,
    shadow_map_texture: u32,
    shadow_map_fbo: u32,
    shadow_map_shader: ShaderProgram,
    screen_vao: VertexArray,
    screen_shader: ShaderProgram,
}

impl Renderer {
    pub fn new() -> anyhow::Result<Self> {
        /* Extra Settings */

        // Set clear color
        set_clear_color(
            BACKGROUND_COLOR[0],
            BACKGROUND_COLOR[1],
            BACKGROUND_COLOR[2],
            BACKGROUND_COLOR[3],
        );
        // Configure global opengl state
        unsafe {
            // Enable Depth Test
            gl::Enable(gl::DEPTH_TEST);
            // Set depth function
            gl::DepthFunc(gl::LESS);
            // Enable MSAA
            gl::Enable(gl::MULTISAMPLE);
        };

        /* Object Vertices & Shader */

        // Prepare model of object
        let cube_model = Model::new(PathBuf::from("assets/models/cube/cube.obj"))?;
        let plane_model = Model::new(PathBuf::from("assets/models/plane_wood/plane.obj"))?;

        // Prepare shader of object
        let object_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/advanced_lighting/019-object.vert"),
            include_str!("../../assets/shaders/advanced_lighting/019-object.frag"),
        )?;
        let shadow_map_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/advanced_lighting/019-shadow-map.vert"),
            include_str!("../../assets/shaders/advanced_lighting/019-shadow-map.frag"),
        )?;

        /* Shadow Map Framebuffer */

        // Generate texture for framebuffer
        let mut shadow_map_texture = 0;
        unsafe {
            gl::GenTextures(1, &mut shadow_map_texture);
            gl::BindTexture(gl::TEXTURE_2D, shadow_map_texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT as GLint,
                SHADOW_MAP_WIDTH,
                SHADOW_MAP_HEIGHT,
                0,
                gl::RGB,
                gl::DEPTH_COMPONENT,
                core::ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        // Create framebuffer object
        let mut shadow_map_fbo = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut shadow_map_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, shadow_map_fbo);
            // Bind texture to framebuffer
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                shadow_map_texture,
                0,
            );
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        /* Screen */

        let screen_vao = VertexArray::new()?;

        let screen_vbo = Buffer::new(BufferType::VertexBuffer)?;
        screen_vbo.bind();
        screen_vbo.set_buffer_data(SCREEN_VERTICES.as_slice(), BufferUsage::StaticDraw);

        screen_vao.bind();
        let mut screen_vertex_desc = VertexDescription::new();
        screen_vertex_desc.add_attribute(gl::FLOAT, 2); // set coords attribute
        screen_vertex_desc.add_attribute(gl::FLOAT, 2); // set Texture coord attribute
        screen_vertex_desc.bind_to(&screen_vbo, Some(&screen_vao));

        // Prepare shader of screen
        let screen_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/advanced_opengl/017-screen.vert"),
            include_str!("../../assets/shaders/advanced_opengl/017-screen.frag"),
        )?;
        screen_shader.set_uniform_1i(CString::new("screen_texture")?.as_c_str(), 0);

        Ok(Self {
            cube_model,
            plane_model,
            object_shader,
            shadow_map_texture,
            shadow_map_fbo,
            shadow_map_shader,
            screen_vao,
            screen_shader,
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
            std::f32::consts::FRAC_PI_4,
            0.1,
            100.0,
        )
        .to_homogeneous(); // Perspective projection
        let projection_name = CString::new("projection")?;

        /* Pass1 : Draw Shadow Map */

        // prepare shader of shadow map
        self.shadow_map_shader.bind();
        let projection_matrix_light =
            glm::ortho(-10.0, 10.0, -10.0, 10.0, SHADOW_MAP_NEAR, SHADOW_MAP_FAR);
        let view_matrix_light = glm::look_at(
            &glm::vec3(LIGHT_POS[0], LIGHT_POS[1], LIGHT_POS[2]),
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 1.0, 0.0),
        );
        let light_space_matrix = projection_matrix_light * view_matrix_light;
        self.shadow_map_shader.set_uniform_mat4fv(
            CString::new("light_space_matrix")?.as_c_str(),
            &light_space_matrix,
        );

        unsafe {
            gl::Viewport(0, 0, SHADOW_MAP_WIDTH, SHADOW_MAP_HEIGHT);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.shadow_map_fbo);
        }
        clear_color(BufferBit::DepthBufferBit as GLenum);

        // do drawing shadow map
        self.render_scence(&self.object_shader)?;

        /* Draw Screen */

        unsafe {
            gl::Viewport(0, 0, window_width as i32, window_height as i32);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            // Disable depth test
            gl::Disable(gl::DEPTH_TEST);
        }

        clear_color((BufferBit::ColorBufferBit as GLenum) as gl::types::GLbitfield);

        // Draw screen
        self.screen_vao.bind();
        self.screen_shader.bind();
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.shadow_map_texture);

            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        // /* Draw object */
        // unsafe {
        //     gl::Viewport(0, 0, window_width as i32, window_height as i32);
        //     gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        // }
        // clear_color(
        //     (BufferBit::ColorBufferBit as GLenum | BufferBit::DepthBufferBit as GLenum)
        //         as gl::types::GLbitfield,
        // );

        // self.object_shader.bind();

        // self.object_shader
        //     .set_uniform_mat4fv(view_name.as_c_str(), &object_view_matrix);
        // self.object_shader
        //     .set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);
        // self.object_shader.set_uniform_3f(
        //     CString::new("camera_pos")?.as_c_str(),
        //     camera.get_pos().x,
        //     camera.get_pos().y,
        //     camera.get_pos().z,
        // );

        // self.render_scence(&self.object_shader)?;

        // Swap buffers of window
        win.swap_buffers()?;

        Ok(())
    }

    pub fn render_scence(&self, shader: &ShaderProgram) -> anyhow::Result<()> {
        let model_name = CString::new("model")?;

        // Plane
        let mut object_model_matrix = na::Matrix4::identity();
        shader.set_uniform_mat4fv(model_name.as_c_str(), &object_model_matrix);
        self.plane_model.draw(shader, "material")?;
        // Cube 1
        object_model_matrix = na::Matrix4::identity();
        object_model_matrix = glm::translate(&object_model_matrix, &glm::vec3(0.0, 1.5, 0.0));
        shader.set_uniform_mat4fv(model_name.as_c_str(), &object_model_matrix);
        self.cube_model.draw(shader, "material")?;
        // Cube 2
        object_model_matrix = na::Matrix4::identity();
        object_model_matrix = glm::translate(&object_model_matrix, &glm::vec3(2.0, 0.0, 1.0));
        shader.set_uniform_mat4fv(model_name.as_c_str(), &object_model_matrix);
        self.cube_model.draw(shader, "material")?;
        // Cube 3
        object_model_matrix = na::Matrix4::identity();
        object_model_matrix = glm::translate(&object_model_matrix, &glm::vec3(-1.0, 0.0, 2.0));
        object_model_matrix = glm::rotate(
            &object_model_matrix,
            glm::radians(&glm::TVec::<f32, 1>::new(60.0))[0],
            &glm::normalize(&glm::vec3(1.0, 0.0, 1.0)),
        );
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
    // Init camera at pos(0,0,3) look-at(0,0,0) up(0,1,0)
    let camera_pos = na::Point3::new(CAMERA_POS[0], CAMERA_POS[1], CAMERA_POS[2]);
    let camera_look_at = na::Vector3::new(0.0, 0.0, -1.0);
    let camera_up = na::Vector3::new(0.0, 1.0, 0.0);
    let mut camera = learn::Camera::new(camera_pos, camera_look_at, camera_up);

    /* Window */
    let (win, event_loop) = match WinitWindow::new("Simple Triangle", SCREEN_WIDTH, SCREEN_HEIGHT) {
        Ok((win, event_loop)) => (win, event_loop),
        Err(e) => {
            error!("Failed to create window: {}", e);
            bail!(e);
        }
    };

    /* Renderer */
    let renderer = match Renderer::new() {
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
            Event::WindowEvent { event, .. } => if !camera.handle_winit_event(&event) {},
            _ => (),
        }
    });

    renderer.close();
    Ok(())
}