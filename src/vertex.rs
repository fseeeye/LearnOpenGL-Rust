use gl::types::*;
use nalgebra as na;

use crate::{get_gl_error, Buffer, BufferType};

#[derive(Debug)]
#[repr(C)]
pub struct Vertex {
    pub position: na::Vector3<f32>,
    pub normal: na::Vector3<f32>,
    pub texture_coords: na::Vector2<f32>,
}

/// Wrapper of [Vertex Array Object](https://www.khronos.org/opengl/wiki/Vertex_Specification#Vertex_Array_Object)
pub struct VertexArray {
    pub id: GLuint,
}

impl VertexArray {
    /// Try to create a Vertex Array Object struct.
    /// wrap `glGenVertexArrays`
    pub fn new() -> anyhow::Result<Self> {
        let mut vao = 0;
        unsafe { gl::GenVertexArrays(1, &mut vao) }

        if vao == 0 {
            Err(get_gl_error().unwrap().into())
        } else {
            Ok(Self { id: vao })
        }
    }

    /// Bind this Vertex Array Object
    /// wrap `glBindVertexArray`
    pub fn bind(&self) {
        unsafe { gl::BindVertexArray(self.id) }
    }

    /// Clear Vertex Array Object binding.
    /// wrap `glBindVertexArray`
    pub fn clear_binding() {
        unsafe { gl::BindVertexArray(0) }
    }
}

/// TODO
pub struct VertexAttributePointer {
    pub ele_type: GLenum,
    pub count: GLint,
    pub should_normalized: GLboolean,
}

impl VertexAttributePointer {
    pub(crate) fn get_type_size(&self) -> usize {
        match self.ele_type {
            gl::FLOAT => std::mem::size_of::<f32>(),
            _ => {
                unimplemented!()
            }
        }
    }
}

/// TODO
pub struct VertexDescription {
    pointers: Vec<VertexAttributePointer>,
    stride: GLsizei,
}

impl Default for VertexDescription {
    fn default() -> Self {
        Self::new()
    }
}

impl VertexDescription {
    pub fn new() -> Self {
        VertexDescription {
            pointers: Vec::new(),
            stride: 0,
        }
    }

    pub fn add_attribute(&mut self, ele_type: GLenum, count: GLint) {
        match ele_type {
            gl::FLOAT => {
                let desc = VertexAttributePointer {
                    ele_type,
                    count,
                    should_normalized: gl::FALSE,
                };
                self.stride += count * desc.get_type_size() as i32;
                self.pointers.push(desc);
            }
            _ => {
                unimplemented!()
            }
        }
    }

    /// Set Vertex Attribute description for Vertex Buffer Object.
    /// This'll call `bind()` of VBO and VAO(if set) automatically.
    ///
    /// If type of this buffer is **not** `ARRAY_BUFFER`, it'll panic!
    ///
    /// wrap `glEnableVertexAttribArray` & `glVertexAttribPointer`.
    pub fn bind_to(&mut self, vbo: &Buffer, vao_opt: Option<&VertexArray>) {
        assert_eq!(vbo.buffer_type, BufferType::VertexBuffer);

        // Bind VAO & VBO
        if let Some(vao) = vao_opt {
            vao.bind();
        }
        vbo.bind();

        // Create & Enable attribute pointers
        let mut offset = 0_u32;
        for (index, element) in self.pointers.iter().enumerate() {
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
                    self.stride,
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
}
