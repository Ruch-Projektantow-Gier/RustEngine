extern crate nalgebra_glm as glm;
use crate::primitives;
use crate::primitives::Model;
use crate::render_gl::{Program, Shader};
use gl;
use std::ffi::*;

type GlInt = gl::types::GLuint;

static LINE: [f32; 6] = [
    0., 0., 0., //
    0., 0., 1.,
];

pub struct Debug<'a> {
    gl: gl::GlPtr,

    shader: Program,
    line_vao: GlInt,
    line_vbo: GlInt,

    pyramid: Model<'a>,
}

pub struct DebugDrawer<'a> {
    debug: &'a Debug<'a>,

    view: &'a glm::Mat4,
    proj: &'a glm::Mat4,
}

impl<'a> DebugDrawer<'a> {
    pub fn draw_color(&self, from: &glm::Vec3, to: &glm::Vec3, color: &glm::Vec4, weight: f32) {
        self.debug.shader.bind();
        let mut model = glm::translate(&glm::identity(), &glm::vec3(0., 0., 0.));

        unsafe {
            self.debug.gl.Disable(gl::CULL_FACE);
            self.debug.gl.Enable(gl::BLEND);
            self.debug
                .gl
                .BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        // pyramid width
        use crate::glm::RealField;

        let test = glm::unproject(
            &glm::vec3(weight, weight, 0.),
            &self.view,
            &self.proj,
            glm::vec4(0., 0., 1., 1.),
        );

        let arrow_size = (to - test).magnitude() * weight * 0.02;

        // line-draw
        unsafe {
            self.debug.gl.LineWidth(4.0 * weight);
            self.debug.gl.BindVertexArray(self.debug.line_vao);

            let mut new_z = (to - from).normalize();
            let mut new_y = glm::cross(&new_z, &glm::vec3(0., 0., 1.));

            if new_z.magnitude() < glm::epsilon() {
                new_z = glm::vec3(0., 0., 1.);
            }

            if new_y.magnitude() < glm::epsilon() {
                new_y = glm::vec3(0., 1., 0.);
            }

            let new_x = glm::cross(&new_z, &new_y).normalize();
            {
                let x = new_x;
                let y = new_y;
                let z = new_z;
                let length = (to - from).magnitude() - arrow_size;

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

            self.debug.shader.setVec4Float(color, "color");
            self.debug.shader.setMat4(&self.view, "view");
            self.debug.shader.setMat4(&self.proj, "projection");
            self.debug.shader.setMat4(&model, "model");

            self.debug.gl.DrawArrays(gl::LINES, 0, 2);
            self.debug.gl.BindVertexArray(0);
        }

        // pyramid
        {
            let mut new_y = (to - from).normalize();
            let mut new_z = glm::cross(&new_y, &glm::vec3(1., 0., 0.));

            if new_z.magnitude() < glm::epsilon() {
                new_z = glm::vec3(0., 0., 1.);
            }

            if new_y.magnitude() < glm::epsilon() {
                new_y = glm::vec3(0., 1., 0.);
            }

            let new_x = glm::cross(&new_z, &new_y).normalize();
            {
                let x = new_x;
                let y = new_y;
                let z = new_z;

                model = glm::mat4(
                    x[0] * arrow_size,
                    y[0] * arrow_size,
                    z[0] * arrow_size,
                    to[0], //
                    x[1] * arrow_size,
                    y[1] * arrow_size,
                    z[1] * arrow_size,
                    to[1], //
                    x[2] * arrow_size,
                    y[2] * arrow_size,
                    z[2] * arrow_size,
                    to[2], //
                    0.,
                    0.,
                    0.,
                    1., //
                );
            }
        }

        model = glm::translate(&model, &glm::vec3(0., -0.5, 0.));

        self.debug.shader.setMat4(&model, "model");
        self.debug.pyramid.draw(&self.debug.shader);
    }

    pub fn draw(&self, from: &glm::Vec3, to: &glm::Vec3) {
        self.draw_color(from, to, &glm::vec4(1., 1., 1., 1.), 1.);
    }

    pub fn draw_gizmo(&self, pos: &glm::Vec3, length: f32, weight: f32) {
        self.draw_color(
            pos,
            &(pos + glm::vec3(1. * length, 0., 0.)),
            &glm::vec4(1., 0., 0., 1.),
            weight,
        );
        self.draw_color(
            pos,
            &(pos + glm::vec3(0., 1. * length, 0.)),
            &glm::vec4(0., 1., 0., 1.),
            weight,
        );
        self.draw_color(
            pos,
            &(pos + glm::vec3(0., 0., 1. * length)),
            &glm::vec4(0., 0., 1., 1.),
            weight,
        );
    }
}

// pub fn lines_collision(one: &glm::vec3) -> bool {}

impl<'a> Debug<'a> {
    pub fn new(gl: &gl::GlPtr) -> Debug {
        let (line_vao, line_vbo) = Debug::gen_line(&gl);
        let shader = Debug::gen_line_shader(&gl);
        let pyramid = primitives::build_pyramid(&gl);

        Debug {
            gl: gl.clone(),
            line_vao,
            line_vbo,
            shader,
            pyramid,
        }
    }

    fn gen_line_shader(gl: &gl::GlPtr) -> Program {
        let vert_shader = Shader::from_vert_source(
            &gl,
            &CString::new(include_str!("shaders/color/color.vert")).unwrap(),
        )
        .unwrap();

        let frag_shader = Shader::from_frag_source(
            &gl,
            &CString::new(include_str!("shaders/color/color.frag")).unwrap(),
        )
        .unwrap();

        Program::from_shaders(&gl, &[vert_shader, frag_shader]).unwrap()
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

    pub fn setup_drawer(&'a self, view: &'a glm::Mat4, proj: &'a glm::Mat4) -> DebugDrawer<'a> {
        DebugDrawer {
            debug: &self,
            view,
            proj,
        }
    }
}
