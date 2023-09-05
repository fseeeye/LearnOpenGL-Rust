use anyhow::Ok;

use crate::{Buffer, BufferType, BufferUsage, Texture, Vertex, VertexArray, VertexDescription};

#[allow(dead_code)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub texture: Vec<Texture>,
    vao: VertexArray,
    vbo: Buffer,
    ibo: Buffer,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        texture: Vec<Texture>,
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

        Ok(Self {
            vertices,
            indices,
            texture,
            vao,
            vbo,
            ibo,
        })
    }
}
