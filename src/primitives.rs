extern crate nalgebra_glm as glm;
use crate::shader::{Program, Shader};
use crate::texture;
use crate::texture::{Texture, TextureKind};
use gl;
use itertools::{zip_eq, Itertools};
use std::borrow::Borrow;
use std::cmp::max;
use std::ffi::*;

#[path = "shader.rs"]
mod shader;
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
    // Top face
    -0.5, -0.5, -0.5, /* normal */ 0., 0., -1., /* uv */ 0.0, 0.0, // Bottom-left
    0.5, -0.5, -0.5, 0., 0., -1., 1.0, 0.0, // bottom-right
    0.5, 0.5, -0.5, 0., 0., -1., 1.0, 1.0, // top-right
    0.5, 0.5, -0.5, 0., 0., -1., 1.0, 1.0, // top-right
    -0.5, 0.5, -0.5, 0., 0., -1., 0.0, 1.0, // top-left
    -0.5, -0.5, -0.5, 0., 0., -1., 0.0, 0.0, // bottom-left
    // Bottom face
    -0.5, -0.5, 0.5, 0., 0., 1., 0.0, 1.0, // bottom-left
    0.5, 0.5, 0.5, 0., 0., 1., 1.0, 0.0, // top-right
    0.5, -0.5, 0.5, 0., 0., 1., 1.0, 1.0, // bottom-right
    0.5, 0.5, 0.5, 0., 0., 1., 1.0, 0.0, // top-right
    -0.5, -0.5, 0.5, 0., 0., 1., 0.0, 1.0, // bottom-left
    -0.5, 0.5, 0.5, 0., 0., 1., 0.0, 0.0, // top-left
    // Left face
    -0.5, 0.5, 0.5, -1., 0., 0., 1.0, 1.0, // top-right
    -0.5, -0.5, -0.5, -1., 0., 0., 0.0, 0.0, // bottom-left
    -0.5, 0.5, -0.5, -1., 0., 0., 1.0, 0.0, // top-left
    -0.5, -0.5, -0.5, -1., 0., 0., 0.0, 0.0, // bottom-left
    -0.5, 0.5, 0.5, -1., 0., 0., 1.0, 1.0, // top-right
    -0.5, -0.5, 0.5, -1., 0., 0., 0.0, 1.0, // bottom-right
    // Right face
    0.5, 0.5, 0.5, 1., 0., 0., 1.0, 0.0, // top-left
    0.5, 0.5, -0.5, 1., 0., 0., 1.0, 1.0, // top-right
    0.5, -0.5, -0.5, 1., 0., 0., 0.0, 1.0, // bottom-right
    0.5, -0.5, -0.5, 1., 0., 0., 0.0, 1.0, // bottom-right
    0.5, -0.5, 0.5, 1., 0., 0., 0.0, 0.0, // bottom-left
    0.5, 0.5, 0.5, 1., 0., 0., 1.0, 0.0, // top-left
    // Back face
    -0.5, -0.5, -0.5, 0., -1., 0., 0.0, 1.0, // top-right
    0.5, -0.5, 0.5, 0., -1., 0., 1.0, 0.0, // bottom-left
    0.5, -0.5, -0.5, 0., -1., 0., 1.0, 1.0, // top-left
    0.5, -0.5, 0.5, 0., -1., 0., 1.0, 0.0, // bottom-left
    -0.5, -0.5, -0.5, 0., -1., 0., 0.0, 1.0, // top-right
    -0.5, -0.5, 0.5, 0., -1., 0., 0.0, 0.0, // bottom-right
    // Front face
    -0.5, 0.5, -0.5, 0., 1., 0., 0.0, 0.0, // top-left
    0.5, 0.5, -0.5, 0., 1., 0., 1.0, 0.0, // top-right
    0.5, 0.5, 0.5, 0., 1., 0., 1.0, 1.0, // bottom-right
    0.5, 0.5, 0.5, 0., 1., 0., 1.0, 1.0, // bottom-right
    -0.5, 0.5, 0.5, 0., 1., 0., 0.0, 1.0, // bottom-left
    -0.5, 0.5, -0.5, 0., 1., 0., 0.0, 0.0, // top-left
];

