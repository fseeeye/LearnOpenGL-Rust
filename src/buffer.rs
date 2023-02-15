use gl::types::*;

pub fn set_clear_color(r: f32, g: f32, b: f32, a: f32) {
    unsafe {
        gl::ClearColor(r, g, b, a);
    }
}

/// enum of Buffer Object types
/// TODO: complete all bindings
#[derive(Clone)]
pub enum BufferType {
    /// Vertex attributes
    Array = gl::ARRAY_BUFFER as isize,
    /// Vertex array indices
    ElementArray = gl::ELEMENT_ARRAY_BUFFER as isize,
}

/// enum of Buffer Object usage
/// TODO: complete all bindings
pub enum BufferUsage {
    StaticDraw = gl::STATIC_DRAW as isize,
}

/// Wrapper of [Buffer Object](https://www.khronos.org/opengl/wiki/Buffer_Object)
pub struct Buffer {
    pub id: GLuint,
}

impl Buffer {
    /// Try to create a Buffer Object struct.
    /// wrap `GenBuffers`
    pub fn new() -> Option<Self> {
        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }

        if vbo == 0 {
            None
        } else {
            Some(Self { id: vbo })
        }
    }

    /// Bind this Buffer Object as spec buffer type
    /// wrap `glBindBuffer`
    pub fn bind(&self, buffer_type: BufferType) {
        unsafe { gl::BindBuffer(buffer_type as GLenum, self.id) }
    }

    /// Clear Buffer Object binding for spec buffer type
    /// wrap `glBindBuffer`
    pub fn clear_binding(buffer_type: BufferType) {
        unsafe { gl::BindBuffer(buffer_type as GLenum, 0) }
    }

    /// Bind Buffer Object and Set its data
    /// wrap `glBufferData`
    pub fn set_buffer_data(&self, data: &[u8], buffer_type: BufferType, usage: BufferUsage) {
        self.bind(buffer_type.clone());

        unsafe {
            gl::BufferData(
                buffer_type as GLenum,
                data.len() as GLsizeiptr,
                data.as_ptr().cast(),
                usage as GLenum,
            );
        }
    }
}
