extern crate stb_image;

use gl;
#[allow(unused_imports)]
use stb_image::image::LoadResult;
#[allow(unused_imports)]
use stb_image::stb_image::bindgen::stbi_load_from_file;

type Path = std::ffi::OsStr;

static mut MAX_TEXTURE_FILTERING: f32 = 0.;

#[derive(Copy, Clone)]
pub enum TextureKind {
    Diffuse,
    Specular,
    Normal,
    Height,
}

impl TextureKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TextureKind::Diffuse => "texture_diffuse",
            TextureKind::Specular => "texture_specular",
            TextureKind::Normal => "texture_normal",
            TextureKind::Height => "texture_height",
        }
    }
}

pub struct Texture {
    gl: gl::GlPtr,
    pub id: u32,
}

impl Texture {
    pub fn init(gl: &gl::GlPtr) {
        unsafe {
            gl.GetFloatv(
                gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT,
                &mut MAX_TEXTURE_FILTERING,
            );
        }
    }

    pub fn new(gl: &gl::GlPtr, width: u32, height: u32) -> Texture {
        let mut texture_id: u32 = 0;

        unsafe {
            gl.GenTextures(1, &mut texture_id);
            gl.BindTexture(gl::TEXTURE_2D, texture_id);
            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as gl::types::GLint,
                width as gl::types::GLint,
                height as gl::types::GLint,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                std::ptr::null() as *const gl::types::GLvoid,
            );

            gl.TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as gl::types::GLint,
            );
            gl.TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as gl::types::GLint,
            );

            gl.BindTexture(gl::TEXTURE_2D, 0);
        }

        Texture {
            gl: gl.clone(),
            id: texture_id,
        }
    }

    pub fn from<P>(gl: &gl::GlPtr, path: P) -> Option<Texture>
    where
        P: AsRef<Path> + std::convert::AsRef<std::path::Path>,
    {
        let texture_load_result = stb_image::image::load(path);
        let mut texture_id: u32 = 0;

        match texture_load_result {
            LoadResult::Error(_) => None,
            LoadResult::ImageU8(image) => {
                let texture_image = image;
                let mode = match &texture_image.depth {
                    3 => gl::RGB,
                    4 => gl::RGBA,
                    _ => gl::RGB,
                };

                unsafe {
                    gl.GenTextures(1, &mut texture_id);
                    gl.BindTexture(gl::TEXTURE_2D, texture_id);
                    gl.TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        mode as i32,
                        texture_image.width as i32,
                        texture_image.height as i32,
                        0,
                        mode,
                        gl::UNSIGNED_BYTE,
                        texture_image.data.as_ptr() as *const gl::types::GLvoid,
                    );
                    gl.GenerateMipmap(gl::TEXTURE_2D);

                    gl.BindTexture(gl::TEXTURE_2D, 0);
                }

                Some(Texture {
                    gl: gl.clone(),
                    id: texture_id,
                })
            }
            LoadResult::ImageF32(_) => None,
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindTexture(gl::TEXTURE_2D, self.id);

            // Parameters to override
            self.gl.TexParameterf(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAX_ANISOTROPY_EXT,
                MAX_TEXTURE_FILTERING,
            );
            //
            self.gl
                .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            self.gl
                .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

            // self.gl.TexParameteri(
            //     gl::TEXTURE_2D,
            //     gl::TEXTURE_MIN_FILTER,
            //     gl::LINEAR_MIPMAP_LINEAR as i32,
            // );
            // self.gl
            //     .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
    }
}
