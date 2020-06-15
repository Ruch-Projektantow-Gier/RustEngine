extern crate nalgebra_glm as glm;
use crate::render_gl::{Program, Shader};
use crate::texture;
use crate::texture::{Texture, TextureKind};
use gl;
use std::ffi::*;

#[path = "render_gl.rs"]
mod render_gl;
#[path = "sphere.rs"]
mod sphere;

type GlInt = gl::types::GLuint;

static QUAD: [f32; 24] = [
    /* pos, uv */
    -1.0, 1.0, 0.0, 1.0, //
    -1.0, -1.0, 0.0, 0.0, //
    1.0, -1.0, 1.0, 0.0, //
    -1.0, 1.0, 0.0, 1.0, //
    1.0, -1.0, 1.0, 0.0, //
    1.0, 1.0, 1.0, 1.0, //
];

pub static CUBE: [f32; 288] = [
    // Back face
    -0.5, -0.5, -0.5, /* normal */ 0., 0., -1., /* uv */ 0.0, 0.0, // Bottom-left
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

pub type TextureAttachment<'a> = (&'a Texture, TextureKind);

pub struct Model<'a> {
    gl: gl::GlPtr,

    vao: GlInt,
    vbo: GlInt,
    ebo: GlInt,

    triangles: i32,
    textures: Vec<TextureAttachment<'a>>,
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
        // unbind after vao
    }

    vbo
}

/* indicies */
fn setup_ebo(gl: &gl::GlPtr, indices: &[u32]) -> GlInt {
    let mut ebo: GlInt = 0;

    // Remember to bind vao first
    unsafe {
        gl.GenBuffers(1, &mut ebo);

        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl.BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
            indices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );
        // unbind after vao
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

        // It's needs to be unbinded after vao
        gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
    }

    vao
}

pub fn create<'a>(
    gl: &gl::GlPtr,
    vertices: &Vec<f32>,
    locations: &[i32],
    textures: Vec<TextureAttachment<'a>>,
) -> Model<'a> {
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
        textures,
    }
}

pub fn create_with_indices<'a>(
    gl: &gl::GlPtr,
    vertices: &Vec<f32>,
    indices: &Vec<u32>,
    locations: &[i32],
    textures: Vec<TextureAttachment<'a>>,
) -> Model<'a> {
    let triangles = indices.len() as i32;

    let mut vao: GlInt = 0;
    let mut vbo: GlInt = 0;
    let mut ebo: GlInt = 0;

    vao = setup_vao(&gl, || {
        ebo = setup_ebo(&gl, &indices);
        vbo = setup_vbo(&gl, &vertices, &locations);
    });

    Model {
        gl: gl.clone(),
        vao,
        vbo,
        ebo,
        triangles,
        textures,
    }
}

