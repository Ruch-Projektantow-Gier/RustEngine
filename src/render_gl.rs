use gl;
use std;
use std::ffi::{CStr, CString};

type Stringable = Into<Vec<u8>>;

pub struct Shader {
    gl: gl::GlPtr,
    id: gl::types::GLuint,
}

impl Shader {
    pub fn from_source(
        gl: &gl::GlPtr,
        source: &CStr,
        kind: gl::types::GLenum,
    ) -> Result<Shader, String> {
        let id = shader_from_source(gl, source, kind)?;
        Ok(Shader { gl: gl.clone(), id })
    }

    pub fn from_vert_source(gl: &gl::GlPtr, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(gl: &gl::GlPtr, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, gl::FRAGMENT_SHADER)
    }

    pub fn from_comp_source(gl: &gl::GlPtr, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, gl::COMPUTE_SHADER)
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
    }
}

pub struct Program {
    gl: gl::GlPtr,
    id: gl::types::GLuint,
}

impl Program {
    pub fn from_shaders(gl: &gl::GlPtr, shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl.CreateProgram() };

        for shader in shaders {
            unsafe { gl.AttachShader(program_id, shader.id()) }
        }

        unsafe { gl.LinkProgram(program_id) }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl.GetShaderiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);
            unsafe {
                gl.GetShaderInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                )
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe { gl.DetachShader(program_id, shader.id()) }
        }

        Ok(Program {
            gl: gl.clone(),
            id: program_id,
        })
    }

    pub fn from_files(
        gl: &gl::GlPtr,
        vert_path: &'static str,
        frag_path: &'static str,
    ) -> Result<Program, String> {
        use std::ffi::CString;

        let vert_shader = Shader::from_vert_source(&gl, &CString::new(vert_path).unwrap())?;
        let frag_shader = Shader::from_frag_source(&gl, &CString::new(frag_path).unwrap())?;

        Self::from_shaders(&gl, &[vert_shader, frag_shader])
    }

    pub fn from_compute_shader_file(gl: &gl::GlPtr, path: &'static str) -> Result<Program, String> {
        use std::ffi::CString;
        let comp_shader = Shader::from_comp_source(&gl, &CString::new(path).unwrap())?;
        Self::from_shaders(&gl, &[comp_shader])
    }

    pub fn bind(&self) {
        unsafe { self.gl.UseProgram(self.id) }
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn setMat4(&self, val: &glm::Mat4, name: &'static str) {
        let cstr_name = CString::new(name).unwrap();

        unsafe {
            self.gl.UniformMatrix4fv(
                self.gl.GetUniformLocation(self.id, cstr_name.as_ptr()),
                1,
                gl::FALSE,
                val.as_ptr(),
            );
        }
    }

    pub fn setFloat(&self, val: f32, name: &str) {
        let cstr_name = CString::new(name).unwrap();

        unsafe {
            self.gl
                .Uniform1f(self.gl.GetUniformLocation(self.id, cstr_name.as_ptr()), val);
        }
    }

    pub fn setInt(&self, val: i32, name: &str) {
        let cstr_name = CString::new(name).unwrap();

        unsafe {
            self.gl
                .Uniform1i(self.gl.GetUniformLocation(self.id, cstr_name.as_ptr()), val);
        }
    }

    pub fn setVec2Int(&self, val: &glm::I32Vec2, name: &str) {
        let cstr_name = CString::new(name).unwrap();

        unsafe {
            self.gl.Uniform2iv(
                self.gl.GetUniformLocation(self.id, cstr_name.as_ptr()),
                1,
                val.as_ptr(),
            );
        }
    }

    pub fn setVec3Float(&self, val: &glm::Vec3, name: &str) {
        let cstr_name = CString::new(name).unwrap();

        unsafe {
            self.gl.Uniform3fv(
                self.gl.GetUniformLocation(self.id, cstr_name.as_ptr()),
                1,
                val.as_ptr(),
            );
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    return unsafe { CString::from_vec_unchecked(buffer) };
}

fn shader_from_source(
    gl: &gl::GlPtr,
    source: &CStr,
    kind: gl::types::GLuint,
) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl.CreateShader(kind) };

    unsafe {
        gl.ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl.CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = create_whitespace_cstring_with_len(len as usize);
        unsafe {
            gl.GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar,
            )
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}
