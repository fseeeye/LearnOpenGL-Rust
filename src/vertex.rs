use gl::types::*;

use crate::get_gl_error;

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

    pub fn push(&mut self, ele_type: GLenum, count: GLint) {
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

    pub(crate) fn get_attrib_pointers(&self) -> &Vec<VertexAttributePointer> {
        &self.pointers
    }

    pub(crate) fn get_stride(&self) -> GLsizei {
        self.stride
    }
}
