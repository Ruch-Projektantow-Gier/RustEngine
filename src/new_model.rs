extern crate nalgebra_glm as glm;
use crate::render_gl::{Program, Shader};
use crate::texture::{Texture, TextureKind};
use crate::{primitives, texture};
use assimp::Vector3D;
use gl;
use path_slash::PathBufExt;
use std::collections::HashMap;
use std::ffi::*;
use std::path::{Path, PathBuf};

extern crate tobj;

#[path = "render_gl.rs"]
mod render_gl;

type GlInt = gl::types::GLuint;

pub struct Material<'a> {
    diffuse: Option<&'a Texture>,
    specular: Option<&'a Texture>,
    normal: Option<&'a Texture>,  // normal_texture
    ambient: Option<&'a Texture>, // in shader also called as height
}

pub struct Vertex {
    position: glm::Vec3,
    normal: glm::Vec3,
    texture_coords: glm::Vec2,
    tangent: glm::Vec3,
    bitangent: glm::Vec3,
}

pub struct Mesh<'a> {
    gl: gl::GlPtr,

    vao: GlInt,
    vbo: GlInt,
    ebo: GlInt,

    vertices: Vec<Vertex>,
    indices: Vec<GlInt>,
    material: Material<'a>,
}

pub struct Model<'a> {
    gl: gl::GlPtr,

    meshes: Vec<Mesh<'a>>,
    textures: HashMap<String, Texture>,
}

impl Model<'_> {
    // pub fn new(gl: &gl::GlPtr, path: 'static str) -> Self<'_> {
    //
    //     Self {
    //
    // }
    // }

    pub fn load(gl: &gl::GlPtr, path: &'static str) {
        let path = Path::new(path);
        let (models, materials) = tobj::load_obj(&path, true).expect("Failed to load file");

        let mut textures: HashMap<String, Texture> = HashMap::new();

        let materials = materials
            .iter()
            .map(|material| {
                let getPath = |name: &String| path.parent().unwrap().join(name.replace("\\", "/"));

                let diffuse = textures.entry(material.specular_texture.clone()).or_insert(
                    Texture::new(gl, getPath(&material.diffuse_texture), TextureKind::Diffuse)
                        .unwrap(),
                );

                let specular = textures.entry(material.specular_texture.clone()).or_insert(
                    Texture::new(
                        gl,
                        getPath(&material.specular_texture),
                        TextureKind::Specular,
                    )
                    .unwrap(),
                );
                let normal = textures.entry(material.specular_texture.clone()).or_insert(
                    Texture::new(gl, getPath(&material.normal_texture), TextureKind::Normal)
                        .unwrap(),
                );
                let ambient = textures.entry(material.specular_texture.clone()).or_insert(
                    Texture::new(gl, getPath(&material.ambient_texture), TextureKind::Height)
                        .unwrap(),
                );

                Material {}
            })
            .collect::<Vec<_>>();

        for model in models {
            let mesh = &model.mesh;

            let positions = mesh
                .positions
                .chunks(3)
                .map(|pos| glm::make_vec3(pos))
                .collect::<Vec<_>>();

            let uvs = mesh
                .texcoords
                .chunks(2)
                .map(|uv| glm::make_vec2(uv))
                .collect::<Vec<_>>();

            let normals = mesh
                .normals
                .chunks(3)
                .map(|normal| glm::make_vec3(normal))
                .collect::<Vec<_>>();

            let (tangents, bitangents) =
                primitives::compute_tangent(&mesh.indices, &positions, &uvs);

            use itertools::izip;
            let vertices = izip!(positions, uvs, normals, tangents, bitangents)
                .map(
                    |(position, texture_coords, normal, tangent, bitangent)| Vertex {
                        position,
                        normal,
                        texture_coords,
                        tangent,
                        bitangent,
                    },
                )
                .collect::<Vec<_>>();

            // let texture_coords = match mesh.get_texture_coord(0, i) {
            //     None => glm::vec2(0., 0.),
            //     Some(coords) => glm::vec2(coords.x, coords.y),
            // };
            //
            //
            //
            // model.mesh.
            //
        }

        // importer.flip_uvs(true);
        // importer.calc_tangent_space(|args| { args.enable = true });
        //
        // let scene = importer.read_file(path)?;
        // processNode(scene.root_node());
    }
    //
    // fn process_node(node: &assimp::Node, scene: &assimp::Scene) -> Vec<Mesh> {
    //     let mut meshes = vec!();
    //
    //     for mesh_id in 0..node.num_meshes() {
    //         let mesh = scene.mesh(mesh_id as usize)?;
    //         meshes.push(process_mesh(&mesh, scene));
    //
    //     }
    //
    //     meshes
    // }
    //
    // fn process_mesh(mesh: &assimp::Mesh, scene: &assimp::Scene) -> Mesh {
    //     let mut vertices = vec!();
    //     let mut indices = vec!();
    //     let mut textures = vec!();
    //
    //     let to_glm = |vec: assimp::Vector3D| { glm::vec3(vec.x, vec.y, vec.z) };
    //
    //     // vertices
    //     for i in 0..mesh.num_vertices() {
    //         let texture_coords = match mesh.get_texture_coord(0, i) {
    //             None => glm::vec2(0., 0.),
    //             Some(coords) => glm::vec2(coords.x, coords.y),
    //         };
    //
    //         vertices.push(Vertex {
    //             position: to_glm(mesh.get_vertex(i)?),
    //             normal: to_glm(mesh.get_normal(i)?),
    //             texture_coords,
    //             tangent: to_glm(mesh.get_tangent(i)?),
    //             bitangent: to_glm(mesh.get_bitangent(i)?)
    //         })
    //     }
    //
    //     // indices
    //     for i in 0..mesh.num_faces() {
    //         let face = mesh.get_face(i)?;
    //
    //         for j in 0..face.num_indices {
    //             indices.push(face.indices[j]);
    //         }
    //     }
    //
    //     // materials
    //     if mesh.material_index >= 0 {
    //         let material: assimp::AiMaterial = scene.materials[mesh.material_index];
    //
    //         // diffuse
    //         assimp::Material::
    //     }
    //
    //
    //     Mesh {
    //
    //     }
    // }

    // pub fn
}

