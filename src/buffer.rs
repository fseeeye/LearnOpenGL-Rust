use gl::types::*;
use std::mem;

use crate::{get_gl_error, Vertex};

/// Enum of Buffer Object types.
/// TODO: complete all bindings
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BufferType {
    /// Vertex Buffer Object, include Vertex attributes
    VertexBuffer = gl::ARRAY_BUFFER as isize,
    /// Index/Element Buffer Object, include Vertex array indices
    IndexBuffer = gl::ELEMENT_ARRAY_BUFFER as isize,
}

/// Enum of Buffer Bit for `glClear()`.
/// TODO: complete all bindings
#[derive(Debug, Clone, Copy)]
pub enum BufferBit {
    /// Indicates the buffers currently enabled for color writing.
    ColorBufferBit = gl::COLOR_BUFFER_BIT as isize,
    /// Indicates the depth buffer.
    DepthBufferBit = gl::DEPTH_BUFFER_BIT as isize,
    /// Indicates the stencil buffer.
    StencilBufferBit = gl::STENCIL_BUFFER_BIT as isize,
}

/// Enum of Buffer Object usage.
/// TODO: complete all bindings
#[derive(Debug, Clone, Copy)]
pub enum BufferUsage {
    // STATIC : The data store contents will be modified once and used many times.
    // DRAW   : The data store contents are modified by the application, and used as the source for GL drawing and
    //          image specification commands.
    StaticDraw = gl::STATIC_DRAW as isize,
}

/// Wrapper of [Buffer Object](https://www.khronos.org/opengl/wiki/Buffer_Object)
pub struct Buffer {
    pub id: GLuint,
    pub buffer_type: BufferType,
}

impl Buffer {
    /// Try to create a Buffer Object struct.
    ///
    /// wrap `GenBuffers`
    pub fn new(buffer_type: BufferType) -> anyhow::Result<Self> {
        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }

        if vbo == 0 {
            Err(get_gl_error().unwrap().into())
        } else {
            Ok(Self {
                id: vbo,
                buffer_type,
            })
        }
    }

    /// Bind this Buffer Object to its buffer type.
    ///
    /// wrap `glBindBuffer`
    pub fn bind(&self) {
        unsafe { gl::BindBuffer(self.buffer_type as GLenum, self.id) }
    }

    /// Clear Buffer Object binding for current Buffer Object's buffer type.
    ///
    /// wrap `glBindBuffer`
    pub fn unbind(&self) {
        unsafe { gl::BindBuffer(self.buffer_type as GLenum, 0) }
    }

    /// Set Buffer Object data, it'll call `bind()` automatically.
    ///
    /// wrap `glBufferData`
    pub fn set_buffer_data<T>(&self, data: &[T], usage: BufferUsage) {
        self.bind();

        unsafe {
            gl::BufferData(
                self.buffer_type as GLenum,
                mem::size_of_val(data) as GLsizeiptr,
                data.as_ptr().cast(),
                usage as GLenum,
            );
        }
    }

    /// Set indices to IndexBuffer, it'll call `bind()` automatically.
    ///
    /// wrap `glBufferData`
    pub fn set_indices(&self, indices: &[u32], usage: BufferUsage) {
        assert_eq!(self.buffer_type, BufferType::IndexBuffer);

        self.set_buffer_data(indices, usage);
    }

    /// Set vertices to VertexBuffer, it'll call `bind()` automatically.
    ///
    /// wrap `glBufferData`
    pub fn set_vertices(&self, vertices: &[Vertex], usage: BufferUsage) {
        assert_eq!(self.buffer_type, BufferType::VertexBuffer);

        self.set_buffer_data(vertices, usage);
    }
}

/// wrap `glClearColor`
#[inline]
pub fn set_clear_color(red: f32, green: f32, blue: f32, alpha: f32) {
    unsafe {
        gl::ClearColor(red, green, blue, alpha);
    }
}

/// wrap `glClear`
#[inline]
pub fn clear_color(bit_mast: GLbitfield) {
    unsafe { gl::Clear(bit_mast) }
}
