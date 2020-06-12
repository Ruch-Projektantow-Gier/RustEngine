extern crate nalgebra_glm as glm;
use gl;
use std::ffi::*;

#[path = "render_gl.rs"]
mod render_gl;
#[path = "sphere.rs"]
mod sphere;

type GlInt = gl::types::GLuint;

static QUAD: [f32; 20] = [
    -1.0, 1.0, 0.0, 0.0, 1.0, //
    -1.0, -1.0, 0.0, 0.0, 0.0, //
    1.0, 1.0, 0.0, 1.0, 1.0, //
    1.0, -1.0, 0.0, 1.0, 0.0, //
];

static CUBE: [f32; 288] = [
    // Back face
    -0.5, -0.5, -0.5, 0., 0., -1., 0.0, 0.0, // Bottom-left
    0.5, -0.5, -0.5, 0., 0., -1., 1.0, 0.0, // bottom-right
    0.5, 0.5, -0.5, 0., 0., -1., 1.0, 1.0, // top-right
    0.5, 0.5, -0.5, 0., 0., -1., 1.0, 1.0, // top-right
    -0.5, 0.5, -0.5, 0., 0., -1., 0.0, 1.0, // top-left
    -0.5, -0.5, -0.5, 0., 0., -1., 0.0, 0.0, // bottom-left
    // Front face
    -0.5, -0.5, 0.5, 0., 0., 1., 0.0, 0.0, // bottom-left
    0.5, 0.5, 0.5, 0., 0., 1., 1.0, 1.0, // top-right
    0.5, -0.5, 0.5, 0., 0., 1., 1.0, 0.0, // bottom-right
    0.5, 0.5, 0.5, 0., 0., 1., 1.0, 1.0, // top-right
    -0.5, -0.5, 0.5, 0., 0., 1., 0.0, 0.0, // bottom-left
    -0.5, 0.5, 0.5, 0., 0., 1., 0.0, 1.0, // top-left
    // Left face
    -0.5, 0.5, 0.5, -1., 0., 0., 1.0, 0.0, // top-right
    -0.5, -0.5, -0.5, -1., 0., 0., 0.0, 1.0, // bottom-left
    -0.5, 0.5, -0.5, -1., 0., 0., 1.0, 1.0, // top-left
    -0.5, -0.5, -0.5, -1., 0., 0., 0.0, 1.0, // bottom-left
    -0.5, 0.5, 0.5, -1., 0., 0., 1.0, 0.0, // top-right
    -0.5, -0.5, 0.5, -1., 0., 0., 0.0, 0.0, // bottom-right
    // Right face
    0.5, 0.5, 0.5, 1., 0., 0., 1.0, 0.0, // top-left
    0.5, 0.5, -0.5, 1., 0., 0., 1.0, 1.0, // top-right
    0.5, -0.5, -0.5, 1., 0., 0., 0.0, 1.0, // bottom-right
    0.5, -0.5, -0.5, 1., 0., 0., 0.0, 1.0, // bottom-right
    0.5, -0.5, 0.5, 1., 0., 0., 0.0, 0.0, // bottom-left
    0.5, 0.5, 0.5, 1., 0., 0., 1.0, 0.0, // top-left
    // Bottom face
    -0.5, -0.5, -0.5, 0., -1., 0., 0.0, 1.0, // top-right
    0.5, -0.5, 0.5, 0., -1., 0., 1.0, 0.0, // bottom-left
    0.5, -0.5, -0.5, 0., -1., 0., 1.0, 1.0, // top-left
    0.5, -0.5, 0.5, 0., -1., 0., 1.0, 0.0, // bottom-left
    -0.5, -0.5, -0.5, 0., -1., 0., 0.0, 1.0, // top-right
    -0.5, -0.5, 0.5, 0., -1., 0., 0.0, 0.0, // bottom-right
    // Top face
    -0.5, 0.5, -0.5, 0., 1., 0., 0.0, 1.0, // top-left
    0.5, 0.5, -0.5, 0., 1., 0., 1.0, 1.0, // top-right
    0.5, 0.5, 0.5, 0., 1., 0., 1.0, 0.0, // bottom-right
    0.5, 0.5, 0.5, 0., 1., 0., 1.0, 0.0, // bottom-right
    -0.5, 0.5, 0.5, 0., 1., 0., 0.0, 0.0, // bottom-left
    -0.5, 0.5, -0.5, 0., 1., 0., 0.0, 1.0, // top-left
];

