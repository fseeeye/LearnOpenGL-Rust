//! This example is about IBL.

// remove console window : https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ffi::CString, path::PathBuf};

use anyhow::bail;
use gl::types::*;

use learn::{
    clear_color, set_clear_color, Buffer, BufferBit, BufferType, BufferUsage, Camera, Model,
    ShaderProgram, Texture, TextureType, TextureUnit, VertexArray, VertexDescription, WinitWindow,
};
use learn_opengl_rs as learn;

use nalgebra as na;
use nalgebra_glm as glm;
use stb_image::stb_image;
use tracing::error;
use winit::event::Event;

/* Screen info */
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const BACKGROUND_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WINDOW_TITLE: &str = "IBL";

/* Camera data */
const CAMERA_POS: [f32; 3] = [0.0, 0.0, 3.0];
const CAMERA_LOOK_AT: [f32; 3] = [0.0, 0.0, -1.0];
const CAMERA_UP: [f32; 3] = [0.0, 1.0, 0.0];
const PROJECTION_FOV: f32 = std::f32::consts::FRAC_PI_4;
const PROJECTION_NEAR: f32 = 0.1;
const PROJECTION_FAR: f32 = 100.0;

/* Object data */
const SPHERE_ROWS: i32 = 7;
const SPHERE_COLUMNS: i32 = 7;
const SPHERE_SPACING: f32 = 2.5;
const QUAD_VERTICES: [[f32; 5]; 4] = [
    // positions + texture Coords
    [-1.0, 1.0, 0.0, 0.0, 1.0],
    [-1.0, -1.0, 0.0, 0.0, 0.0],
    [1.0, 1.0, 0.0, 1.0, 1.0],
    [1.0, -1.0, 0.0, 1.0, 0.0],
];

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

/* IBL values */
const CUBEMAP_WIDTH: i32 = 512;
const CUBEMAP_HEIGHT: i32 = 512;
const IRRADIANCE_MAP_WIDTH: i32 = 32;
const IRRADIANCE_MAP_HEIGHT: i32 = 32;
const PREFILTERED_MAP_WIDTH: i32 = 128;
const PREFILTERED_MAP_HEIGHT: i32 = 128;
const PREFILTERED_MAP_MIPMAP_LEVELS: i32 = 5;
const BRDF_LUT_WIDTH: i32 = 512;
const BRDF_LUT_HEIGHT: i32 = 512;

struct Renderer {
    sphere_vao: VertexArray,
    sphere_index_len: usize,

    pbr_shader: ShaderProgram,
    albedo_map: Texture,
    normal_map: Texture,
    metallic_map: Texture,
    roughness_map: Texture,
    ao_map: Texture,

    skybox_shader: ShaderProgram,
    cube_model: Model,
    env_cubemap: Texture,
    irradiance_map: Texture,
    prefiltered_envmap: Texture,
    brdf_lut_map: Texture,
}

impl Renderer {
    pub fn new() -> anyhow::Result<Self> {
        /* Extra Settings */

        // Configure global opengl state
        unsafe {
            // Enable Depth Test
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL); // Set to less than AND equal for skybox depth trick.
                                       // Cubemap
            gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS);
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

