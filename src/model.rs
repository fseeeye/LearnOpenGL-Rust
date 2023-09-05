use std::ffi::CString;

use anyhow::Ok;
use gl::types::*;

use crate::{
    Buffer, BufferType, BufferUsage, ShaderProgram, Texture, TextureType, TextureUnit, Vertex,
    VertexArray, VertexDescription,
};

#[allow(dead_code)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Texture>,
    vao: VertexArray,
    vbo: Buffer,
    ibo: Buffer,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        textures: Vec<Texture>,
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

        Ok(Self {
            vertices,
            indices,
            textures,
            vao,
            vbo,
            ibo,
        })
    }

    pub fn draw(&self, shader: &ShaderProgram, uniform_name: &str) -> anyhow::Result<()> {
        let mut diffuse_num = 0;
        let mut specular_num = 0;
        let mut normal_num = 0;
        let mut height_num = 0;

        // Bind textures
        let mut texture_unit = TextureUnit::TEXTURE0;
        for texture in self.textures.iter() {
            let texture_name = match texture.tex_type {
                TextureType::Diffuse => {
                    let name =
                        CString::new(format!("{uniform_name}.texture_diffuse{diffuse_num}"))?;
                    diffuse_num += 1;
                    name
                }
                TextureType::Specular => {
                    let name =
                        CString::new(format!("{uniform_name}.texture_specular{specular_num}"))?;
                    specular_num += 1;
                    name
                }
                TextureType::Normal => {
                    let name =
                        CString::new(format!("{uniform_name}.texture_normal{normal_num}"))?;
                    normal_num += 1;
                    name
                }
                TextureType::Height => {
                    let name =
                        CString::new(format!("{uniform_name}.texture_height{height_num}"))?;
                    height_num += 1;
                    name
                }
                TextureType::Unknown => panic!("Unknown texture type"),
            };

            shader.set_texture_unit(&texture_name, texture, texture_unit);
            texture_unit = texture_unit.increase();
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
}
