use std::path::PathBuf;

use gl::types::*;
use image::GenericImageView;

use crate::get_gl_error;

/// Wrapper of [Texture Object](https://www.khronos.org/opengl/wiki/Texture)
#[derive(Debug, Clone)]
pub struct Texture {
    pub id: GLuint,
    pub tex_type: TextureType,
    pub path: PathBuf,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextureType {
    Diffuse,
    Specular,
    Normal,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
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

impl TextureUnit {
    pub fn increase(&self) -> TextureUnit {
        let val: GLint = (*self).into();
        (val + 1).into()
    }
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

impl From<GLint> for TextureUnit {
    fn from(val: GLint) -> Self {
        match val {
            0 => TextureUnit::TEXTURE0,
            1 => TextureUnit::TEXTURE1,
            2 => TextureUnit::TEXTURE2,
            3 => TextureUnit::TEXTURE3,
            4 => TextureUnit::TEXTURE4,
            5 => TextureUnit::TEXTURE5,
            6 => TextureUnit::TEXTURE6,
            7 => TextureUnit::TEXTURE7,
            8 => TextureUnit::TEXTURE8,
            9 => TextureUnit::TEXTURE9,
            10 => TextureUnit::TEXTURE10,
            11 => TextureUnit::TEXTURE11,
            12 => TextureUnit::TEXTURE12,
            13 => TextureUnit::TEXTURE13,
            14 => TextureUnit::TEXTURE14,
            15 => TextureUnit::TEXTURE15,
            _ => panic!("Invalid TextureUnit value."),
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

impl Texture {
    fn new(path: PathBuf, texture_type: TextureType) -> anyhow::Result<Self> {
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
        }

        if texture != 0 {
            Ok(Self {
                id: texture,
                tex_type: texture_type,
                path,
            })
        } else {
            Err(get_gl_error().unwrap().into())
        }
    }

    /// Create Texture
    pub fn create(path: PathBuf, texture_type: Option<TextureType>) -> anyhow::Result<Self> {
        // Generate Texture
        let tex_type: TextureType = match texture_type {
            Some(t) => t,
            None => TextureType::Unknown,
        };
        let texture = Self::new(path, tex_type)?;

        // Bind Texture
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, texture.id); // Bind Texture
        }

        // Load Texture image
        let img = image::open(&texture.path).unwrap().flipv();
        let (width, height) = img.dimensions();
        let img_format: GLenum;
        let img_type: GLenum;
        match img.color() {
            image::ColorType::Rgb8 => {
                img_format = gl::RGB;
                img_type = gl::UNSIGNED_BYTE;
            }
            image::ColorType::Rgba8 => {
                img_format = gl::RGBA;
                img_type = gl::UNSIGNED_BYTE;
            }
            _ => {
                anyhow::bail!("Unsupported image color type: {:?}", img.color())
            }
        }

        // Set Texture wrapping & filtering
        unsafe {
            if img_format == gl::RGBA {
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_WRAP_S,
                    gl::CLAMP_TO_EDGE as GLint,
                );
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_WRAP_T,
                    gl::CLAMP_TO_EDGE as GLint,
                );
            } else {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            }
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        }

        // Send Texture image data
        let pixels = img.into_bytes();
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                img_format as GLint,
                width.try_into()?,
                height.try_into()?,
                0,
                img_format,
                img_type,
                pixels.as_ptr().cast(),
            );
        }

        // Generate mipmap
        unsafe { gl::GenerateMipmap(gl::TEXTURE_2D) }

        Ok(texture)
    }

    /// Active texture unit/slot and Bind this Texture Object to it.
    pub fn bind(&self, unit: TextureUnit) {
        // Active Texture unit
        Self::active(unit);

        // Bind Texture
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.id) }
    }

    pub fn active(unit: TextureUnit) {
        unsafe {
            gl::ActiveTexture(unit.into());
        }
    }
}
