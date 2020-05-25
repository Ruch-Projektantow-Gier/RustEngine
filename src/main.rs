extern crate sdl2;
extern crate stb_image;

use gl;
#[allow(unused_imports)]
use stb_image::image::LoadResult;
#[allow(unused_imports)]
use stb_image::stb_image::bindgen::stbi_load_from_file;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

pub mod render_gl;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5);

    // antialiasing
    gl_attr.set_multisample_buffers(1);
    gl_attr.set_multisample_samples(8);

    let window = video_subsystem
        .window("Game", 900, 700)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let gl: gl::GlPtr = Rc::new(gl::Gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
    }));

    unsafe {
        gl.Enable(gl::MULTISAMPLE);
    }

    // Verticles
    let vertices: Vec<f32> = vec![
        // positions      // colors
        0.5, 0.5, 0.0, /* */ 1.0, 0.0, 0.0, /* */ 1.0, 1.0, // top right
        0.5, -0.5, 0.0, /* */ 0.0, 1.0, 0.0, /* */ 1.0, 0.0, // bottom right
        -0.5, -0.5, 0.0, /* */ 0.0, 0.0, 1.0, /* */ 0.0, 0.0, // bottom left
        -0.5, 0.5, 0.0, /* */ 0.5, 0.5, 0.5, /* */ 0.0, 1.0, // top left
    ];

    let indices: Vec<i32> = vec![
        0, 1, 3, //
        1, 2, 3, //
    ];

    unsafe {
        gl.TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            gl::MIRRORED_REPEAT as i32,
        );
        gl.TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            gl::MIRRORED_REPEAT as i32,
        );

        gl.TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    }

    let texture_load_result = stb_image::image::load("res/wall.jpg");
    let mut texture_id: u32 = 0;

    match texture_load_result {
        LoadResult::Error(_) => {}
        LoadResult::ImageU8(image) => {
            let texture_image = image;

            unsafe {
                gl.GenTextures(1, &mut texture_id);
                gl.BindTexture(gl::TEXTURE_2D, texture_id);
                gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGB as i32,
                    texture_image.width as i32,
                    texture_image.height as i32,
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    texture_image.data.as_ptr() as *const gl::types::GLvoid,
                );
                gl.GenerateMipmap(gl::TEXTURE_2D);
            }
        }
        LoadResult::ImageF32(_) => {}
    }

    // vao
    let mut vao: gl::types::GLuint = 0;
    let mut vbo: gl::types::GLuint = 0;
    let mut ebo: gl::types::GLuint = 0;

    unsafe {
        gl.GenVertexArrays(1, &mut vao);
        gl.GenBuffers(1, &mut vbo);
        gl.GenBuffers(1, &mut ebo);
    }

    unsafe {
        gl.BindVertexArray(vao);

        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl.BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );

        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl.BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<i32>()) as gl::types::GLsizeiptr,
            indices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );

        // positions
        gl.EnableVertexAttribArray(0);
        gl.VertexAttribPointer(
            0, // layout (location = 0)
            3,
            gl::FLOAT,
            gl::FALSE,
            (8 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride
            std::ptr::null(),                                     // offset of first component
        );

        // colors
        gl.EnableVertexAttribArray(1);
        gl.VertexAttribPointer(
            1, // layout (location = 1)
            3,
            gl::FLOAT,                                                    // data-type
            gl::FALSE,                                                    // normalized
            (8 * std::mem::size_of::<f32>()) as gl::types::GLint,         // stride
            (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid, // offset
        );

        // texture coords
        gl.EnableVertexAttribArray(2);
        gl.VertexAttribPointer(
            2, // layout (location = 2)
            2,
            gl::FLOAT,                                                    // data-type
            gl::FALSE,                                                    // normalized
            (8 * std::mem::size_of::<f32>()) as gl::types::GLint,         // stride
            (6 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid, // offset
        );

        gl.EnableVertexAttribArray(0);
        gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        gl.BindVertexArray(0);
        // gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
    }

    // Shader
    use std::ffi::CString;
    let vert_shader = render_gl::Shader::from_vert_source(
        &gl,
        &CString::new(include_str!("triangle.vert")).unwrap(),
    )
    .unwrap();

    let frag_shader = render_gl::Shader::from_frag_source(
        &gl,
        &CString::new(include_str!("triangle.frag")).unwrap(),
    )
    .unwrap();

    let shader_program =
        render_gl::Program::from_shaders(&gl, &[vert_shader, frag_shader]).unwrap();


    // unsafe {
    //     gl::Viewport(0, 0, 900, 700);
    //     gl::ClearColor(0.5, 0.5, 0.5, 1.0);
    // }

    let s_per_update = 1.0 / 30.0;
    let mut previous = SystemTime::now();
    let mut lag = 0.0;

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        let current = SystemTime::now();
        let elapsed = current.duration_since(previous).unwrap();
        previous = current;
        lag += elapsed.as_secs_f64();

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        'logic: loop {
            if lag < s_per_update {
                break 'logic
            }

            // update
            lag -= s_per_update;
        }

        let _alpha = lag / s_per_update;

        shader_program.set_used();
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT);

            gl.BindTexture(gl::TEXTURE_2D, texture_id);
            gl.BindVertexArray(vao);
            gl.DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        }

        window.gl_swap_window();
    }
}
