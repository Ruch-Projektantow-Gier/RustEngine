extern crate nalgebra_glm as glm;
use gl;
use std::ffi::*;

#[path = "render_gl.rs"]
mod render_gl;

type GlInt = gl::types::GLuint;

static LINE: [f32; 6] = [
    0., 0., 0., //
    0., 0., 1.,
];

pub struct Debug {
    gl: gl::GlPtr,

    line_program: render_gl::Program,
    line_vao: GlInt,
    line_vbo: GlInt,
}

pub struct DebugDrawer<'a> {
    debug: &'a Debug,

    view: &'a glm::Mat4,
    proj: &'a glm::Mat4,
}

impl<'a> DebugDrawer<'a> {
    pub fn draw(&self, from: &glm::Vec3, to: &glm::Vec3) {
        let model_name = CString::new("model").unwrap();
        let view_name = CString::new("view").unwrap();
        let proj_name = CString::new("projection").unwrap();

        self.debug.line_program.set_used();
        let mut model = glm::translate(&glm::identity(), &glm::vec3(0., 0., 0.));

        unsafe {
            self.debug.gl.LineWidth(4.0);
            self.debug.gl.BindVertexArray(self.debug.line_vao);

            let new_z = (to - from).normalize();
            let mut new_y = glm::cross(&new_z, &glm::vec3(0., 0., 1.));

            if new_y.magnitude() < glm::epsilon() {
                new_y = glm::vec3(0., 1., 0.);
            }

            let new_x = glm::cross(&new_z, &new_y).normalize();
            {
                let x = new_x;
                let y = new_y;
                let z = new_z;
                let length = (to - from).magnitude();

                model = glm::mat4(
                    x[0] * length,
                    y[0] * length,
                    z[0] * length,
                    from[0], //
                    x[1] * length,
                    y[1] * length,
                    z[1] * length,
                    from[1], //
                    x[2] * length,
                    y[2] * length,
                    z[2] * length,
                    from[2], //
                    0.,
                    0.,
                    0.,
                    1., //
                );
            }

            self.debug.gl.UniformMatrix4fv(
                self.debug
                    .gl
                    .GetUniformLocation(self.debug.line_program.id(), view_name.as_ptr()),
                1,
                gl::FALSE,
                self.view.as_ptr(),
            );

            self.debug.gl.UniformMatrix4fv(
                self.debug
                    .gl
                    .GetUniformLocation(self.debug.line_program.id(), proj_name.as_ptr()),
                1,
                gl::FALSE,
                self.proj.as_ptr(),
            );

            self.debug.gl.UniformMatrix4fv(
                self.debug
                    .gl
                    .GetUniformLocation(self.debug.line_program.id(), model_name.as_ptr()),
                1,
                gl::FALSE,
                model.as_ptr(),
            );

            self.debug.gl.DrawArrays(gl::LINES, 0, 2);
            self.debug.gl.BindVertexArray(0);
        }
    }
}

impl Debug {
    pub fn new(gl: &gl::GlPtr) -> Debug {
        let (line_vao, line_vbo) = Debug::gen_line(&gl);
        let line_program = Debug::gen_line_shader(&gl);

        Debug {
            gl: gl.clone(),
            line_vao,
            line_vbo,
            line_program,
        }
    }

    fn gen_line_shader(gl: &gl::GlPtr) -> render_gl::Program {
        let vert_shader = render_gl::Shader::from_vert_source(
            &gl,
            &CString::new(include_str!("ray.vert")).unwrap(),
        )
        .unwrap();

        let frag_shader = render_gl::Shader::from_frag_source(
            &gl,
            &CString::new(include_str!("ray.frag")).unwrap(),
        )
        .unwrap();

        render_gl::Program::from_shaders(&gl, &[vert_shader, frag_shader]).unwrap()
    }

    fn gen_line(gl: &gl::GlPtr) -> (GlInt, GlInt) {
        let mut line_vao: GlInt = 0;
        let mut line_vbo: GlInt = 0;

        unsafe {
            gl.GenVertexArrays(1, &mut line_vao);
            gl.GenBuffers(1, &mut line_vbo);

            // bin vao
            gl.BindVertexArray(line_vao);

            // verticles (vbo)
            gl.BindBuffer(gl::ARRAY_BUFFER, line_vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (LINE.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                LINE.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW,
            );

            // positions in uniform
            gl.EnableVertexAttribArray(0);
            gl.VertexAttribPointer(
                0, // layout (location = 0)
                3,
                gl::FLOAT,
                gl::FALSE,
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride
                std::ptr::null(),                                     // offset of first component
            );

            gl.EnableVertexAttribArray(0);
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            gl.BindVertexArray(0);
        }

        (line_vao, line_vbo)
    }

    pub fn setup_drawer<'a>(&'a self, view: &'a glm::Mat4, proj: &'a glm::Mat4) -> DebugDrawer<'a> {
        DebugDrawer {
            debug: &self,
            view,
            proj,
        }
    }
}
