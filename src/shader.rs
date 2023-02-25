use std::ffi::CStr;

use gl::types::*;

/// enum of Shader types
#[derive(Clone)]
pub enum ShaderType {
    Vertex = gl::VERTEX_SHADER as isize,
    Fragment = gl::FRAGMENT_SHADER as isize,
}

/// Wrapper of [Shader Object](https://www.khronos.org/opengl/wiki/GLSL_Object#Shader_objects)
#[derive(Clone)]
pub struct Shader {
    id: GLuint,
}

impl Shader {
    /// Makes a new Shader.
    /// wrap `glCreateShader`.
    fn new(shader_type: ShaderType) -> Option<Self> {
        let shader = unsafe { gl::CreateShader(shader_type as GLenum) };
        if shader != 0 {
            Some(Self { id: shader })
        } else {
            None
        }
    }

    /// Set source of Shader.
    /// wrap `glShaderSource`.
    fn set_source(&self, src: &str) {
        unsafe {
            gl::ShaderSource(
                self.id,
                1,
                &(src.as_bytes().as_ptr().cast()),
                &(src.len() as GLint),
            );
        }
    }

    /// Check Shader Object compiling result
    pub fn check_compile_result(shader_id: u32) -> Result<(), String> {
        let mut is_success = gl::FALSE as GLint;
        unsafe { gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut is_success) }

        if is_success == gl::FALSE as GLint {
            let mut log_cap = 0;
            unsafe { gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut log_cap) }
            let mut log_buf: Vec<u8> = Vec::with_capacity(log_cap as usize);

            let mut log_len = 0i32;
            unsafe {
                gl::GetShaderInfoLog(
                    shader_id,
                    log_buf.capacity() as i32,
                    &mut log_len,
                    log_buf.as_mut_ptr() as *mut GLchar,
                );
                log_buf.set_len(log_len as usize);
            }

            Err(String::from_utf8_lossy(&log_buf).into_owned())
        } else {
            Ok(())
        }
    }

    /// Compiles the shader after setting source and Check compiling result
    /// wrap `glCompileShader`
    fn compile(&self) -> Result<(), String> {
        unsafe { gl::CompileShader(self.id) }

        Self::check_compile_result(self.id)
    }

    /// Calling this method forces the destructor to be called, destroying the shader.
    /// wrap `glDeleteShader`
    pub fn delete(self) {}

    /// Create/Attach/Link shader program from source
    pub fn from_source(shader_type: ShaderType, src: &str) -> Result<Self, String> {
        let shader = Self::new(shader_type).ok_or("Unable to create Shader Object".to_string())?;
        shader.set_source(src);
        shader.compile()?;

        Ok(shader)
    }

    /// Create/Attach/Link shader program from shader file
    pub fn from_file(shader_type: ShaderType, path: &str) -> Result<Self, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read shader file: {e}"))?;

        Self::from_source(shader_type, &source)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id) }
    }
}

/// Wrapper of [Program Object](https://www.khronos.org/opengl/wiki/GLSL_Object#Program_objects)
pub struct ShaderProgram {
    pub id: GLuint,
}

impl ShaderProgram {
    /// Create a new Program Object
    /// wrap `CreateProgram`
    fn new() -> Option<Self> {
        let program = unsafe { gl::CreateProgram() };
        if program != 0 {
            Some(Self { id: program })
        } else {
            None
        }
    }

    /// Create Shader Program from vertex & fragment Shader Objects.
    /// This calling will consume Shader Objects.
    pub fn create(vert_shader: Shader, frag_shader: Shader) -> Result<Self, String> {
        let program = Self::new().ok_or_else(|| "Couldn't allocate a program".to_string())?;

        // Attach vertex & fragment shader to program
        program.attach_shader(&vert_shader);
        program.attach_shader(&frag_shader);

        // Link all attached shader stages into program
        let link_rst = program.link_program();

        // Delete shaders after link completed
        // tip: Of course, this operation does not need to be written out because it will be done on destructing. But
        //      for learning, I think it's necessary to write it out.
        vert_shader.delete();
        frag_shader.delete();

        match link_rst {
            Ok(_) => Ok(program),
            Err(msg) => Err(msg),
        }
    }