fn scale_uv(verts: &Vec<f32>, width: f32, height: f32) -> Vec<f32> {
    verts
        .chunks(8)
        .map(|chunk| {
            let mut chunk = chunk.to_vec();

            chunk[6] *= width;
            chunk[7] *= height;

            chunk
        })
        .flatten()
        .collect()
}

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
        self.bind_textures_to(&shader, true);
        self.raw_draw(gl::TRIANGLES);
        self.unbind_textures_from(&shader);
    }

    pub fn draw_no_scaled(&self, shader: &Program) {
        self.bind_textures_to(&shader, false);
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

    pub fn draw_lines(&self, line_width: f32) {
        unsafe {
            self.gl.Disable(gl::CULL_FACE);
            self.gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

            self.gl.LineWidth(line_width);

            self.gl.Enable(gl::BLEND);
            self.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        self.raw_draw(gl::LINES);

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

    pub fn bind_textures_to(&self, shader: &Program, mipmap: bool) {
        let mut diffuse_number = 1;
        let mut specular_number = 1;
        let mut normal_number = 1;
        let mut height_number = 1;

        for (i, &(texture, kind)) in self.textures.iter().enumerate() {
            unsafe {
                self.gl.ActiveTexture(gl::TEXTURE0 + i as u32);

                let number = match kind {
                    TextureKind::Diffuse => &mut diffuse_number,
                    TextureKind::Specular => &mut specular_number,
                    TextureKind::Normal => &mut normal_number,
                    TextureKind::Height => &mut height_number,
                };

                shader.setInt(i as i32, &format!("material.{}{}", kind.as_str(), number));
                texture.bind();

                unsafe {
                    self.gl
                        .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
                    self.gl
                        .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

                    // self.gl.TexParameteri(
                    //     gl::TEXTURE_2D,
                    //     gl::TEXTURE_WRAP_S,
                    //     gl::CLAMP_TO_EDGE as gl::types::GLint,
                    // );
                    // self.gl.TexParameteri(
                    //     gl::TEXTURE_2D,
                    //     gl::TEXTURE_WRAP_T,
                    //     gl::CLAMP_TO_EDGE as gl::types::GLint,
                    // );

                    if mipmap {
                        self.gl.TexParameteri(
                            gl::TEXTURE_2D,
                            gl::TEXTURE_MIN_FILTER,
                            gl::LINEAR_MIPMAP_NEAREST as gl::types::GLint,
                        );
                    } else {
                        self.gl.TexParameteri(
                            gl::TEXTURE_2D,
                            gl::TEXTURE_MIN_FILTER,
                            gl::LINEAR as gl::types::GLint,
                        );
                    }

                    self.gl.TexParameteri(
                        gl::TEXTURE_2D,
                        gl::TEXTURE_MAG_FILTER,
                        gl::LINEAR as gl::types::GLint,
                    );
                }

                *number += 1;
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

        let delta_pos_1 = v1 - v0; // edge
        let delta_pos_2 = v2 - v0;
        let delta_uv_1 = uv1 - uv0; //
        let delta_uv_2 = uv2 - uv0;

        let r = 1.0 / (delta_uv_1.x * delta_uv_2.y - delta_uv_2.x * delta_uv_1.y);

        let tangent = glm::vec3(
            delta_uv_2.y * delta_pos_1.x - delta_uv_1.y * delta_pos_2.x,
            delta_uv_2.y * delta_pos_1.y - delta_uv_1.y * delta_pos_2.y,
            delta_uv_2.y * delta_pos_1.z - delta_uv_1.y * delta_pos_2.z,
        ) * r;

        let bitangent = glm::vec3(
            -delta_uv_2.x * delta_pos_1.x - delta_uv_1.x * delta_pos_2.x,
            -delta_uv_2.x * delta_pos_1.y - delta_uv_1.x * delta_pos_2.y,
            -delta_uv_2.x * delta_pos_1.z - delta_uv_1.x * delta_pos_2.z,
        ) * r;

        tangents.push(tangent);
        tangents.push(tangent);
        tangents.push(tangent);

        bitangents.push(bitangent);
        bitangents.push(bitangent);
        bitangents.push(bitangent);
    }

    (tangents, bitangents)
}

pub fn compute_tangent_without_indicies(
    vertices: &Vec<glm::Vec3>,
    uvs: &Vec<glm::Vec2>,
) -> (Vec<glm::Vec3>, Vec<glm::Vec3>) {
    let mut tangents = vec![];
    let mut bitangents = vec![];

    for (v, uv) in zip_eq(vertices.chunks(3), uvs.chunks(3)) {
        let v0 = &v[0];
        let v1 = &v[1];
        let v2 = &v[2];

        let uv0 = &uv[0];
        let uv1 = &uv[1];
        let uv2 = &uv[2];

        let delta_pos_1 = v1 - v0; // edge
        let delta_pos_2 = v2 - v0;
        let delta_uv_1 = uv1 - uv0; //
        let delta_uv_2 = uv2 - uv0;

        let r = 1.0 / (delta_uv_1.x * delta_uv_2.y - delta_uv_2.x * delta_uv_1.y);

        let tangent = glm::vec3(
            delta_uv_2.y * delta_pos_1.x - delta_uv_1.y * delta_pos_2.x,
            delta_uv_2.y * delta_pos_1.y - delta_uv_1.y * delta_pos_2.y,
            delta_uv_2.y * delta_pos_1.z - delta_uv_1.y * delta_pos_2.z,
        ) * r;

        let bitangent = glm::vec3(
            -delta_uv_2.x * delta_pos_1.x - delta_uv_1.x * delta_pos_2.x,
            -delta_uv_2.x * delta_pos_1.y - delta_uv_1.x * delta_pos_2.y,
            -delta_uv_2.x * delta_pos_1.z - delta_uv_1.x * delta_pos_2.z,
        ) * r;

        tangents.push(tangent);
        tangents.push(tangent);
        tangents.push(tangent);

        bitangents.push(bitangent);
        bitangents.push(bitangent);
        bitangents.push(bitangent);
    }

    (tangents, bitangents)
}

fn bundle_from_source(source: Vec<f32>) -> Vec<f32> {
    let mut data = vec![];

    let mut verticles = vec![];
    let mut normals = vec![];
    let mut texture_coords = vec![];

    for result in source.chunks(8) {
        verticles.push(glm::vec3(result[0], result[1], result[2]));
        normals.push(glm::vec3(result[3], result[4], result[5]));
        texture_coords.push(glm::vec2(result[6], result[7]));
    }

    let (tangents, bitangents) = compute_tangent_without_indicies(&verticles, &texture_coords);

    use itertools::izip;
    for (v, n, uv, t, b) in izip!(verticles, normals, texture_coords, tangents, bitangents) {
        data.append(&mut (v as glm::Vec3).as_mut_slice().to_vec());
        data.append(&mut (n as glm::Vec3).as_mut_slice().to_vec());
        data.append(&mut (uv as glm::Vec2).as_mut_slice().to_vec());
        data.append(&mut (t as glm::Vec3).as_mut_slice().to_vec());
        data.append(&mut (b as glm::Vec3).as_mut_slice().to_vec());
    }

    data
}

fn bundle_from_source_indices(source: Vec<f32>, indices: &Vec<u32>) -> Vec<f32> {
    let mut data = vec![];

    let mut verticles = vec![];
    let mut normals = vec![];
    let mut texture_coords = vec![];

    for result in source.chunks(8) {
        verticles.push(glm::vec3(result[0], result[1], result[2]));
        normals.push(glm::vec3(result[3], result[4], result[5]));
        texture_coords.push(glm::vec2(result[6], result[7]));
    }

    let (tangents, bitangents) = compute_tangent(&indices, &verticles, &texture_coords);

    use itertools::izip;
    for (v, n, uv, t, b) in izip!(verticles, normals, texture_coords, tangents, bitangents) {
        data.append(&mut (v as glm::Vec3).as_mut_slice().to_vec());
        data.append(&mut (n as glm::Vec3).as_mut_slice().to_vec());
        data.append(&mut (uv as glm::Vec2).as_mut_slice().to_vec());
        data.append(&mut (t as glm::Vec3).as_mut_slice().to_vec());
        data.append(&mut (b as glm::Vec3).as_mut_slice().to_vec());
    }

    data
}

/* primitives */
pub fn build_cube<'a>(
    gl: &gl::GlPtr,
    textures: Vec<TextureAttachment<'a>>,
    scale_x: f32,
    scale_y: f32,
) -> Model<'a> {
    let cube = scale_uv(&CUBE.to_vec(), scale_x, scale_y);
    let data = bundle_from_source(cube);

    create(
        &gl,
        &data,
        &[
            3, /* verticles */
            3, /* normals */
            2, /* texture coords */
            3, /* t */
            3, /* b */
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
    let ((vertices, _, _), indices) = sphere::build_isosphere();

    let mut normals = vec![];
    let mut tex_coords = vec![];

    // todo https://mft-dev.dk/uv-mapping-sphere/
    // you can iterate indices per 8 as well
    vertices.chunks(3).for_each(|chunk| {
        let vec = chunk.to_vec();

        let x = *vec.get(0).unwrap();
        let y = *vec.get(1).unwrap();
        let z = *vec.get(2).unwrap();

        let test = glm::vec3(x, y, z).normalize();

        use crate::glm::RealField;
        let u: f32 = f32::clamp(
            (test.z.atan2(test.x) + f32::pi()) / (2. * f32::pi()),
            0.0,
            1.0,
        );
        let v: f32 = f32::clamp((test.y.asin() / f32::pi()) + 0.5, 0.0, 1.0);

        // let u: f32 = ((test.x / test.z).atan() + f32::pi()) / (2. * f32::pi());
        // let v: f32 = (-test.y.asin() + f32::pi() / 2.) / f32::pi();

        // let m = 2. * (x.powi(2) + y.powi(2) + (z+1.).powi(2)).sqrt();
        // let u = (x/m) + 0.5;
        // let v = (y/m) + 0.5;

        tex_coords.push(u);
        tex_coords.push(v);

        // normals
        normals.push(test.x);
        normals.push(test.y);
        normals.push(test.z);
    });

    println!("{}", vertices.len());
    println!("{}", normals.len());
    // assert_eq!(t1.len(), tex_coords.len());

    let bundle = sphere::bundle(vertices, normals, tex_coords);
    let data = bundle_from_source_indices(bundle, &indices);

    create_with_indices(
        &gl,
        &data,
        &indices,
        &[
            3, /* verticles */
            3, /* normals */
            2, /* texture coords */
            3, /* t */
            3, /* b */
        ],
        textures,
    )
}

pub fn build_grid<'a>(gl: &gl::GlPtr, steps: i32) -> Model<'a> {
    let mut lines = vec![];

    for i in 0..=steps {
        let step = (i as f32 / steps as f32) * 2. - 1.;

        lines.append(&mut vec![step, 0., 1.]);
        lines.append(&mut vec![step, 0., -1.]);

        lines.append(&mut vec![1., 0., step]);
        lines.append(&mut vec![-1., 0., step]);
    }

    create(
        &gl,
        &lines,
        &[
            3, // verticles
        ],
        vec![],
    )
}

pub fn build_pyramid<'a>(gl: &gl::GlPtr) -> Model<'a> {
    let unit = std::f32::consts::FRAC_1_SQRT_2; // 0.7071

    create(
        &gl,
        &vec![
            // Front face
            -0.5, -0.5, 0.5, 0., unit, unit, // bottom-left
            0.5, -0.5, 0.5, 0., unit, unit, // bottom-right
            0., 0.5, 0., 0., unit, unit, // top-center
            // Right face
            0.5, -0.5, 0.5, unit, unit, 0., // bottom-left
            0.5, -0.5, -0.5, unit, unit, 0., // bottom-right
            0., 0.5, 0., unit, unit, 0., // top-center
            // Back face
            0.5, -0.5, -0.5, 0., unit, -unit, // bottom-left
            -0.5, -0.5, -0.5, 0., unit, -unit, // bottom-right
            0., 0.5, 0., 0., unit, -unit, // top-center
            // Left face
            -0.5, -0.5, -0.5, -unit, unit, 0., // bottom-left
            -0.5, -0.5, 0.5, -unit, unit, 0., // bottom-right
            0., 0.5, 0., -unit, unit, 0., // top-center
            // Bottom face
            0.5, -0.5, 0.5, 0., -1., 0., //
            -0.5, -0.5, 0.5, 0., -1., 0., //
            0.5, -0.5, -0.5, 0., -1., 0., //
            0.5, -0.5, -0.5, 0., -1., 0., //
            -0.5, -0.5, 0.5, 0., -1., 0., //
            -0.5, -0.5, -0.5, 0., -1., 0., //
        ]
        .to_vec(),
        &[3 /* verticles */, 3 /* normals */],
        vec![],
    )
}
