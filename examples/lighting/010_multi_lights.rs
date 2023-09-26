//! This example is about impl multiple types of light source.

// remove console window : https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ffi::CString, path::PathBuf};

use anyhow::bail;
use gl::types::*;

use learn::{
    clear_color, set_clear_color, Buffer, BufferBit, BufferType, BufferUsage, Camera,
    DirectionalLight, FlashLight, MaterialPhong, PointLight, ShaderProgram, Texture, VertexArray,
    VertexDescription, WinitWindow,
};
use learn_opengl_rs as learn;

use nalgebra as na;
use tracing::error;
use winit::event::Event;

/* Screen info */
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const BACKGROUND_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/* Camera data */
const CAMERA_POS: [f32; 3] = [0.0, 0.0, 5.0];

/* Vertex data */
type Vertex = [f32; 8]; // NDC coord(3) + Normal(3) + Texture coord(2)
const CUBE_VERTICES: [Vertex; 36] = [
    // Panel 1
    [-0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 0.0],
    [0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 0.0],
    [0.5, 0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 1.0],
    [0.5, 0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 1.0],
    [-0.5, 0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 1.0],
    [-0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 0.0],
    // Panel 2
    [-0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0],
    [0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 0.0],
    [0.5, 0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 1.0],
    [0.5, 0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 1.0],
    [-0.5, 0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 1.0],
    [-0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0],
    // Panel 3
    [-0.5, 0.5, 0.5, -1.0, 0.0, 0.0, 1.0, 0.0],
    [-0.5, 0.5, -0.5, -1.0, 0.0, 0.0, 1.0, 1.0],
    [-0.5, -0.5, -0.5, -1.0, 0.0, 0.0, 0.0, 1.0],
    [-0.5, -0.5, -0.5, -1.0, 0.0, 0.0, 0.0, 1.0],
    [-0.5, -0.5, 0.5, -1.0, 0.0, 0.0, 0.0, 0.0],
    [-0.5, 0.5, 0.5, -1.0, 0.0, 0.0, 1.0, 0.0],
    // Panel 4
    [0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 1.0, 0.0],
    [0.5, 0.5, -0.5, 1.0, 0.0, 0.0, 1.0, 1.0],
    [0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.0, 1.0],
    [0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.0, 1.0],
    [0.5, -0.5, 0.5, 1.0, 0.0, 0.0, 0.0, 0.0],
    [0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 1.0, 0.0],
    // Panel 5
    [-0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.0, 1.0],
    [0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 1.0, 1.0],
    [0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 1.0, 0.0],
    [0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 1.0, 0.0],
    [-0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 0.0, 0.0],
    [-0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.0, 1.0],
    // Panel 6
    [-0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 1.0],
    [0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 1.0, 1.0],
    [0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 1.0, 0.0],
    [0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 1.0, 0.0],
    [-0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 0.0],
    [-0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 1.0],
];

const CUBE_POSTIONS: [na::Vector3<f32>; 10] = [
    na::Vector3::new(0.0, 0.0, 0.0),
    na::Vector3::new(2.0, 5.0, -15.0),
    na::Vector3::new(-1.5, -2.2, -2.5),
    na::Vector3::new(-3.8, -2.0, -12.3),
    na::Vector3::new(2.4, -0.4, -3.5),
    na::Vector3::new(-1.7, 3.0, -7.5),
    na::Vector3::new(1.3, -2.0, -2.5),
    na::Vector3::new(1.5, 2.0, -2.5),
    na::Vector3::new(1.5, 0.2, -1.5),
    na::Vector3::new(-1.3, 1.0, -1.5),
];

/* Lighting data */
const LIGHT_COLOR: na::Vector3<f32> = na::Vector3::new(1.0, 1.0, 1.0);
const FALLOFF_LINEAR: f32 = 0.09;
const FALLOFF_QUADRATIC: f32 = 0.032;
const DIR_LIGHT_DIRECTION: na::Vector3<f32> = na::Vector3::new(-0.2, -1.0, -0.3);
const POINT_LIGHT_POS: [na::Vector3<f32>; 4] = [
    na::Vector3::new(0.7, 0.2, 2.0),
    na::Vector3::new(2.3, -3.3, -4.0),
    na::Vector3::new(-4.0, 2.0, -12.0),
    na::Vector3::new(0.0, 0.0, -3.0),
];
const FLASH_LIGHT_CUTOFF: f32 = 12.5_f32;
const FLASH_LIGHT_OUTER_CUTOFF: f32 = 15.0_f32;

struct Renderer {
    cube_shader: ShaderProgram,
    cube_vao: VertexArray,
    light_shader: ShaderProgram,
    light_vao: VertexArray,
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
        // Enable Depth Test
        unsafe { gl::Enable(gl::DEPTH_TEST) };

        /* Light */

        // Prepare light casters
        let dir_light = DirectionalLight::new(DIR_LIGHT_DIRECTION, LIGHT_COLOR);

        let point_lights: Vec<PointLight> = POINT_LIGHT_POS
            .iter()
            .map(|&point_light_pos| {
                PointLight::new(
                    point_light_pos,
                    LIGHT_COLOR,
                    FALLOFF_LINEAR,
                    FALLOFF_QUADRATIC,
                )
            })
            .collect();

        // Prepare vertex of light
        let light_vao = VertexArray::new()?;

        let lighting_vbo = Buffer::new(BufferType::VertexBuffer)?;
        lighting_vbo.bind();
        lighting_vbo.set_buffer_data(CUBE_VERTICES.as_slice(), BufferUsage::StaticDraw);

