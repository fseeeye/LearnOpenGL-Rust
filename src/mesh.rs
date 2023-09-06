use std::ffi::CString;

use anyhow::bail;
use gl::types::GLsizei;

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
