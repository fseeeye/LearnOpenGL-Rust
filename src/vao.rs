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
}
