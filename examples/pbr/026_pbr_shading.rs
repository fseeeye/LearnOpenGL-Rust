//! This example is about PBR Lighting.

// remove console window : https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ffi::CString, path::PathBuf};

use anyhow::bail;
use gl::types::*;

use learn::{
    clear_color, set_clear_color, Buffer, BufferBit, BufferType, BufferUsage, Camera,
    ShaderProgram, Texture, TextureType, VertexArray, VertexDescription, WinitWindow,
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
const WINDOW_TITLE: &str = "PBR";

/* Camera data */
const CAMERA_POS: [f32; 3] = [0.0, 0.0, 22.0];
const CAMERA_LOOK_AT: [f32; 3] = [0.0, 0.0, -1.0];
const CAMERA_UP: [f32; 3] = [0.0, 1.0, 0.0];
const PROJECTION_FOV: f32 = std::f32::consts::FRAC_PI_4;
const PROJECTION_NEAR: f32 = 0.1;
const PROJECTION_FAR: f32 = 100.0;

/* Object data */
const SPHERE_ROWS: i32 = 7;
const SPHERE_COLUMNS: i32 = 7;
const SPHERE_SPACING: f32 = 2.5;

/* Scene data */
const POINT_LIGHT_POS: [glm::Vec3; 4] = [
    glm::Vec3::new(-10.0, 10.0, 10.0),
    glm::Vec3::new(10.0, 10.0, 10.0),
    glm::Vec3::new(-10.0, -10.0, 10.0),
    glm::Vec3::new(10.0, -10.0, 10.0),
];
const POINT_LIGHT_COLOR: [glm::Vec3; 4] = [
    glm::Vec3::new(300.0, 300.0, 300.0),
    glm::Vec3::new(300.0, 300.0, 300.0),
    glm::Vec3::new(300.0, 300.0, 300.0),
    glm::Vec3::new(300.0, 300.0, 300.0),
];

struct Renderer {
    pbr_shader: ShaderProgram,

    sphere_vao: VertexArray,
    sphere_index_len: usize,

    albedo_map: Texture,
    normal_map: Texture,
    metallic_map: Texture,
    roughness_map: Texture,
    ao_map: Texture,
}

impl Renderer {
    pub fn new() -> anyhow::Result<Self> {
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

        /* Textures */
        let albedo_map = Texture::create(
            PathBuf::from("assets/textures/pbr/rusted_iron/albedo.png"),
            Some(TextureType::PbrAlbedo),
        )?;
        let normal_map = Texture::create(
            PathBuf::from("assets/textures/pbr/rusted_iron/normal.png"),
            Some(TextureType::Normal),
        )?;
        let metallic_map = Texture::create(
            PathBuf::from("assets/textures/pbr/rusted_iron/metallic.png"),
            Some(TextureType::PbrMetallic),
        )?;
        let roughness_map = Texture::create(
            PathBuf::from("assets/textures/pbr/rusted_iron/roughness.png"),
            Some(TextureType::PbrRoughness),
        )?;
        let ao_map = Texture::create(
            PathBuf::from("assets/textures/pbr/rusted_iron/ao.png"),
            Some(TextureType::PbrAO),
        )?;

        /* Shaders */

        // Create shader of PBR
        let pbr_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/pbr/026-pbr.vert"),
            include_str!("../../assets/shaders/pbr/026-pbr.frag"),
        )?;

        /* Object Models */

        let sphere_vao = VertexArray::new()?;
        let sphere_vbo = Buffer::new(BufferType::VertexBuffer)?;
        let sphere_ibo = Buffer::new(BufferType::IndexBuffer)?;

        // Prepare vao data
        let mut positions: Vec<glm::Vec3> = Vec::new();
        let mut uvs: Vec<glm::Vec2> = Vec::new();
        let mut normals: Vec<glm::Vec3> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        const X_SEGMENTS: u32 = 64;
        const Y_SEGMENTS: u32 = 64;
        for x in 0..=X_SEGMENTS {
            for y in 0..=Y_SEGMENTS {
                let x_segment = x as f32 / X_SEGMENTS as f32;
                let y_segment = y as f32 / Y_SEGMENTS as f32;

                let x_pos = (x_segment * 2.0 * std::f32::consts::PI).cos()
                    * (y_segment * std::f32::consts::PI).sin();
                let y_pos = (y_segment * std::f32::consts::PI).cos();
                let z_pos = (x_segment * 2.0 * std::f32::consts::PI).sin()
                    * (y_segment * std::f32::consts::PI).sin();

                positions.push(glm::vec3(x_pos, y_pos, z_pos));
                uvs.push(glm::vec2(x_segment, y_segment));
                normals.push(glm::vec3(x_pos, y_pos, z_pos));
            }
        }
        let mut odd_row = false;
        for y in 0..Y_SEGMENTS {
            if !odd_row {
                for x in 0..=X_SEGMENTS {
                    indices.push(y * (X_SEGMENTS + 1) + x);
                    indices.push((y + 1) * (X_SEGMENTS + 1) + x);
                }
            } else {
                for x in (0..=X_SEGMENTS).rev() {
                    indices.push((y + 1) * (X_SEGMENTS + 1) + x);
                    indices.push(y * (X_SEGMENTS + 1) + x);
                }
            }
            odd_row = !odd_row;
        }
        let sphere_index_len = indices.len();

        let mut vertex_data: Vec<f32> = Vec::new();
        for i in 0..positions.len() {
            vertex_data.push(positions[i].x);
            vertex_data.push(positions[i].y);
            vertex_data.push(positions[i].z);
            if normals.len() > i {
                vertex_data.push(normals[i].x);
                vertex_data.push(normals[i].y);
                vertex_data.push(normals[i].z);
            }
            if uvs.len() > i {
                vertex_data.push(uvs[i].x);
                vertex_data.push(uvs[i].y);
            }
        }

        // Set vao data
        sphere_vao.bind();
        sphere_vbo.set_buffer_data(vertex_data.as_slice(), BufferUsage::StaticDraw);
        let mut vertex_desc = VertexDescription::new();
        vertex_desc.add_attribute(gl::FLOAT, 3); // push NDC coords
        vertex_desc.add_attribute(gl::FLOAT, 3); // push normal
        vertex_desc.add_attribute(gl::FLOAT, 2); // push texture coords
        vertex_desc.bind_to(&sphere_vbo, Some(&sphere_vao));
        sphere_ibo.set_buffer_data(indices.as_slice(), BufferUsage::StaticDraw);

        Ok(Self {
            pbr_shader,
            sphere_vao,
            sphere_index_len,
            albedo_map,
            normal_map,
            metallic_map,
            roughness_map,
            ao_map,
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

        /* Pass 1 : PBR */

        self.pbr_shader.bind();
        self.pbr_shader
            .set_uniform_mat4fv(view_name.as_c_str(), &object_view_matrix);
        self.pbr_shader
            .set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);
        self.pbr_shader.set_uniform_3f(
            CString::new("camera_pos")?.as_c_str(),
            camera.get_pos().x,
            camera.get_pos().y,
            camera.get_pos().z,
        );

        // Bind textures
        self.pbr_shader.set_texture_unit(
            &CString::new("albedo_map")?,
            &self.albedo_map,
            learn::TextureUnit::TEXTURE0,
        );
        self.pbr_shader.set_texture_unit(
            &CString::new("normal_map")?,
            &self.normal_map,
            learn::TextureUnit::TEXTURE1,
        );
        self.pbr_shader.set_texture_unit(
            &CString::new("metallic_map")?,
            &self.metallic_map,
            learn::TextureUnit::TEXTURE2,
        );
        self.pbr_shader.set_texture_unit(
            &CString::new("roughness_map")?,
            &self.roughness_map,
            learn::TextureUnit::TEXTURE3,
        );
        self.pbr_shader.set_texture_unit(
            &CString::new("ao_map")?,
            &self.ao_map,
            learn::TextureUnit::TEXTURE4,
        );

        // Setup Lights
        for i in 0..POINT_LIGHT_POS.len() {
            self.pbr_shader.set_uniform_3f(
                &CString::new(format!("light_positions[{}]", i))?,
                POINT_LIGHT_POS[i].x,
                POINT_LIGHT_POS[i].y,
                POINT_LIGHT_POS[i].z,
            );
            self.pbr_shader.set_uniform_3f(
                &CString::new(format!("light_colors[{}]", i))?,
                POINT_LIGHT_COLOR[i].x,
                POINT_LIGHT_COLOR[i].y,
                POINT_LIGHT_COLOR[i].z,
            );
        }

        // Render spheres with varying metallic/roughness values scaled by rows and columns respectively
        let model_name = CString::new("model")?;
        self.pbr_shader
            .set_uniform_3f(CString::new("albedo")?.as_c_str(), 0.5, 0.0, 0.0);
        self.pbr_shader
            .set_uniform_1f(CString::new("ao")?.as_c_str(), 1.0);
        for row in 0..SPHERE_ROWS {
            self.pbr_shader.set_uniform_1f(
                CString::new("metallic")?.as_c_str(),
                row as f32 / (SPHERE_ROWS as f32),
            );
            for column in 0..SPHERE_COLUMNS {
                // we clamp the roughness to 0.05 - 1.0 as perfectly smooth surfaces (roughness of 0.0) tend to look
                // a bit off on direct lighting.
                self.pbr_shader.set_uniform_1f(
                    CString::new("roughness")?.as_c_str(),
                    f32::clamp(column as f32 / (SPHERE_COLUMNS as f32), 0.05, 1.0),
                );

                let mut model_matrix = glm::Mat4::identity();
                model_matrix = glm::translate(
                    &model_matrix,
                    &glm::vec3(
                        (column - (SPHERE_COLUMNS / 2)) as f32 * SPHERE_SPACING,
                        (row - (SPHERE_ROWS / 2)) as f32 * SPHERE_SPACING,
                        0.0,
                    ),
                );
                self.pbr_shader
                    .set_uniform_mat4fv(model_name.as_c_str(), &model_matrix);
                self.pbr_shader.set_uniform_mat3fv(
                    CString::new("normal_matrix")?.as_c_str(),
                    &glm::transpose(&glm::inverse(
                        &model_matrix.fixed_view::<3, 3>(0, 0).clone_owned(),
                    )),
                );

                // Center sphere use PBR maps to render
                if row == 3 && column == 3 {
                    self.pbr_shader
                        .set_uniform_1i(CString::new("enable_pbr_map").unwrap().as_c_str(), 1);
                } else {
                    self.pbr_shader
                        .set_uniform_1i(CString::new("enable_pbr_map").unwrap().as_c_str(), 0);
                }

                self.render_sphere()?;
            }
        }

        // Render light source (simply re-render sphere at light positions)
        for light_pos in &POINT_LIGHT_POS {
            let mut model_matrix = glm::Mat4::identity();
            model_matrix = glm::translate(&model_matrix, light_pos);
            model_matrix = glm::scale(&model_matrix, &glm::vec3(0.5, 0.5, 0.5));
            self.pbr_shader
                .set_uniform_mat4fv(model_name.as_c_str(), &model_matrix);
            self.pbr_shader.set_uniform_mat3fv(
                CString::new("normal_matrix")?.as_c_str(),
                &glm::transpose(&glm::inverse(
                    &model_matrix.fixed_view::<3, 3>(0, 0).clone_owned(),
                )),
            );

            self.render_sphere()?;
        }

        // Swap buffers of window
        win.swap_buffers()?;

        Ok(())
    }

    pub fn render_sphere(&self) -> anyhow::Result<()> {
        // Draw sphere
        self.sphere_vao.bind();
        unsafe {
            gl::DrawElements(
                gl::TRIANGLE_STRIP,
                self.sphere_index_len as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }

        Ok(())
    }

    pub fn close(self) {
        self.pbr_shader.close();
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