pub struct Model {
    gl: gl::GlPtr,

    vao: GlInt,
    vbo: GlInt,
    ebo: GlInt,

    triangles: i32,
}

fn setup_vertex_attrib(gl: &gl::GlPtr, locations: &[i32]) {
    let stride: i32 = locations.iter().sum();
    let mut offset: i32 = 0;

    let vertex_size = std::mem::size_of::<f32>() as i32;

    // i == (location in shader)
    for i in 0..locations.len() {
        let len = locations[i];

        unsafe {
            gl.EnableVertexAttribArray(i as u32);
            gl.VertexAttribPointer(
                i as u32, // layout (location = 0)
                len,
                gl::FLOAT,
                gl::FALSE,
                (stride * vertex_size) as gl::types::GLint, // stride
                (offset * vertex_size) as *const gl::types::GLvoid, // offset of first component
            );
        }

        offset += len;
    }
}

/* https://www.khronos.org/opengl/wiki/Vertex_Specification#Vertex_Buffer_Object */
fn setup_vbo(gl: &gl::GlPtr, vertices: &Vec<f32>, locations: &[i32]) -> GlInt {
    let mut vbo: GlInt = 0;

    // Remember to bind vao first
    unsafe {
        gl.GenBuffers(1, &mut vbo);

        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl.BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );
        setup_vertex_attrib(&gl, &locations);
        gl.BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    vbo
}

/* indicies */
fn setup_ebo(gl: &gl::GlPtr, indices: &[i32]) -> GlInt {
    let mut ebo: GlInt = 0;

    // Remember to bind vao first
    unsafe {
        gl.GenBuffers(1, &mut ebo);

        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl.BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<i32>()) as gl::types::GLsizeiptr,
            indices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );
        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
    }

    ebo
}

/* indicies */
fn setup_vao<F>(gl: &gl::GlPtr, f: F) -> GlInt
where
    F: FnOnce(),
{
    let mut vao: GlInt = 0;

    unsafe {
        gl.GenVertexArrays(1, &mut vao);

        gl.BindVertexArray(vao);
        f();
        gl.BindVertexArray(0);
    }

    vao
}

pub fn from_vertices(gl: &gl::GlPtr, vertices: &Vec<f32>, locations: &[i32]) -> Model {
    let mut triangles = vertices.len() as i32;
    let stride: i32 = locations.iter().sum();

    if stride != 0 {
        triangles /= stride;
    }

    let mut vao: GlInt = 0;
    let mut vbo: GlInt = 0;

    vao = setup_vao(&gl, || {
        vbo = setup_vbo(&gl, &vertices, &locations);
    });

    Model {
        gl: gl.clone(),
        vao,
        vbo,
        ebo: 0,
        triangles,
    }
}

impl Model {
    pub fn draw(&self) {
        unsafe {
            self.gl.BindVertexArray(self.vao);
            // glDrawElements is for indicies
            self.gl.DrawArrays(gl::TRIANGLE_STRIP, 0, self.triangles);
            self.gl.BindVertexArray(0);
        }
    }
}

/* primitives */
pub fn build_cube(gl: &gl::GlPtr) -> Model {
    from_vertices(
        &gl,
        &CUBE.to_vec(),
        &[
            3, /* verticles */
            3, /* normals */
            2, /* texture coords */
        ],
    )
}

pub fn build_quad(gl: &gl::GlPtr) -> Model {
    from_vertices(
        &gl,
        &QUAD.to_vec(),
        &[3 /* verticles */, 2 /* texture coords */],
    )
}

pub fn build_sphere(gl: &gl::GlPtr) -> Model {
    from_vertices(&gl, &sphere::gen_sphere_vertices(10), &[3 /* verticles */])
}
