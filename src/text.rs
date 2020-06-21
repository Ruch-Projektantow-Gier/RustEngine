use crate::camera::Camera;
use crate::shader;
use crate::shader::Program;
use freetype::Library;
use gl::GlPtr;
use std::ffi::OsStr;
use std::mem;

type GlInt = gl::types::GLuint;

struct Character {
    texture_id: u32,
    size: glm::TVec2<i32>,
    bearing_offset: glm::TVec2<i32>,
    advance_offset: i64,
}

pub struct Font {
    gl: gl::GlPtr,
    lib: freetype::Library,
    shader: Program,
    vao: GlInt,
    vbo: GlInt,
}

pub struct FontRenderer<'a> {
    base: &'a Font,
    characters: Vec<Character>,
}

impl Font {
    pub fn new(gl: &GlPtr) -> Self {
        let shader = shader::Program::from_files(
            &gl,
            include_str!("shaders/text/text.vert"),
            include_str!("shaders/text/text.frag"),
        )
        .unwrap();

        let (vao, vbo) = Self::setup_vbo(&gl);

        Self {
            gl: gl.clone(),
            lib: Library::init().unwrap(),
            shader,
            vao,
            vbo,
        }
    }

    pub fn load<P>(&self, path: P, size: u32) -> FontRenderer
    where
        P: AsRef<OsStr>,
    {
        let face = self.lib.new_face(path, 0).unwrap();
        face.set_pixel_sizes(0, size).unwrap();

        let mut characters: Vec<Character> = vec![];

        unsafe {
            self.gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            for i in 0..128 {
                face.load_char(i, freetype::face::LoadFlag::RENDER).unwrap();
                let glyph = face.glyph();

                let mut texture_id: u32 = 0;

                self.gl.GenTextures(1, &mut texture_id);
                self.gl.BindTexture(gl::TEXTURE_2D, texture_id);
                self.gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RED as i32,
                    glyph.bitmap().width() as i32,
                    glyph.bitmap().rows() as i32,
                    0,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    glyph.bitmap().buffer().as_ptr() as *const gl::types::GLvoid,
                );

                self.gl
                    .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                self.gl
                    .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                self.gl
                    .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                self.gl
                    .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

                characters.push(Character {
                    texture_id,
                    size: glm::vec2(glyph.bitmap().width(), glyph.bitmap().rows()),
                    bearing_offset: glm::vec2(glyph.bitmap_left(), glyph.bitmap_top()),
                    advance_offset: glyph.advance().x,
                })
            }
        }

        FontRenderer {
            base: self,
            characters,
        }
    }

    fn setup_vbo(gl: &gl::GlPtr) -> (GlInt, GlInt) {
        let mut vao: GlInt = 0;
        let mut vbo: GlInt = 0;

        unsafe {
            gl.GenVertexArrays(1, &mut vao);
            gl.GenBuffers(1, &mut vbo);

            gl.BindVertexArray(vao);
            // vbo

            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (6 * 4 * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // quad has 6 vertices with 4 floats
                std::ptr::null() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW,
            );

            gl.EnableVertexAttribArray(0);
            gl.VertexAttribPointer(
                0,
                4,
                gl::FLOAT,
                gl::FALSE,
                4 * std::mem::size_of::<f32>() as gl::types::GLint, // stride
                std::ptr::null() as *const gl::types::GLvoid,       // offset of first component
            );

            // end vbo
            gl.BindVertexArray(0);

            // It's needs to be unbinded after vao
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        (vao, vbo)
    }
}

impl<'a> FontRenderer<'a> {
    pub fn get_width(&self, str: &str, scale: f32) -> f32 {
        let mut width = 0.;

        for i in str.chars() {
            let character = self.characters.get(i as usize).expect("No character found");
            width += (character.advance_offset >> 6) as f32 * scale // bitshift by 6 to get value in pixels (2^6 = 64)
        }

        width
    }

    pub fn render(
        &self,
        camera: &Camera,
        str: &str,
        x: f32,
        y: f32,
        scale: f32,
        color: &glm::Vec3,
    ) {
        let base = self.base;
        let gl = &self.base.gl;

        base.shader.bind();

        let projection = glm::ortho(
            0.,
            camera.screen_width as f32,
            0.,
            camera.screen_height as f32,
            0.,
            1.,
        );
        base.shader.setMat4(&projection, "projection");
        base.shader.setVec3Float(&color, "textColor");

        let mut x = x;

        unsafe {
            gl.Enable(gl::BLEND);
            gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindVertexArray(base.vao);

            for i in str.chars() {
                let character = self.characters.get(i as usize).expect("No character found");

                let x_pos = x + character.bearing_offset.x as f32 * scale;
                let y_pos = y - (character.size.y - character.bearing_offset.y) as f32 * scale;
                let w = character.size.x as f32 * scale;
                let h = character.size.y as f32 * scale;

                let vertices = [
                    [x_pos, y_pos + h, 0., 0.],
                    [x_pos, y_pos, 0., 1.],
                    [x_pos + w, y_pos, 1., 1.],
                    [x_pos, y_pos + h, 0., 0.],
                    [x_pos + w, y_pos, 1., 1.],
                    [x_pos + w, y_pos + h, 1., 0.],
                ];

                gl.BindTexture(gl::TEXTURE_2D, character.texture_id);
                gl.BindBuffer(gl::ARRAY_BUFFER, base.vbo);
                gl.BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (6 * 4 * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                    vertices.as_ptr() as *const gl::types::GLvoid,
                );
                gl.BindBuffer(gl::ARRAY_BUFFER, 0);

                gl.DrawArrays(gl::TRIANGLES, 0, 6);

                x += (character.advance_offset >> 6) as f32 * scale // bitshift by 6 to get value in pixels (2^6 = 64)
            }

            gl.BindVertexArray(0);
            gl.BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn render_with_shadow<F>(
        &self,
        camera: &Camera,
        str: &str,
        pos: F,
        scale: f32,
        color: &glm::Vec3,
    ) where
        F: Fn(f32) -> (f32, f32),
    {
        let width = self.get_width(str, scale);
        let (x, y) = pos(width);

        self.render(
            &camera,
            str,
            x + 1.,
            y - 1.,
            scale,
            &glm::vec3(0.5, 0.5, 0.5),
        );
        self.render(&camera, str, x, y, scale, color);
    }
}
