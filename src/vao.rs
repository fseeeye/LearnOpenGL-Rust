use crate::buffer::{Buffer, BufferType, VertexBufferLayout};

use gl::types::*;

/// Wrapper of [Vertex Array Object](https://www.khronos.org/opengl/wiki/Vertex_Specification#Vertex_Array_Object)
pub struct VertexArray {
    pub id: GLuint,
}

impl VertexArray {
    /// Try to create a Vertex Array Object struct.
    /// wrap `glGenVertexArrays`
    pub fn new() -> Option<Self> {
        let mut vao = 0;
        unsafe { gl::GenVertexArrays(1, &mut vao) }

        if vao == 0 {
            None
        } else {
            Some(Self { id: vao })
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

    pub fn add_vertex_buffer(&mut self, buffer: &Buffer, buffer_layout: &VertexBufferLayout) {
        assert_eq!(buffer.buffer_type, BufferType::Array);

        self.bind();
        buffer.bind();

        let elements = buffer_layout.get_elements();
        let mut offset = 0_u32;

        for (index, element) in elements.iter().enumerate() {
            unsafe {
                gl::EnableVertexAttribArray(index as u32);
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
                    buffer_layout.get_stride(),
                    // We have to convert the pointer location using usize values and then cast to a const pointer
                    // once we have our usize. We do not want to make a null pointer and then offset it with the `offset`
                    // method. That's gonna generate an out of bounds pointer, which is UB. We could try to remember to use the
                    // `wrapping_offset` method, or we could just do all the math in usize and then cast at the end.
                    // I prefer the latter option.
                    offset as *const _,
                );
            }

            offset += element.count as u32 * element.get_type_size() as u32;
        }
    }
}
