//! This example has more infos about depth test.

// remove console window : https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ffi::CString, path::PathBuf};

use anyhow::bail;
use gl::types::*;

use image::GenericImageView;
use learn::{clear_color, set_clear_color, BufferBit, Camera, Model, ShaderProgram, WinitWindow, VertexArray, Buffer, BufferType, BufferUsage, VertexDescription};
use learn_opengl_rs as learn;

use nalgebra as na;
use tracing::error;
use winit::event::Event;

/* Screen info */
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const BACKGROUND_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/* Camera data */
const CAMERA_POS: [f32; 3] = [0.0, 0.7, 2.0];

/* Skybox data */
const SKYBOX_VERTICES: [[f32; 3]; 36] = [
    [-1.0,  1.0, -1.0],
    [-1.0, -1.0, -1.0],
    [ 1.0, -1.0, -1.0],
    [ 1.0, -1.0, -1.0],
    [ 1.0,  1.0, -1.0],
    [-1.0,  1.0, -1.0],

    [-1.0, -1.0,  1.0],
    [-1.0, -1.0, -1.0],
    [-1.0,  1.0, -1.0],
    [-1.0,  1.0, -1.0],
    [-1.0,  1.0,  1.0],
    [-1.0, -1.0,  1.0],

    [ 1.0, -1.0, -1.0],
    [ 1.0, -1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0,  1.0, -1.0],
    [ 1.0, -1.0, -1.0],

    [-1.0, -1.0,  1.0],
    [-1.0,  1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0, -1.0,  1.0],
    [-1.0, -1.0,  1.0],

    [-1.0,  1.0, -1.0],
    [ 1.0,  1.0, -1.0],
    [ 1.0,  1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [-1.0,  1.0,  1.0],
    [-1.0,  1.0, -1.0],

    [-1.0, -1.0, -1.0],
    [-1.0, -1.0,  1.0],
    [ 1.0, -1.0, -1.0],
    [ 1.0, -1.0, -1.0],
    [-1.0, -1.0,  1.0],
    [ 1.0, -1.0,  1.0]
];

struct Renderer {
    cube_model: Model,
    object_shader: ShaderProgram,
    skybox_vao: VertexArray,
    skybox_shader: ShaderProgram,
    skybox_cubemap: GLuint
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
        unsafe {
            // Enable Depth Test
            gl::Enable(gl::DEPTH_TEST);
            // Set depth function
            gl::DepthFunc(gl::LESS);
        };

        /* Object Vertices & Shader */

        // Prepare model of object
        let cube_model = Model::new(PathBuf::from("assets/models/cube/cube.obj"))?;

        // Prepare shader of object
        let object_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/advanced_opengl/018-object.vert"),
            include_str!("../../assets/shaders/advanced_opengl/018-object.frag"),
        )?;

        /* Skybox Vertices & Shader & Texture */

        // Prepare VAO of skybox
        let skybox_vao = VertexArray::new()?;

        let skybox_vbo = Buffer::new(BufferType::VertexBuffer)?;
        skybox_vbo.bind();
        skybox_vbo.set_buffer_data(SKYBOX_VERTICES.as_slice(), BufferUsage::StaticDraw);

        skybox_vao.bind();
        let mut skybox_vertex_desc = VertexDescription::new();
        skybox_vertex_desc.add_attribute(gl::FLOAT, 3); // set coords attribute
        skybox_vertex_desc.bind_to(&skybox_vbo, Some(&skybox_vao));

        // Prepare shader of skybox
        let skybox_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/advanced_opengl/018-skybox.vert"),
            include_str!("../../assets/shaders/advanced_opengl/018-skybox.frag"),
        )?;
        skybox_shader.set_uniform_1i(CString::new("skybox")?.as_c_str(), 0);

        // Load cubemap of skybox
        let skybox_cubemap = Self::load_skybox_texture()?;

        Ok(Self {
            cube_model,
            object_shader,
            skybox_vao,
            skybox_shader,
            skybox_cubemap
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

        // Model Matrix
        let model_name = CString::new("model")?;
        let object_model_matrix = na::Matrix4::identity();

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

        /* Draw skybox */

        self.skybox_shader.bind();
        self.skybox_vao.bind();

        let mut skybox_view_matrix = object_view_matrix
            .fixed_view::<3, 3>(0, 0)
            .fixed_resize::<4, 4>(0.0);
        skybox_view_matrix[(3, 3)] = 1.0;
        self.skybox_shader
            .set_uniform_mat4fv(view_name.as_c_str(), &skybox_view_matrix);
        self.skybox_shader
            .set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);

        unsafe {
            gl::DepthMask(gl::FALSE);

            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.skybox_cubemap);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            gl::DepthMask(gl::TRUE);
        }

        /* Draw object */

        self.object_shader.bind();

        self.object_shader
            .set_uniform_mat4fv(model_name.as_c_str(), &object_model_matrix);
        self.object_shader
            .set_uniform_mat4fv(view_name.as_c_str(), &object_view_matrix);
        self.object_shader
            .set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);

        self.cube_model.draw(&self.object_shader, "material")?;

        // Swap buffers of window
        win.swap_buffers()?;

        Ok(())
    }

    pub fn close(self) {
        self.object_shader.close();
    }

    pub fn load_skybox_texture() -> anyhow::Result<GLuint> {
        let mut cubemap: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut cubemap);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, cubemap);
        }

        // Load cubemap textures
        let faces = [
            "assets/textures/skybox/right.jpg",
            "assets/textures/skybox/left.jpg",
            "assets/textures/skybox/top.jpg",
            "assets/textures/skybox/bottom.jpg",
            "assets/textures/skybox/front.jpg",
            "assets/textures/skybox/back.jpg",
        ];
        
        for (i, face) in faces.iter().enumerate() {
            let img = image::open(face).unwrap();

            let (width, height) = img.dimensions();
            let img_format: GLenum;
            let img_type: GLenum;
            match img.color() {
                image::ColorType::Rgb8 => {
                    img_format = gl::RGB;
                    img_type = gl::UNSIGNED_BYTE;
                }
                image::ColorType::Rgba8 => {
                    img_format = gl::RGBA;
                    img_type = gl::UNSIGNED_BYTE;
                }
                _ => {
                    bail!("Unsupported image color type: {:?}", img.color())
                }
            }

            let pixels = img.into_bytes();
            unsafe {
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as GLenum,
                    0,
                    img_format as GLint,
                    width.try_into()?,
                    height.try_into()?,
                    0,
                    img_format,
                    img_type,
                    pixels.as_ptr().cast(),
                );
            }

            // Set Texture wrapping & filtering
            unsafe {
                gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
                gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
                gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as GLint);
            }
        }

        Ok(cubemap)
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
