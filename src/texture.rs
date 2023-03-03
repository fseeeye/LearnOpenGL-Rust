use anyhow::bail;
use gl::types::*;
use image::GenericImageView;

use crate::{get_gl_error, ShaderProgram};

/// Wrapper of [Texture Object](https://www.khronos.org/opengl/wiki/Texture)
pub struct Texture {
    pub id: GLuint,
    unit: TextureUnit,
}

#[derive(Debug, Clone, Copy)]
pub enum TextureUnit {
    TEXTURE0,
    TEXTURE1,
    TEXTURE2,
    TEXTURE3,
    TEXTURE4,
    TEXTURE5,
    TEXTURE6,
    TEXTURE7,
    TEXTURE8,
    TEXTURE9,
    TEXTURE10,
    TEXTURE11,
    TEXTURE12,
    TEXTURE13,
    TEXTURE14,
    TEXTURE15,
}

impl From<TextureUnit> for GLint {
    fn from(val: TextureUnit) -> Self {
        match val {
            TextureUnit::TEXTURE0 => 0,
            TextureUnit::TEXTURE1 => 1,
            TextureUnit::TEXTURE2 => 2,
            TextureUnit::TEXTURE3 => 3,
            TextureUnit::TEXTURE4 => 4,
            TextureUnit::TEXTURE5 => 5,
            TextureUnit::TEXTURE6 => 6,
            TextureUnit::TEXTURE7 => 7,
            TextureUnit::TEXTURE8 => 8,
            TextureUnit::TEXTURE9 => 9,
            TextureUnit::TEXTURE10 => 10,
            TextureUnit::TEXTURE11 => 11,
            TextureUnit::TEXTURE12 => 12,
            TextureUnit::TEXTURE13 => 13,
            TextureUnit::TEXTURE14 => 14,
            TextureUnit::TEXTURE15 => 15,
        }
    }
}

impl From<TextureUnit> for GLenum {
    fn from(val: TextureUnit) -> Self {
        match val {
            TextureUnit::TEXTURE0 => gl::TEXTURE0,
            TextureUnit::TEXTURE1 => gl::TEXTURE1,
            TextureUnit::TEXTURE2 => gl::TEXTURE2,
            TextureUnit::TEXTURE3 => gl::TEXTURE3,
            TextureUnit::TEXTURE4 => gl::TEXTURE4,
            TextureUnit::TEXTURE5 => gl::TEXTURE5,
            TextureUnit::TEXTURE6 => gl::TEXTURE6,
            TextureUnit::TEXTURE7 => gl::TEXTURE7,
            TextureUnit::TEXTURE8 => gl::TEXTURE8,
            TextureUnit::TEXTURE9 => gl::TEXTURE9,
            TextureUnit::TEXTURE10 => gl::TEXTURE10,
            TextureUnit::TEXTURE11 => gl::TEXTURE11,
            TextureUnit::TEXTURE12 => gl::TEXTURE12,
            TextureUnit::TEXTURE13 => gl::TEXTURE13,
            TextureUnit::TEXTURE14 => gl::TEXTURE14,
            TextureUnit::TEXTURE15 => gl::TEXTURE15,
        }
    }
}

// TODO: complete
#[derive(Debug, Clone, Copy)]
pub enum TextureFormat {
    RGB = gl::RGB as isize,
    RGBA = gl::RGBA as isize,
}

impl Texture {
    fn new(texture_unit: TextureUnit) -> anyhow::Result<Self> {
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
        }

        if texture != 0 {
            Ok(Self {
                id: texture,
                unit: texture_unit,
            })
        } else {
            Err(get_gl_error().unwrap().into())
        }
    }

    /// Create Texture
    pub fn create(
        path: &str,
        format: TextureFormat,
        texture_unit: TextureUnit,
    ) -> anyhow::Result<Self> {
        // Generate Texture
        let texture = Self::new(texture_unit)?;

        // Bind Texture
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, texture.id); // Bind Texture
        }

        // Set Texture wrapping & filtering
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        }

        // Load Texture image
        let img = image::open(path).unwrap().flipv();
        let (width, height) = img.dimensions();
        let pixels = img.into_bytes();
        // Send Texture image data
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                format as GLint,
                width.try_into()?,
                height.try_into()?,
                0,
                format as GLenum,
                gl::UNSIGNED_BYTE,
                pixels.as_ptr().cast(),
            );
        }
        // Generate mipmap
        unsafe { gl::GenerateMipmap(gl::TEXTURE_2D) }

        Ok(texture)
    }

    pub fn reset_texture_unit(&mut self, texture_unit: TextureUnit) {
        self.unit = texture_unit;
    }

    #[allow(dead_code)]
    fn texture_unit_to_int(texture_unit: GLenum) -> anyhow::Result<GLint> {
        match texture_unit {
            gl::TEXTURE0 => Ok(0),
            gl::TEXTURE1 => Ok(1),
            gl::TEXTURE2 => Ok(2),
            gl::TEXTURE3 => Ok(3),
            gl::TEXTURE4 => Ok(4),
            gl::TEXTURE5 => Ok(5),
            gl::TEXTURE6 => Ok(6),
            gl::TEXTURE7 => Ok(7),
            gl::TEXTURE8 => Ok(8),
            gl::TEXTURE9 => Ok(9),
            gl::TEXTURE10 => Ok(10),
            gl::TEXTURE11 => Ok(11),
            gl::TEXTURE12 => Ok(12),
            gl::TEXTURE13 => Ok(13),
            gl::TEXTURE14 => Ok(14),
            gl::TEXTURE15 => Ok(15),
            _ => bail!("Texture unit is longer than 15 or unknow: {}", texture_unit),
        }
    }

    /// Bind Texture unit/slot to spec uniform sampler of spec shader program.
    ///
    /// wrap `glUniform1i`
    pub fn bind_texture_unit(&self, uniform_name: &str, shader_program: &ShaderProgram) {
        // Bind shader program before setting uniforms
        shader_program.bind();

        // Bind sampler uniform var to spec texture unit
        unsafe {
            gl::Uniform1i(
                gl::GetUniformLocation(shader_program.id, uniform_name.as_ptr().cast()),
                self.unit.into(),
            ); // unnecessary for TEXTURE 0
        }
    }

    /// Active texture unit/slot and Bind this Texture Object to it.
    pub fn active(&self) {
        // Active Texture unit
        unsafe {
            gl::ActiveTexture(self.unit.into()) // unnecessary for TEXTURE 0
        }

        // Bind Texture
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.id) }
    }
}