    /// Create Program Object from vertex & fragment shader source
    /// tip: you can use `include_str!` to embed small shader file content.
    pub fn create_from_source(vert: &str, frag: &str) -> Result<Self, String> {
        // Create vertex & fragment shader
        let vert_shader = Shader::from_source(ShaderType::Vertex, vert)
            .map_err(|e| format!("Vertex Compile Error: {e}"))?;
        let frag_shader = Shader::from_source(ShaderType::Fragment, frag)
            .map_err(|e| format!("Fragment Compile Error: {e}"))?;

        Self::create(vert_shader, frag_shader)
    }

    /// Create Program Object from vertex & fragment shader file
    pub fn create_from_file(vert_path: &str, frag_path: &str) -> Result<Self, String> {
        // Create vertex & fragment shader
        let vert_shader = Shader::from_file(ShaderType::Vertex, vert_path)
            .map_err(|e| format!("Vertex Compile Error: {e}"))?;
        let frag_shader = Shader::from_file(ShaderType::Fragment, frag_path)
            .map_err(|e| format!("Fragment Compile Error: {e}"))?;

        Self::create(vert_shader, frag_shader)
    }

    /// Check Shader Program linking result
    pub fn check_link_result(program_id: u32) -> Result<(), String> {
        let mut is_success = 0;
        unsafe { gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut is_success) }

        if is_success == 0 {
            let mut log_cap = 0;
            unsafe { gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut log_cap) }

            let mut log_buf: Vec<u8> = Vec::with_capacity(log_cap as usize);

            let mut log_len = 0i32;
            unsafe {
                gl::GetProgramInfoLog(
                    program_id,
                    log_buf.capacity() as i32,
                    &mut log_len,
                    log_buf.as_mut_ptr() as *mut GLchar,
                );
                log_buf.set_len(log_len as usize);
            }

            Err(String::from_utf8_lossy(&log_buf).into_owned())
        } else {
            Ok(())
        }
    }

    /// Attach a Shader Object to this Program Object.
    /// wrap `glAttachShader`
    fn attach_shader(&self, shader: &Shader) {
        unsafe { gl::AttachShader(self.id, shader.id) };
    }

    /// Link all compiled&attached shader objects into a this program.
    /// wrap `glLinkProgram`
    fn link_program(&self) -> Result<(), String> {
        unsafe { gl::LinkProgram(self.id) };

        Self::check_link_result(self.id)
    }

    /// Sets the program as the program to use when drawing.
    /// wrap `glUseProgram`
    pub fn bind(&self) {
        unsafe { gl::UseProgram(self.id) };
    }

    /// Marks the program for deletion.
    /// wrap `glDeleteProgram`.
    ///
    /// Tip: `glDeleteProgram` _does not_ immediately delete the program. If the program is
    /// currently in use it won't be deleted until it's not the active program.
    /// When a program is finally deleted and attached shaders are unattached.
    pub fn close(self) {
        unsafe { gl::DeleteProgram(self.id) };
    }

    /// wrap `glGetUniformLocation`
    pub fn get_uniform_location(&self, uniform_name: &CStr) -> i32 {
        unsafe { gl::GetUniformLocation(self.id, uniform_name.as_ptr()) }
    }

    /// Send uniform data, it'll call `bind()` automatically.
    /// wrap `glUniform*`
    pub fn set_uniform_4f(&self, uniform_location: i32, v0: f32, v1: f32, v2: f32, v3: f32) {
        self.bind();
        unsafe { gl::Uniform4f(uniform_location, v0, v1, v2, v3) }
    }
}
