extern crate nalgebra_glm as glm;
use gl;
use std::ffi::*;

#[path = "render_gl.rs"]
mod render_gl;

type GlInt = gl::types::GLuint;

static CUBE: [f32; 180] = [
    -0.5, -0.5, -0.5, /* */ 0.0, 0.0, //
    0.5, -0.5, -0.5, /* */ 1.0, 0.0, //
    0.5, 0.5, -0.5, /* */ 1.0, 1.0, //
    0.5, 0.5, -0.5, /* */ 1.0, 1.0, //
    -0.5, 0.5, -0.5, /* */ 0.0, 1.0, //
    -0.5, -0.5, -0.5, /* */ 0.0, 0.0, //
    -0.5, -0.5, 0.5, /* */ 0.0, 0.0, //
    0.5, -0.5, 0.5, /* */ 1.0, 0.0, //
    0.5, 0.5, 0.5, /* */ 1.0, 1.0, //
    0.5, 0.5, 0.5, /* */ 1.0, 1.0, //
    -0.5, 0.5, 0.5, /* */ 0.0, 1.0, //
    -0.5, -0.5, 0.5, /* */ 0.0, 0.0, //
    -0.5, 0.5, 0.5, /* */ 1.0, 0.0, //
    -0.5, 0.5, -0.5, /* */ 1.0, 1.0, //
    -0.5, -0.5, -0.5, /* */ 0.0, 1.0, //
    -0.5, -0.5, -0.5, /* */ 0.0, 1.0, //
    -0.5, -0.5, 0.5, /* */ 0.0, 0.0, //
    -0.5, 0.5, 0.5, /* */ 1.0, 0.0, //
    0.5, 0.5, 0.5, /* */ 1.0, 0.0, //
    0.5, 0.5, -0.5, /* */ 1.0, 1.0, //
    0.5, -0.5, -0.5, /* */ 0.0, 1.0, //
    0.5, -0.5, -0.5, /* */ 0.0, 1.0, //
    0.5, -0.5, 0.5, /* */ 0.0, 0.0, //
    0.5, 0.5, 0.5, /* */ 1.0, 0.0, //
    -0.5, -0.5, -0.5, /* */ 0.0, 1.0, //
    0.5, -0.5, -0.5, /* */ 1.0, 1.0, //
    0.5, -0.5, 0.5, /* */ 1.0, 0.0, //
    0.5, -0.5, 0.5, /* */ 1.0, 0.0, //
    -0.5, -0.5, 0.5, /* */ 0.0, 0.0, //
    -0.5, -0.5, -0.5, /* */ 0.0, 1.0, //
    -0.5, 0.5, -0.5, /* */ 0.0, 1.0, //
    0.5, 0.5, -0.5, /* */ 1.0, 1.0, //
    0.5, 0.5, 0.5, /* */ 1.0, 0.0, //
    0.5, 0.5, 0.5, /* */ 1.0, 0.0, //
    -0.5, 0.5, 0.5, /* */ 0.0, 0.0, //
    -0.5, 0.5, -0.5, /* */ 0.0, 1.0, //
];

pub struct Model {
    gl: gl::GlPtr,

    vao: GlInt,
    vbo: GlInt,
    ebo: GlInt,
}