impl Model<'_> {
    pub fn raw_draw(&self, mode: gl::types::GLenum) {
        unsafe {
            self.gl.BindVertexArray(self.vao);

            if self.ebo != 0 {
                self.gl
                    .DrawElements(mode, self.triangles, gl::UNSIGNED_INT, std::ptr::null());
            } else {
                self.gl.DrawArrays(mode, 0, self.triangles);
            }

            self.gl.BindVertexArray(0);
        }
    }

    pub fn draw(&self, shader: &Program) {
        self.bind_textures_to(&shader);
        self.raw_draw(gl::TRIANGLES);
        self.unbind_textures_from(&shader);
    }

    pub fn draw_mesh(&self, line_width: f32) {
        unsafe {
            self.gl.Disable(gl::CULL_FACE);
            self.gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

            self.gl.LineWidth(line_width);

            self.gl.Enable(gl::BLEND);
            self.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        self.raw_draw(gl::TRIANGLES);

        unsafe {
            self.gl.PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            self.gl.Disable(gl::BLEND);
        }
    }

    pub fn draw_vertices(&self, point_size: f32) {
        unsafe {
            self.gl.Disable(gl::CULL_FACE);
            self.gl.PolygonMode(gl::FRONT_AND_BACK, gl::POINT);

            self.gl.PointSize(point_size);

            self.gl.Enable(gl::BLEND);
            self.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        self.raw_draw(gl::TRIANGLES);

        unsafe {
            self.gl.PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            self.gl.Disable(gl::BLEND);
        }
    }

    pub fn bind_textures_to(&self, shader: &Program) {
        let mut diffuse_number = 1;
        let mut specular_number = 1;
        let mut normal_number = 1;
        let mut height_number = 1;

        for (i, &(texture, kind)) in self.textures.iter().enumerate() {
            unsafe {
                self.gl.ActiveTexture(gl::TEXTURE0 + i as u32);

                match kind {
                    TextureKind::Diffuse => diffuse_number += 1,
                    TextureKind::Specular => specular_number += 1,
                    TextureKind::Normal => normal_number += 1,
                    TextureKind::Height => height_number += 1,
                }

                shader.setInt(i as i32, &format!("{}{}", kind.as_str(), i));
                texture.bind();
            }
        }
    }

    fn unbind_textures_from(&self, shader: &Program) {
        for (i, texture) in self.textures.iter().enumerate() {
            unsafe {
                self.gl.ActiveTexture(gl::TEXTURE0 + i as u32);
                self.gl.BindTexture(gl::TEXTURE_2D, 0);
            }
        }
    }
}

// http://www.opengl-tutorial.org/intermediate-tutorials/tutorial-13-normal-mapping/#computing-the-tangents-and-bitangents
pub fn compute_tangent(
    indices: &Vec<u32>,
    vertices: &Vec<glm::Vec3>,
    uvs: &Vec<glm::Vec2>,
) -> (Vec<glm::Vec3>, Vec<glm::Vec3>) {
    let mut tangents = vec![];
    let mut bitangents = vec![];

    for index in indices.chunks(3) {
        let v0 = &vertices[index[0] as usize];
        let v1 = &vertices[index[1] as usize];
        let v2 = &vertices[index[2] as usize];

        let uv0 = &uvs[index[0] as usize];
        let uv1 = &uvs[index[1] as usize];
        let uv2 = &uvs[index[2] as usize];

        // Edges of the triangle : position delta
        let delta_pos_1 = v1 - v0;
        let delta_pos_2 = v2 - v0;

        // UV delta
        let delta_uv_1 = uv1 - uv0;
        let delta_uv_2 = uv2 - uv0;

        let r = 1.0 / (delta_uv_1.x * delta_uv_2.y - delta_uv_1.y * delta_uv_2.x);
        let tangent = (delta_pos_1 * delta_uv_2.y - delta_pos_2 * delta_uv_1.y) * r;
        let bitangent = (delta_pos_2 * delta_uv_1.x - delta_pos_1 * delta_uv_2.x) * r;

        tangents.push(tangent);
        tangents.push(tangent);
        tangents.push(tangent);

        bitangents.push(bitangent);
        bitangents.push(bitangent);
        bitangents.push(bitangent);
    }

    (tangents, bitangents)
}

/* primitives */
pub fn build_cube<'a>(gl: &gl::GlPtr, textures: Vec<TextureAttachment<'a>>) -> Model<'a> {
    create(
        &gl,
        &CUBE.to_vec(),
        &[
            3, /* verticles */
            3, /* normals */
            2, /* texture coords */
        ],
        textures,
    )
}

pub fn build_quad<'a>(gl: &gl::GlPtr, textures: Vec<TextureAttachment<'a>>) -> Model<'a> {
    create(
        &gl,
        &QUAD.to_vec(),
        &[
            2, // verticles
            2, // texture coords
        ],
        textures,
    )
}

pub fn build_sphere<'a>(gl: &gl::GlPtr, textures: Vec<TextureAttachment<'a>>) -> Model<'a> {
    // let (vertices, indices) = sphere::gen_sphere(1.0, 30, 30);
    let (vertices, indices) = sphere::build_isosphere();

    create_with_indices(
        &gl,
        &vertices,
        &indices,
        &[
            3, /* verticles */
            3, /* normals */
            2, /* texture coords */
        ],
        textures,
    )
}
