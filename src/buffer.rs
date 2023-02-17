use gl::types::*;

/// enum of Buffer Object types
/// TODO: complete all bindings
#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub buffer_type: BufferType,
}

impl Buffer {
    /// Try to create a Buffer Object struct.
    /// wrap `GenBuffers`
    pub fn new(buffer_type: BufferType) -> Option<Self> {
        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }

        if vbo == 0 {
            None
        } else {
            Some(Self {
                id: vbo,
                buffer_type,
            })
        }
    }

    /// Bind this Buffer Object to its buffer type
    /// wrap `glBindBuffer`
    pub fn bind(&self) {
        unsafe { gl::BindBuffer(self.buffer_type as GLenum, self.id) }
    }

    /// Clear Buffer Object binding for current Buffer Object's buffer type
    /// wrap `glBindBuffer`
    pub fn unbind(&self) {
        unsafe { gl::BindBuffer(self.buffer_type as GLenum, 0) }
    }

    /// Bind Buffer Object and Set its data
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

    /// wrap `glClearColor`
    pub fn set_clear_color(red: f32, green: f32, blue: f32, alpha: f32) {
        unsafe {
            gl::ClearColor(red, green, blue, alpha);
        }
    }
}

pub struct VertexBufferElement {
    pub ele_type: GLenum,
    pub count: GLint,
    pub should_normalized: GLboolean,
}

impl VertexBufferElement {
    pub fn get_type_size(&self) -> usize {
        match self.ele_type {
            gl::FLOAT => return std::mem::size_of::<f32>(),
            _ => {
                unimplemented!()
            }
        }
    }
}

pub struct VertexBufferLayout {
    elements: Vec<VertexBufferElement>,
    stride: GLsizei,
}

impl Default for VertexBufferLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl VertexBufferLayout {
    pub fn new() -> Self {
        VertexBufferLayout {
            elements: Vec::new(),
            stride: 0,
        }
    }

    pub fn push(&mut self, ele_type: GLenum, count: GLint) {
        match ele_type {
            gl::FLOAT => {
                let element = VertexBufferElement {
                    ele_type,
                    count,
                    should_normalized: gl::FALSE,
                };
                self.stride += count * element.get_type_size() as i32;
                self.elements.push(element);
            }
            _ => {
                unimplemented!()
            }
        }
    }

    pub fn get_elements(&self) -> &Vec<VertexBufferElement> {
        &self.elements
    }

    pub fn get_stride(&self) -> GLsizei {
        self.stride
    }
}
