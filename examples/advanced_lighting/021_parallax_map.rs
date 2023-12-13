//! This example has more infos about parallax map.

// remove console window : https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ffi::CString, path::PathBuf};

use anyhow::bail;
use gl::types::*;

use learn::{
    clear_color, set_clear_color, Buffer, BufferBit, BufferType, BufferUsage, Camera,
    ShaderProgram, Texture, TextureType, TextureUnit, VertexArray, VertexDescription, WinitWindow,
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
const WINDOW_TITLE: &str = "Parallax Map";

/* Camera data */
const CAMERA_POS: [f32; 3] = [0.0, 0.0, 3.0];
const CAMERA_LOOK_AT: [f32; 3] = [0.0, 0.0, -1.0];
const CAMERA_UP: [f32; 3] = [0.0, 1.0, 0.0];
const PROJECTION_FOV: f32 = std::f32::consts::FRAC_PI_4;
const PROJECTION_NEAR: f32 = 0.1;
const PROJECTION_FAR: f32 = 100.0;

/* Light data */
const LIGHT_POS: [f32; 3] = [0.5, 1.0, 0.3];

/* Wall data */
lazy_static::lazy_static! {
    // positions
    static ref WALL_POSITIONS: [glm::Vec3; 4] = [
        glm::vec3(-1.0, 1.0, 0.0),
        glm::vec3(-1.0, -1.0, 0.0),
        glm::vec3(1.0, -1.0, 0.0),
        glm::vec3(1.0, 1.0, 0.0),
    ];
    // texture coordinates
    static ref WALL_TEX_COORDS: [glm::Vec2; 4] = [
        glm::vec2(0.0, 1.0),
        glm::vec2(0.0, 0.0),
        glm::vec2(1.0, 0.0),
        glm::vec2(1.0, 1.0),
    ];
    // normal vector
    static ref WALL_NORMAL: glm::Vec3 = glm::vec3(0.0, 0.0, 1.0);
}

struct Renderer {
    wall_vertices: [[f32; 14]; 6],
    wall_vao: VertexArray,
    wall_diffuse_map: Texture,
    wall_normal_map: Texture,
    wall_displacement_map: Texture,
    object_shader: ShaderProgram,
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

        /* Shaders */

        // Create shader of object
        let object_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/advanced_lighting/021-object.vert"),
            include_str!("../../assets/shaders/advanced_lighting/021-object.frag"),
        )?;
        object_shader.set_uniform_3f(
            CString::new("light_pos")?.as_c_str(),
            LIGHT_POS[0],
            LIGHT_POS[1],
            LIGHT_POS[2],
        );

        /* Wall Object */

        // Textures of wall
        let wall_diffuse_map = Texture::create(
            PathBuf::from("assets/textures/bricks2.jpg"),
            Some(TextureType::Diffuse),
        )?;
        let wall_normal_map = Texture::create(
            PathBuf::from("assets/textures/bricks2_normal.jpg"),
            Some(TextureType::Diffuse),
        )?;
        let wall_displacement_map = Texture::create(
            PathBuf::from("assets/textures/bricks2_disp.jpg"),
            Some(TextureType::Diffuse),
        )?;

        // Calculate TB(tangent & bitangent) vectors of Triangle 1
        let mut tangent1: glm::Vec3;
        let mut bitangent1: glm::Vec3;

        let edge1: glm::Vec3 = WALL_POSITIONS[1] - WALL_POSITIONS[0];
        let edge2: glm::Vec3 = WALL_POSITIONS[2] - WALL_POSITIONS[0];
        let delta_uv1: glm::Vec2 = WALL_TEX_COORDS[1] - WALL_TEX_COORDS[0];
        let delta_uv2: glm::Vec2 = WALL_TEX_COORDS[2] - WALL_TEX_COORDS[0];

        let f: f32 = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

        tangent1 = glm::vec3(
            f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
            f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
            f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z),
        );
        tangent1 = tangent1.normalize();
        bitangent1 = glm::vec3(
            f * (-delta_uv2.x * edge1.x + delta_uv1.x * edge2.x),
            f * (-delta_uv2.x * edge1.y + delta_uv1.x * edge2.y),
            f * (-delta_uv2.x * edge1.z + delta_uv1.x * edge2.z),
        );
        bitangent1 = bitangent1.normalize();

        // Calculate TB(tangent & bitangent) vectors of Triangle 2
        let mut tangent2: glm::Vec3;
        let mut bitangent2: glm::Vec3;

        let edge1: glm::Vec3 = WALL_POSITIONS[2] - WALL_POSITIONS[0];
        let edge2: glm::Vec3 = WALL_POSITIONS[3] - WALL_POSITIONS[0];
        let delta_uv1: glm::Vec2 = WALL_TEX_COORDS[2] - WALL_TEX_COORDS[0];
        let delta_uv2: glm::Vec2 = WALL_TEX_COORDS[3] - WALL_TEX_COORDS[0];

        let f: f32 = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

        tangent2 = glm::vec3(
            f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
            f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
            f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z),
        );
        tangent2 = tangent2.normalize();
        bitangent2 = glm::vec3(
            f * (-delta_uv2.x * edge1.x + delta_uv1.x * edge2.x),
            f * (-delta_uv2.x * edge1.y + delta_uv1.x * edge2.y),
            f * (-delta_uv2.x * edge1.z + delta_uv1.x * edge2.z),
        );
        bitangent2 = bitangent2.normalize();

        tracing::info!(
            "tangent1: {:?}, bitangent1: {:?}, tangent2: {:?}, bitangent2: {:?}",
            tangent1,
            bitangent1,
            tangent2,
            bitangent2
        );

        // Generate vertices data of wall
        let wall_vertices: [[f32; 14]; 6] = [
            // positions + normals + texcoords + tangent + bitangent
            [
                WALL_POSITIONS[0].x,
                WALL_POSITIONS[0].y,
                WALL_POSITIONS[0].z,
                WALL_NORMAL.x,
                WALL_NORMAL.y,
                WALL_NORMAL.z,
                WALL_TEX_COORDS[0].x,
                WALL_TEX_COORDS[0].y,
                tangent1.x,
                tangent1.y,
                tangent1.z,
                bitangent1.x,
                bitangent1.y,
                bitangent1.z,
            ],
            [
                WALL_POSITIONS[1].x,
                WALL_POSITIONS[1].y,
                WALL_POSITIONS[1].z,
                WALL_NORMAL.x,
                WALL_NORMAL.y,
                WALL_NORMAL.z,
                WALL_TEX_COORDS[1].x,
                WALL_TEX_COORDS[1].y,
                tangent1.x,
                tangent1.y,
                tangent1.z,
                bitangent1.x,
                bitangent1.y,
                bitangent1.z,
            ],
            [
                WALL_POSITIONS[2].x,
                WALL_POSITIONS[2].y,
                WALL_POSITIONS[2].z,
                WALL_NORMAL.x,
                WALL_NORMAL.y,
                WALL_NORMAL.z,
                WALL_TEX_COORDS[2].x,
                WALL_TEX_COORDS[2].y,
                tangent1.x,
                tangent1.y,
                tangent1.z,
                bitangent1.x,
                bitangent1.y,
                bitangent1.z,
            ],
            [
                WALL_POSITIONS[0].x,
                WALL_POSITIONS[0].y,
                WALL_POSITIONS[0].z,
                WALL_NORMAL.x,
                WALL_NORMAL.y,
                WALL_NORMAL.z,
                WALL_TEX_COORDS[0].x,
                WALL_TEX_COORDS[0].y,
                tangent2.x,
                tangent2.y,
                tangent2.z,
                bitangent2.x,
                bitangent2.y,
                bitangent2.z,
            ],
            [
                WALL_POSITIONS[2].x,
                WALL_POSITIONS[2].y,
                WALL_POSITIONS[2].z,
                WALL_NORMAL.x,
                WALL_NORMAL.y,
                WALL_NORMAL.z,
                WALL_TEX_COORDS[2].x,
                WALL_TEX_COORDS[2].y,
                tangent2.x,
                tangent2.y,
                tangent2.z,
                bitangent2.x,
                bitangent2.y,
                bitangent2.z,
            ],
            [
                WALL_POSITIONS[3].x,
                WALL_POSITIONS[3].y,
                WALL_POSITIONS[3].z,
                WALL_NORMAL.x,
                WALL_NORMAL.y,
                WALL_NORMAL.z,
                WALL_TEX_COORDS[3].x,
                WALL_TEX_COORDS[3].y,
                tangent2.x,
                tangent2.y,
                tangent2.z,
                bitangent2.x,
                bitangent2.y,
                bitangent2.z,
            ],
        ];

        // Generate VAO of wall
        let wall_vao = VertexArray::new()?;
        wall_vao.bind();

        let wall_vbo = Buffer::new(BufferType::VertexBuffer)?;
        wall_vbo.bind();
        unsafe {
            gl::BufferData(
                wall_vbo.buffer_type as GLenum,
                std::mem::size_of_val(&wall_vertices) as GLsizeiptr,
                wall_vertices.as_ptr().cast(),
                BufferUsage::StaticDraw as GLenum,
            );
        }

        let mut wall_vertex_desc = VertexDescription::new();
        wall_vertex_desc.add_attribute(gl::FLOAT, 3); // set NDC coord attribute
        wall_vertex_desc.add_attribute(gl::FLOAT, 3); // set normal attribute
        wall_vertex_desc.add_attribute(gl::FLOAT, 2); // set Texture coord attribute
        wall_vertex_desc.add_attribute(gl::FLOAT, 3); // set tangent of TBN attribute
        wall_vertex_desc.add_attribute(gl::FLOAT, 3); // set bitangent of TBN attribute
        wall_vertex_desc.bind_to(&wall_vbo, Some(&wall_vao));
        wall_vao.unbind();
        wall_vbo.unbind();

        Ok(Self {
            wall_vertices,
            wall_vao,
            wall_diffuse_map,
            wall_normal_map,
            wall_displacement_map,
            object_shader,
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

        /* Pass 1 : Draw object */
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

        // Swap buffers of window
        win.swap_buffers()?;

        Ok(())
    }

    pub fn render_scence(&self, shader: &ShaderProgram) -> anyhow::Result<()> {
        lazy_static::lazy_static! {
            static ref SYSTEM_TIME: std::time::SystemTime = std::time::SystemTime::now();
        }
        /* Wall */

        // Setup shader uniform: model matrix
        let model_name = CString::new("model")?;
        let mut object_model_matrix = glm::Mat4::identity();
        object_model_matrix = glm::rotate(
            &object_model_matrix,
            glm::radians(&glm::TVec::<f32, 1>::new(
                SYSTEM_TIME.elapsed().unwrap().as_secs_f32() * -20.0,
            ))[0],
            &glm::normalize(&glm::vec3(1.0, 0.0, 1.0)),
        );
        let normal_matrix_name = CString::new("normal_matrix")?;
        let object_normal_matrix = object_model_matrix
            .fixed_view::<3, 3>(0, 0)
            .try_inverse()
            .unwrap()
            .transpose();
        shader.set_uniform_mat4fv(model_name.as_c_str(), &object_model_matrix); // Model Matrix
        shader.set_uniform_mat3fv(normal_matrix_name.as_c_str(), &object_normal_matrix); // Normal Matrix

        // Setup shader uniform: shininess & depth_scale
        let material_uniform_name = "material";
        shader.set_uniform_1f(
            &CString::new(format!("{material_uniform_name}.shininess"))?,
            32.0,
        );
        shader.set_uniform_1f(
            &CString::new(format!("{material_uniform_name}.depth_scale"))?,
            0.1,
        );

        // Setup shader uniform: diffuse map & normal map & displacement map
        let mut texture_unit = TextureUnit::TEXTURE10;
        shader.set_texture_unit(
            &CString::new(format!("{material_uniform_name}.diffuse_map"))?,
            &self.wall_diffuse_map,
            texture_unit,
        );
        texture_unit = texture_unit.increase();
        shader.set_texture_unit(
            &CString::new(format!("{material_uniform_name}.normal_map"))?,
            &self.wall_normal_map,
            texture_unit,
        );
        texture_unit = texture_unit.increase();
        shader.set_texture_unit(
            &CString::new(format!("{material_uniform_name}.displacement_map"))?,
            &self.wall_displacement_map,
            texture_unit,
        );

        // Draw wall
        self.wall_vao.bind();
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, self.wall_vertices.len() as GLsizei);
        }

        // always good practice to set everything back to defaults once configured.
        self.wall_vao.unbind();
        Texture::active(TextureUnit::TEXTURE0);

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