        // Load HDRI as a texture
        let hdri_map = Texture::new(
            PathBuf::from("assets/textures/hdr/newport_loft.hdr"),
            TextureType::Unknown,
        )?;
        let mut hdri_width: i32 = 0;
        let mut hdri_height: i32 = 0;
        let mut hdri_channels_in_file: i32 = 0;
        let hdri_img_data = unsafe {
            stb_image::stbi_set_flip_vertically_on_load(1);
            stb_image::stbi_loadf(
                "assets/textures/hdr/newport_loft.hdr".as_ptr().cast(),
                &mut hdri_width,
                &mut hdri_height,
                &mut hdri_channels_in_file,
                0,
            )
        };
        if hdri_img_data.is_null() {
            bail!("Failed to load HDR image.");
        }
        tracing::debug!(
            "HDR image loaded. path: {:?}, channels: {:?}, size: {:?}x{:?}",
            hdri_map.path,
            hdri_channels_in_file,
            hdri_width,
            hdri_height
        );
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, hdri_map.id);
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
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB16F as GLint,
                hdri_width,
                hdri_height,
                0,
                gl::RGB,
                gl::FLOAT,
                hdri_img_data.cast(),
            );
        }
        unsafe {
            stb_image::stbi_image_free(hdri_img_data.cast());
        }

        /* Shaders */

        // Create shader of PBR
        let pbr_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/pbr/027-pbr.vert"),
            include_str!("../../assets/shaders/pbr/027-pbr.frag"),
        )?;
        let equirectangular_to_cubemap_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/pbr/027-cubemap.vert"),
            include_str!("../../assets/shaders/pbr/027-equirectangular-to-cubemap.frag"),
        )?;
        let skybox_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/pbr/027-skybox.vert"),
            include_str!("../../assets/shaders/pbr/027-skybox.frag"),
        )?;
        let irradiance_map_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/pbr/027-cubemap.vert"),
            include_str!("../../assets/shaders/pbr/027-irradiance-map.frag"),
        )?;
        let prefilter_envmap_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/pbr/027-cubemap.vert"),
            include_str!("../../assets/shaders/pbr/027-prefilter-envmap.frag"),
        )?;
        let prefilter_brdf_shader = ShaderProgram::create_from_source(
            include_str!("../../assets/shaders/pbr/027-prefilter-brdf.vert"),
            include_str!("../../assets/shaders/pbr/027-prefilter-brdf.frag"),
        )?;

        /* Object Models */
        let cube_model: Model = Model::new(PathBuf::from("assets/models/cube_wood/cube.obj"))?;

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

        /* Get environment cubemap */

        // Prepare cubemap framebuffer
        let mut env_cubemap_fbo = 0;
        let mut env_cubemap_rbo = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut env_cubemap_fbo);
            gl::GenRenderbuffers(1, &mut env_cubemap_rbo);

            gl::BindFramebuffer(gl::FRAMEBUFFER, env_cubemap_fbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, env_cubemap_rbo);
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH_COMPONENT24,
                CUBEMAP_WIDTH,
                CUBEMAP_HEIGHT,
            );
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::RENDERBUFFER,
                env_cubemap_rbo,
            );
        }
        // Prepare cubemap texture
        let env_cubemap = Texture::new(PathBuf::new(), TextureType::Cubemap)?;
        unsafe {
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, env_cubemap.id);
            for i in 0..6 {
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
                    0,
                    gl::RGB16F as GLint,
                    CUBEMAP_WIDTH,
                    CUBEMAP_HEIGHT,
                    0,
                    gl::RGB,
                    gl::FLOAT,
                    core::ptr::null(),
                );
            }
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_R,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as GLint,
            );
        }

        // Convert HDR equirectangular environment map to cubemap equivalent
        let env_cubemap_projection =
            glm::perspective(1.0, glm::radians(&glm::vec1(90.0))[0], 0.1, 10.0);
        let env_cubemap_views = [
            glm::look_at(
                &glm::vec3(0.0, 0.0, 0.0),
                &glm::vec3(1.0, 0.0, 0.0),
                &glm::vec3(0.0, -1.0, 0.0),
            ),
            glm::look_at(
                &glm::vec3(0.0, 0.0, 0.0),
                &glm::vec3(-1.0, 0.0, 0.0),
                &glm::vec3(0.0, -1.0, 0.0),
            ),
            glm::look_at(
                &glm::vec3(0.0, 0.0, 0.0),
                &glm::vec3(0.0, 1.0, 0.0),
                &glm::vec3(0.0, 0.0, 1.0),
            ),
            glm::look_at(
                &glm::vec3(0.0, 0.0, 0.0),
                &glm::vec3(0.0, -1.0, 0.0),
                &glm::vec3(0.0, 0.0, -1.0),
            ),
            glm::look_at(
                &glm::vec3(0.0, 0.0, 0.0),
                &glm::vec3(0.0, 0.0, 1.0),
                &glm::vec3(0.0, -1.0, 0.0),
            ),
            glm::look_at(
                &glm::vec3(0.0, 0.0, 0.0),
                &glm::vec3(0.0, 0.0, -1.0),
                &glm::vec3(0.0, -1.0, 0.0),
            ),
        ];
        equirectangular_to_cubemap_shader.bind();
        equirectangular_to_cubemap_shader.set_uniform_mat4fv(
            CString::new("projection").unwrap().as_c_str(),
            &env_cubemap_projection,
        );
        equirectangular_to_cubemap_shader.set_texture_unit(
            CString::new("equirectangular_map").unwrap().as_c_str(),
            &hdri_map,
            TextureUnit::TEXTURE0,
        );
        unsafe {
            gl::Viewport(0, 0, CUBEMAP_WIDTH, CUBEMAP_HEIGHT);
            gl::BindFramebuffer(gl::FRAMEBUFFER, env_cubemap_fbo);
        }
        for (i, view_matrix) in env_cubemap_views.iter().enumerate() {
            equirectangular_to_cubemap_shader
                .set_uniform_mat4fv(CString::new("view").unwrap().as_c_str(), view_matrix);
            unsafe {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as GLenum,
                    env_cubemap.id,
                    0,
                );
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }
            cube_model.draw(&equirectangular_to_cubemap_shader, "material")?;
        }

        // Generate mipmaps of envmap (combatting visible dots artifact)
        unsafe {
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, env_cubemap.id);
            gl::GenerateMipmap(gl::TEXTURE_CUBE_MAP);
        }

        /* Prefilter Environment Cubemap on hemisphere */

        let irradiance_map = Texture::new(PathBuf::new(), TextureType::Cubemap)?;
        unsafe {
            // Prepare prefiltered environment cubemap texture
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, irradiance_map.id);
            for i in 0..6 {
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
                    0,
                    gl::RGB16F as GLint,
                    IRRADIANCE_MAP_WIDTH,
                    IRRADIANCE_MAP_HEIGHT,
                    0,
                    gl::RGB,
                    gl::FLOAT,
                    core::ptr::null(),
                );
            }
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_R,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as GLint,
            );

            // Scale fbo to prefilter cubemap size
            gl::BindFramebuffer(gl::FRAMEBUFFER, env_cubemap_fbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, env_cubemap_rbo);
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH_COMPONENT24,
                IRRADIANCE_MAP_WIDTH,
                IRRADIANCE_MAP_HEIGHT,
            );
            gl::Viewport(0, 0, IRRADIANCE_MAP_WIDTH, IRRADIANCE_MAP_HEIGHT);
        }

        // Do prefiltering
        irradiance_map_shader.bind();
        irradiance_map_shader.set_uniform_mat4fv(
            CString::new("projection").unwrap().as_c_str(),
            &env_cubemap_projection,
        );
        irradiance_map_shader.set_texture_unit(
            CString::new("environment_map").unwrap().as_c_str(),
            &env_cubemap,
            TextureUnit::TEXTURE0,
        );
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, env_cubemap_fbo);
        }
        for (i, view_matrix) in env_cubemap_views.iter().enumerate() {
            irradiance_map_shader
                .set_uniform_mat4fv(CString::new("view").unwrap().as_c_str(), view_matrix);
            unsafe {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as GLenum,
                    irradiance_map.id,
                    0,
                );
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }
            cube_model.draw(&irradiance_map_shader, "material")?;
        }

        /* Prefilter Environment Cubemap (The Split Sum) */

        // Prepare prefiltered environment cubemap
        let prefiltered_envmap = Texture::new(PathBuf::new(), TextureType::Cubemap)?;
        unsafe {
            // Prepare prefiltered environment cubemap texture
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, prefiltered_envmap.id);
            for i in 0..6 {
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
                    0,
                    gl::RGB16F as GLint,
                    PREFILTERED_MAP_WIDTH,
                    PREFILTERED_MAP_HEIGHT,
                    0,
                    gl::RGB,
                    gl::FLOAT,
                    core::ptr::null(),
                );
            }
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_R,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as GLint,
            );
            // Make sure to set minification filter to mip_linear
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as GLint,
            );

            // Generate mipmaps for the cubemap so OpenGL automatically allocates the required memory.
            gl::GenerateMipmap(gl::TEXTURE_CUBE_MAP);
        }

        // Prefiltering the environment cubemap of all mipmap levels
        prefilter_envmap_shader.bind();
        prefilter_envmap_shader.set_uniform_mat4fv(
            CString::new("projection").unwrap().as_c_str(),
            &env_cubemap_projection,
        );
        irradiance_map_shader.set_texture_unit(
            CString::new("environment_map").unwrap().as_c_str(),
            &env_cubemap,
            TextureUnit::TEXTURE0,
        );
        for mip_level in 0..PREFILTERED_MAP_MIPMAP_LEVELS {
            // 1. reisze framebuffer according to mip-level size.
            let mip_width = (PREFILTERED_MAP_WIDTH as f32 * (0.5_f32).powi(mip_level)) as i32;
            let mip_height = (PREFILTERED_MAP_HEIGHT as f32 * (0.5_f32).powi(mip_level)) as i32;
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, env_cubemap_fbo);
                gl::BindRenderbuffer(gl::RENDERBUFFER, env_cubemap_rbo);
                gl::RenderbufferStorage(
                    gl::RENDERBUFFER,
                    gl::DEPTH_COMPONENT24,
                    mip_width,
                    mip_height,
                );
                gl::Viewport(0, 0, mip_width, mip_height);
            }
            // 2. Set the roughness of this mipmap level : level1 is 0, level2 is 1/4, ...
            let target_roughness = mip_level as f32 / (PREFILTERED_MAP_MIPMAP_LEVELS - 1) as f32;
            // 3. Do prefiltering
            prefilter_envmap_shader.set_uniform_1f(
                CString::new("target_roughness").unwrap().as_c_str(),
                target_roughness,
            );
            for (i, view_matrix) in env_cubemap_views.iter().enumerate() {
                prefilter_envmap_shader
                    .set_uniform_mat4fv(CString::new("view").unwrap().as_c_str(), view_matrix);
                unsafe {
                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        gl::COLOR_ATTACHMENT0,
                        gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as GLenum,
                        prefiltered_envmap.id,
                        mip_level, // set the level of prefiltered environment map
                    );
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                }
                cube_model.draw(&prefilter_envmap_shader, "material")?;
            }
        }

        /* Prefiltering BRDF (The Spilt Sum) */

        // Prepare 2D Texture for LUT
        let brdf_lut_map = Texture::new(PathBuf::new(), TextureType::Unknown)?;
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, brdf_lut_map.id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB16F as GLint,
                BRDF_LUT_WIDTH,
                BRDF_LUT_HEIGHT,
                0,
                gl::RG,
                gl::FLOAT,
                core::ptr::null(),
            );
            // be sure to set wrapping mode to GL_CLAMP_TO_EDGE
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
            // Make sure to set minification filter to mip_linear
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        }

        // Reconfigure framebuffer
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, env_cubemap_fbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, env_cubemap_rbo);
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH_COMPONENT24,
                BRDF_LUT_WIDTH,
                BRDF_LUT_HEIGHT,
            );
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                brdf_lut_map.id,
                0,
            );
            gl::Viewport(0, 0, BRDF_LUT_WIDTH, BRDF_LUT_HEIGHT);
        }

        // Generate 2D LUT from the BRDF equations used.
        prefilter_brdf_shader.bind();
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        let quad_vao = VertexArray::new()?;
        let quad_vbo = Buffer::new(BufferType::VertexBuffer)?;
        quad_vao.bind();
        quad_vbo.bind();
        quad_vbo.set_buffer_data(QUAD_VERTICES.as_slice(), BufferUsage::StaticDraw);
        let mut quad_vertex_desc = VertexDescription::new();
        quad_vertex_desc.add_attribute(gl::FLOAT, 3); // set coords attribute
        quad_vertex_desc.add_attribute(gl::FLOAT, 2); // set Texture coord attribute
        quad_vertex_desc.bind_to(&quad_vbo, Some(&quad_vao));
        unsafe {
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
        quad_vao.unbind();

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(0, 0, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
        }

        Ok(Self {
            skybox_shader,
            pbr_shader,
            sphere_vao,
            sphere_index_len,
            albedo_map,
            normal_map,
            metallic_map,
            roughness_map,
            ao_map,
            cube_model,
            env_cubemap,
            irradiance_map,
            prefiltered_envmap,
            brdf_lut_map,
        })
    }

    pub fn redraw(
        &self,
        win: &WinitWindow,
        camera: &Camera,
        _delta_time: f32,
    ) -> anyhow::Result<()> {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
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

        // Bind uniforms
        self.pbr_shader
            .set_uniform_3f(CString::new("albedo")?.as_c_str(), 0.5, 0.0, 0.0);
        self.pbr_shader
            .set_uniform_1f(CString::new("ao")?.as_c_str(), 1.0);
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
        self.pbr_shader.set_texture_unit(
            CString::new("irradiance_map").unwrap().as_c_str(),
            &self.irradiance_map,
            TextureUnit::TEXTURE5,
        );
        self.pbr_shader.set_texture_unit(
            CString::new("prefiltered_envmap").unwrap().as_c_str(),
            &self.prefiltered_envmap,
            TextureUnit::TEXTURE6,
        );
        self.pbr_shader.set_texture_unit(
            CString::new("brdf_lut").unwrap().as_c_str(),
            &self.brdf_lut_map,
            TextureUnit::TEXTURE7,
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

        /* Pass 2 : Render light source */

        // simply re-render sphere at light positions
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

        /* Pass 3 : render skybox */

        self.skybox_shader.bind();
        self.skybox_shader
            .set_uniform_mat4fv(view_name.as_c_str(), &object_view_matrix);
        self.skybox_shader
            .set_uniform_mat4fv(projection_name.as_c_str(), &projection_matrix);
        self.skybox_shader.set_texture_unit(
            &CString::new("environment_map")?,
            &self.env_cubemap,
            TextureUnit::TEXTURE1,
        );
        self.cube_model.draw(&self.skybox_shader, "material")?;

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