        light_vao.bind();
        let mut cube_vertex_desc = VertexDescription::new();
        cube_vertex_desc.add_attribute(gl::FLOAT, 3); // set NDC coords attribute
        cube_vertex_desc.add_attribute(gl::FLOAT, 3); // set normal attribute
        cube_vertex_desc.add_attribute(gl::FLOAT, 2); // set Texture coord attribute
        cube_vertex_desc.bind_to(&lighting_vbo, Some(&light_vao));

        // Prepare shader of light
        let light_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/lighting/010-lighting.vert"),
            include_str!("../../assets/shaders/lighting/010-lighting.frag"),
        )?;
        light_shader.set_uniform_3f(
            CString::new("light_color")?.as_c_str(),
            LIGHT_COLOR[0],
            LIGHT_COLOR[1],
            LIGHT_COLOR[2],
        );

        /* Cube Vertices & Shader */

        // Prepare cube material
        let texture_diffuse =
            Texture::create(PathBuf::from("assets/textures/container2.png"), None)?;
        let texture_specular = Texture::create(
            PathBuf::from("assets/textures/container2_specular.png"),
            None,
        )?;
        let cube_material = MaterialPhong::new(texture_diffuse, texture_specular, 128.0, None);

        // Prepare vertex of cube
        let cube_vao = VertexArray::new()?;

        let cube_vbo = Buffer::new(BufferType::VertexBuffer)?;
        cube_vbo.bind();
        cube_vbo.set_buffer_data(CUBE_VERTICES.as_slice(), BufferUsage::StaticDraw);

        cube_vao.bind();
        let mut cube_vertex_desc = VertexDescription::new();
        cube_vertex_desc.add_attribute(gl::FLOAT, 3); // set NDC coord attribute
        cube_vertex_desc.add_attribute(gl::FLOAT, 3); // set normal attribute
        cube_vertex_desc.add_attribute(gl::FLOAT, 2); // set Texture coord attribute
        cube_vertex_desc.bind_to(&cube_vbo, Some(&cube_vao));

        // Prepare shader of cube
        let cube_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/lighting/010-cube.vert"),
            include_str!("../../assets/shaders/lighting/010-cube.frag"),
        )?;

        cube_shader.set_uniform_material_phong(String::from("material"), &cube_material)?;
        cube_shader.set_uniform_directional_light(String::from("dir_light"), &dir_light)?;
        for (i, point_light) in point_lights.iter().enumerate() {
            cube_shader.set_uniform_point_light(format!("point_lights[{i}]"), point_light)?;
        }

        Ok(Self {
            cube_shader,
            cube_vao,
            light_shader,
            light_vao,
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

        let flash_light = FlashLight::new(
            LIGHT_COLOR,
            FLASH_LIGHT_CUTOFF.to_radians().cos(),
            FLASH_LIGHT_OUTER_CUTOFF.to_radians().cos(),
            FALLOFF_LINEAR,
            FALLOFF_QUADRATIC,
        );

        // Model Matrix
        let model_name = CString::new("model")?;
        let normal_matrix_name = CString::new("normal_matrix")?;

        // View Matrix
        let view_name = CString::new("view")?;
        let cube_view_matrix = camera.get_lookat_matrix();

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

        /* Draw cube */

        self.cube_vao.bind();
        self.cube_shader.bind();

        self.cube_shader
            .set_uniform_mat4fv(view_name.as_c_str(), &cube_view_matrix);
        self.cube_shader
            .set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);
        self.cube_shader.set_uniform_3f(
            CString::new("camera_pos")?.as_c_str(),
            camera.get_pos().x,
            camera.get_pos().y,
            camera.get_pos().z,
        );
        self.cube_shader.set_uniform_flash_light(
            String::from("spot_light"),
            &flash_light,
            camera,
        )?;

        for cube_position in CUBE_POSTIONS {
            // Model Matrix & Normal Matrix of cube
            // let model_matrix_rotation = na::Rotation3::from_axis_angle(
            //     &na::Unit::new_normalize(na::Vector3::new(0.5, 1.0, 0.0)),
            //     -std::f32::consts::PI / 3.0 * delta_time,
            // )
            // .to_homogeneous();
            let model_matrix_transform = na::Translation3::from(cube_position).to_homogeneous();
            let cube_model_matrix = model_matrix_transform;
            let cube_normal_matrix = cube_model_matrix
                .fixed_view::<3, 3>(0, 0)
                .try_inverse()
                .unwrap()
                .transpose();

            self.cube_shader
                .set_uniform_mat4fv(model_name.as_c_str(), &cube_model_matrix);
            self.cube_shader
                .set_uniform_mat3fv(normal_matrix_name.as_c_str(), &cube_normal_matrix);

            // Draw
            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }
        }

        /* Draw lighting */

        self.light_vao.bind();
        self.light_shader.bind();

        self.light_shader
            .set_uniform_mat4fv(view_name.as_c_str(), &camera.get_lookat_matrix());
        self.light_shader
            .set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);

        for point_light_pos in POINT_LIGHT_POS {
            // Model matrix of light
            let light_model_matrix_scale = na::Matrix4::new_scaling(0.2);
            let light_model_matrix = light_model_matrix_scale.append_translation(&point_light_pos);
            self.light_shader
                .set_uniform_mat4fv(model_name.as_c_str(), &light_model_matrix);

            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }
        }

        // Swap buffers of window
        win.swap_buffers()?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn close(self) {
        self.cube_shader.close();
        self.light_shader.close();
    }
}

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
    let mut camera = Camera::new(camera_pos, camera_look_at, camera_up);

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
            error!("Failed to create renderer: {}", e);
            bail!(e);
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
}