impl<'a> Mesh<'a> {
    // pub fn new(gl: &gl::GlPtr, vertices: Vec<Vertex>, indices: Vec<GlInt>, textures: Vec<&'a Texture>) -> Self {
    //     let mut vao: GlInt = 0;
    //     let mut vbo: GlInt = 0;
    //     let mut ebo: GlInt = 0;
    //
    //     vao = setup_vao(&gl, || {
    //         vbo = setup_vbo(&gl, &vertices);
    //         ebo = setup_ebo(&gl, &indices);
    //     });
    //
    //     Self {
    //         gl: gl.clone(),
    //         vao,
    //         vbo,
    //         ebo,
    //         vertices,
    //         indices,
    //         // textures
    //     }
    // }

    pub fn draw(&self, shader: &Program) {
        // Bind textures
        let mut diffuse_number = 1;
        let mut specular_number = 1;
        let mut normal_number = 1;
        let mut height_number = 1;

        // for (i, texture) in self.textures.iter().enumerate() {
        //     unsafe {
        //         self.gl.ActiveTexture(gl::TEXTURE0 + i as u32);
        //
        //         match texture.kind {
        //             TextureKind::Diffuse => { diffuse_number += 1 },
        //             TextureKind::Specular => { specular_number += 1 },
        //             TextureKind::Normal => { normal_number += 1 },
        //             TextureKind::Height => { height_number += 1 },
        //         }
        //
        //         shader.setInt(i as i32, &format!("{}{}", texture.kind.as_str(), i));
        //         texture.bind();
        //     }
        // }

        // Draw
        unsafe {
            self.gl.BindVertexArray(self.vao);
            self.gl.DrawElements(
                gl::TRIANGLES,
                self.indices.len() as gl::types::GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
            self.gl.BindVertexArray(0);
        }

        // Cleanup
        // for (i, texture) in self.textures.iter().enumerate() {
        //     unsafe {
        //         self.gl.ActiveTexture(gl::TEXTURE0 + i as u32);
        //         self.gl.BindTexture(gl::TEXTURE_2D, 0);
        //     }
        // }
    }
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
fn setup_vbo(gl: &gl::GlPtr, vertices: &Vec<Vertex>) -> GlInt {
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

        setup_vertex_attrib(
            &gl,
            &[
                3, // position
                3, // normal
                2, // texture_coords
                3, // tangent
                3, // bitangent
            ],
        );

        gl.BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    vbo
}

/* indicies */
fn setup_ebo(gl: &gl::GlPtr, indices: &Vec<GlInt>) -> GlInt {
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
