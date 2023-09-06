use std::{ffi::CString, path::PathBuf};

use anyhow::{bail, Ok};
use gl::types::*;
use nalgebra as na;
use tracing::{debug, trace, warn};

use crate::{
    Buffer, BufferType, BufferUsage, ShaderProgram, Texture, TextureType, TextureUnit, Vertex,
    VertexArray, VertexDescription,
};

#[allow(dead_code)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub diffuse_texture: Option<Texture>,
    pub specular_texture: Option<Texture>,
    pub normal_texture: Option<Texture>,
    vao: VertexArray,
    vbo: Buffer,
    ibo: Buffer,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        diffuse_texture: Option<Texture>,
        specular_texture: Option<Texture>,
        normal_texture: Option<Texture>,
    ) -> anyhow::Result<Self> {
        /* Vertex Array Object */
        let vao = VertexArray::new()?;

        /* Vertex Buffer Object */
        let vbo = Buffer::new(BufferType::VertexBuffer)?;
        vbo.set_vertices(&vertices, BufferUsage::StaticDraw);

        /* Vertex Attribute description */
        let mut vertex_desc = VertexDescription::new();
        vertex_desc.add_attribute(gl::FLOAT, 3); // set NDC coord attribute
        vertex_desc.add_attribute(gl::FLOAT, 3); // set normal attribute
        vertex_desc.add_attribute(gl::FLOAT, 2); // set Texture coord attribute
        vertex_desc.bind_to(&vbo, Some(&vao));

        /* Index Buffer Object */
        let ibo = Buffer::new(BufferType::IndexBuffer)?;
        ibo.set_indices(&indices, BufferUsage::StaticDraw);

        vao.unbind();

        // Check texture type
        if let Some(diffuse_texture) = diffuse_texture.as_ref() {
            Self::check_texture_type(diffuse_texture, TextureType::Diffuse)?;
        }
        if let Some(specular_texture) = specular_texture.as_ref() {
            Self::check_texture_type(specular_texture, TextureType::Specular)?;
        }
        if let Some(normal_texture) = normal_texture.as_ref() {
            Self::check_texture_type(normal_texture, TextureType::Normal)?;
        }

        Ok(Self {
            vertices,
            indices,
            diffuse_texture,
            specular_texture,
            normal_texture,
            vao,
            vbo,
            ibo,
        })
    }

    pub fn draw(&self, shader: &ShaderProgram, material_uniform_name: &str) -> anyhow::Result<()> {
        // Bind textures
        let mut texture_unit = TextureUnit::TEXTURE0;
        if let Some(diffuse_texture) = &self.diffuse_texture {
            shader.set_texture_unit(
                &CString::new(format!("{material_uniform_name}.diffuse_map"))?,
                diffuse_texture,
                texture_unit,
            );
            texture_unit = texture_unit.increase();
        }
        if let Some(specular_texture) = &self.specular_texture {
            shader.set_texture_unit(
                &CString::new(format!("{material_uniform_name}.specular_map"))?,
                specular_texture,
                texture_unit,
            );
            texture_unit = texture_unit.increase();
        }
        if let Some(normal_texture) = &self.normal_texture {
            shader.set_texture_unit(
                &CString::new(format!("{material_uniform_name}.normal_map"))?,
                normal_texture,
                texture_unit,
            );
            // texture_unit = texture_unit.increase();
        }

        // Draw mesh
        self.vao.bind();
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }

        // always good practice to set everything back to defaults once configured.
        self.vao.unbind();
        Texture::active(TextureUnit::TEXTURE0);

        Ok(())
    }

    fn check_texture_type(texture: &Texture, texture_type: TextureType) -> anyhow::Result<()> {
        if texture.tex_type != texture_type {
            bail!(
                "Texture type({:?}) is not matched with expected type({:?})",
                texture.tex_type,
                texture_type
            );
        }

        Ok(())
    }
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    textures_loaded: Vec<Texture>,
    model_path: PathBuf,
}

