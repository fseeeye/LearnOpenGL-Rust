use gl::types::*;

use crate::{get_gl_error, VertexArray, VertexDescription};

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
    pub fn set_buffer_data(&self, data: &[u8], usage: BufferUsage) {
        self.bind();

        unsafe {
            gl::BufferData(
                self.buffer_type as GLenum,
                data.len() as GLsizeiptr,
                data.as_ptr().cast(),
                usage as GLenum,
            );
        }
    }

    /// Set Vertex Attribute description for Vertex Buffer Object.
    /// This'll call `bind()` of VBO and VAO(if set) automatically.
    ///
    /// If type of this buffer is **not** `ARRAY_BUFFER`, it'll panic!
    ///
    /// wrap `glEnableVertexAttribArray` & `glVertexAttribPointer`.
    pub fn set_vertex_description(
        &mut self,
        desc: &VertexDescription,
        vao_opt: Option<&VertexArray>,
    ) {
        assert_eq!(self.buffer_type, BufferType::VertexBuffer);

        if let Some(vao) = vao_opt {
            vao.bind();
        }
        self.bind();

        let pointers = desc.get_attrib_pointers();
        let mut offset = 0_u32;

        for (index, element) in pointers.iter().enumerate() {
            unsafe {
                gl::VertexAttribPointer(
                    // attribute index
                    index as u32,
                    // attribute element size
                    element.count,
                    // attribute element type
                    element.ele_type,
                    // coordinate should be normalized or not
                    element.should_normalized,
                    // attribute size
                    desc.get_stride(),
                    // We have to convert the pointer location using usize values and then cast to a const pointer
                    // once we have our usize. We do not want to make a null pointer and then offset it with the `offset`
                    // method. That's gonna generate an out of bounds pointer, which is UB. We could try to remember to use the
                    // `wrapping_offset` method, or we could just do all the math in usize and then cast at the end.
                    // I prefer the latter option.
                    offset as *const _,
                );
                gl::EnableVertexAttribArray(index as u32);
            }

            offset += element.count as u32 * element.get_type_size() as u32;
        }
    }

    /// wrap `glClearColor`
    pub fn set_clear_color(red: f32, green: f32, blue: f32, alpha: f32) {
        unsafe {
            gl::ClearColor(red, green, blue, alpha);
        }
    }

    /// wrap `glClear`
    pub fn clear(bit_mast: GLbitfield) {
        unsafe { gl::Clear(bit_mast) }
    }
}
