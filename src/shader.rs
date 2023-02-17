use gl::types::*;

/// enum of Shader types
#[derive(Clone)]
pub enum ShaderType {
    Vertex = gl::VERTEX_SHADER as isize,
    Fragment = gl::FRAGMENT_SHADER as isize,
}

/// Wrapper of [Shader Object](https://www.khronos.org/opengl/wiki/GLSL_Object#Shader_objects)
#[derive(Clone)]
struct Shader {
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

    fn check_compile_result(&self) -> Result<(), String> {
        let mut is_success = gl::FALSE as GLint;
        unsafe { gl::GetShaderiv(self.id, gl::COMPILE_STATUS, &mut is_success) }

        if is_success == gl::FALSE as GLint {
            let mut log_cap = 0;
            unsafe { gl::GetShaderiv(self.id, gl::INFO_LOG_LENGTH, &mut log_cap) }
            let mut log_buf: Vec<u8> = Vec::with_capacity(log_cap as usize);

            let mut log_len = 0i32;
            unsafe {
                gl::GetShaderInfoLog(
                    self.id,
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

        self.check_compile_result()
    }

    /// Delete the shader
    /// wrap `glDeleteShader`
    pub fn delete(&mut self) {
        unsafe { gl::DeleteShader(self.id) }
    }

    /// Create/Attach/Link shader program from source
    pub(self) fn from_source(shader_type: ShaderType, src: &str) -> Result<Self, String> {
        let shader =
            Self::new(shader_type.clone()).ok_or("Unable to create Shader Object".to_string())?;
        shader.set_source(src);
        if let Err(msg) = shader.compile() {
            return Err(msg);
        }

        Ok(shader)
    }
}

/// Wrapper of [Program Object](https://www.khronos.org/opengl/wiki/GLSL_Object#Program_objects)
pub struct ShaderProgram {
    id: GLuint,
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

    /// Attach a Shader Object to this Program Object.
    /// wrap `glAttachShader`
    fn attach_shader(&self, shader: &Shader) {
        unsafe { gl::AttachShader(self.id, shader.id) };
    }

    fn check_link_result(&self) -> Result<(), String> {
        let mut is_success = 0;
        unsafe { gl::GetProgramiv(self.id, gl::LINK_STATUS, &mut is_success) }

        if is_success == 0 {
            let mut log_cap = 0;
            unsafe { gl::GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut log_cap) }

            let mut log_buf: Vec<u8> = Vec::with_capacity(log_cap as usize);

            let mut log_len = 0i32;
            unsafe {
                gl::GetProgramInfoLog(
                    self.id,
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

    /// Link all compiled&attached shader objects into a this program.
    /// wrap `glLinkProgram`
    fn link_program(&self) -> Result<(), String> {
        unsafe { gl::LinkProgram(self.id) };

        self.check_link_result()
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

    /// Create Program Object from vertex & fragment shader source
    pub fn from_vert_frag(vert: &str, frag: &str) -> Result<Self, String> {
        let program = Self::new().ok_or_else(|| "Couldn't allocate a program".to_string())?;

        // Create vertex & fragment shader
        let mut vertex_shader = Shader::from_source(ShaderType::Vertex, vert)
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        let mut frag_shader = Shader::from_source(ShaderType::Fragment, frag)
            .map_err(|e| format!("Fragment Compile Error: {}", e))?;

        // Attach vertex & fragment shader to program
        program.attach_shader(&vertex_shader);
        program.attach_shader(&frag_shader);

        // Link all attached shader stages into program
        let link_rst = program.link_program();

        // Delete shaders after link completed
        vertex_shader.delete();
        frag_shader.delete();

        match link_rst {
            Ok(_) => Ok(program),
            Err(msg) => Err(msg),
        }
    }
}