impl Model {
    pub fn new(model_path: PathBuf) -> anyhow::Result<Model> {
        let mut model = Self {
            meshes: Vec::new(),
            textures_loaded: Vec::new(),
            model_path,
        };

        model.load_model()?;

        Ok(model)
    }

    pub fn draw(&self, shader: &ShaderProgram, material_uniform_name: &str) -> anyhow::Result<()> {
        for mesh in self.meshes.iter() {
            mesh.draw(shader, material_uniform_name)?;
        }

        Ok(())
    }

    fn load_model(&mut self) -> anyhow::Result<()> {
        debug!("Loading model from {:?}", &self.model_path);

        // Load .obj file
        let (models, materials) = tobj::load_obj(
            &self.model_path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ignore_points: true,
                ignore_lines: true,
            },
        )?;
        let materials = materials?;

        trace!("Number of models          = {}", models.len());
        trace!("Number of materials       = {}", materials.len());

        // Load Meshes
        for (i, model) in models.iter().enumerate() {
            trace!("model[{}].name             = \'{}\'", i, model.name);
            trace!(
                "model[{}].mesh.material_id = {:?}",
                i,
                model.mesh.material_id
            );
            trace!(
                "model[{}].face_count       = {}",
                i,
                model.mesh.face_arities.len()
            );

            let mesh = self.load_mesh(&model.mesh, &materials, &model.name)?;
            self.meshes.push(mesh);
        }

        Ok(())
    }

    fn load_mesh(
        &mut self,
        mesh: &tobj::Mesh,
        materials: &[tobj::Material],
        model_name: &str,
    ) -> anyhow::Result<Mesh> {
        // Handle indices of mesh
        let indices: Vec<u32> = mesh.indices.clone();

        // Handle vertices of mesh
        assert!(mesh.positions.len() % 3 == 0);
        let vertices_num = mesh.positions.len() / 3;
        let mut vertices: Vec<Vertex> = Vec::with_capacity(vertices_num);
        for i in 0..vertices_num {
            vertices.push(Vertex {
                position: na::Vector3::new(
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                ),
                normal: na::Vector3::new(
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                ),
                texture_coords: na::Vector2::new(mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]),
            })
        }

        // Handle textures of mesh
        let mut diffuse_texture = None;
        let mut specular_texture = None;
        let mut normal_texture = None;
        if let Some(material_id) = mesh.material_id {
            let material = &materials[material_id];
            // load diffuse map
            if let Some(ref diffuse_texture_filename) = material.diffuse_texture {
                diffuse_texture =
                    Some(self.load_texture(diffuse_texture_filename, TextureType::Diffuse)?);
            } else {
                warn!("No diffuse texture for mesh in model({})!", model_name)
            }
            // load specular map
            if let Some(ref specular_texture_filename) = material.specular_texture {
                specular_texture =
                    Some(self.load_texture(specular_texture_filename, TextureType::Specular)?);
            } else {
                warn!("No specular texture for mesh in model({})!", model_name)
            }
            // load normal map
            if let Some(ref normal_texture_filename) = material.normal_texture {
                normal_texture =
                    Some(self.load_texture(normal_texture_filename, TextureType::Normal)?);
            } else {
                warn!("No normal texture for mesh in model({})!", model_name)
            }
            // TODO: load ambient & shiness map
        } else {
            bail!("No material id for mesh")
        }

        Ok(Mesh::new(
            vertices,
            indices,
            diffuse_texture,
            specular_texture,
            normal_texture,
        )?)
    }

    fn load_texture(
        &mut self,
        filename: &str,
        texture_type: TextureType,
    ) -> anyhow::Result<Texture> {
        let texture_path = self
            .model_path
            .parent()
            .unwrap_or(&PathBuf::new())
            .join(filename);

        // Check if texture was loaded before and if so, return it
        if let Some(texture_loaded) = self.textures_loaded.iter().find(|t| t.path == texture_path) {
            return Ok(texture_loaded.clone());
        }

        let texture = Texture::create(texture_path, Some(texture_type))?;
        self.textures_loaded.push(texture.clone());

        Ok(texture)
    }
}
