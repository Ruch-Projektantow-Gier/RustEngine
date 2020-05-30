extern crate imgui;
extern crate nalgebra_glm as glm;
extern crate sdl2;
extern crate stb_image;

use gl;
use imgui::*;

#[allow(unused_imports)]
use stb_image::image::LoadResult;
#[allow(unused_imports)]
use stb_image::stb_image::bindgen::stbi_load_from_file;
use std::rc::Rc;
use std::time::SystemTime;

mod cube;
pub mod render_gl;

pub trait FromVec<T> {
    fn from_vec4(x: &glm::Vec3, y: &glm::Vec3, z: &glm::Vec3, origin: &glm::Vec3) -> T;
}

impl FromVec<glm::Mat4> for glm::Mat4 {
    fn from_vec4(x: &glm::Vec3, y: &glm::Vec3, z: &glm::Vec3, origin: &glm::Vec3) -> glm::Mat4 {
        glm::mat4(
            x[0], y[0], z[0], origin[0], //
            x[1], y[1], z[1], origin[1], //
            x[2], y[2], z[2], origin[2], //
            0., 0., 0., 1., //
        )
    }
}

fn main() {
    let width = 900;
    let height = 700;

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    sdl.mouse().capture(true);

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5);

    // antialiasing
    gl_attr.set_multisample_buffers(1);
    gl_attr.set_multisample_samples(8);

    let window = video_subsystem
        .window("Game", width, height)
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
    // plane
    // let vertices: Vec<f32> = vec![
    //     // positions      // colors
    //     0.5, 0.5, 0.0, /* */ 1.0, 0.0, 0.0, /* */ 1.0, 1.0, // top right
    //     0.5, -0.5, 0.0, /* */ 0.0, 1.0, 0.0, /* */ 1.0, 0.0, // bottom right
    //     -0.5, -0.5, 0.0, /* */ 0.0, 0.0, 1.0, /* */ 0.0, 0.0, // bottom left
    //     -0.5, 0.5, 0.0, /* */ 0.5, 0.5, 0.5, /* */ 0.0, 1.0, // top left
    // ];

    // line
    let line: Vec<f32> = vec![
        0., 0., 0., //
        0., 0., 1.,
    ];

    // vao
    let mut vaoLine: gl::types::GLuint = 0;
    let mut vboLine: gl::types::GLuint = 0;

    unsafe {
        gl.GenVertexArrays(1, &mut vaoLine);
        gl.GenBuffers(1, &mut vboLine);
    }

    unsafe {
        gl.BindVertexArray(vaoLine);

        gl.BindBuffer(gl::ARRAY_BUFFER, vboLine);
        gl.BufferData(
            gl::ARRAY_BUFFER,
            (line.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            line.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );

        // positions
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

        gl.Enable(gl::DEPTH_TEST);
    }

    // cube
    let vertices: Vec<f32> = vec![
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

    // let indices: Vec<i32> = vec![
    //     0, 1, 3, //
    //     1, 2, 3, //
    // ];

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

    // anisotropic
    let mut filtering: f32 = 0.;
    unsafe {
        gl.GetFloatv(gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut filtering);
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

        // gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        // gl.BufferData(
        //     gl::ELEMENT_ARRAY_BUFFER,
        //     (indices.len() * std::mem::size_of::<i32>()) as gl::types::GLsizeiptr,
        //     indices.as_ptr() as *const gl::types::GLvoid,
        //     gl::STATIC_DRAW,
        // );

        // positions
        gl.EnableVertexAttribArray(0);
        gl.VertexAttribPointer(
            0, // layout (location = 0)
            3,
            gl::FLOAT,
            gl::FALSE,
            (5 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride
            std::ptr::null(),                                     // offset of first component
        );

        // // colors
        // gl.EnableVertexAttribArray(1);
        // gl.VertexAttribPointer(
        //     1, // layout (location = 1)
        //     3,
        //     gl::FLOAT,                                                    // data-type
        //     gl::FALSE,                                                    // normalized
        //     (8 * std::mem::size_of::<f32>()) as gl::types::GLint,         // stride
        //     (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid, // offset
        // );

        // texture coords
        gl.EnableVertexAttribArray(2);
        gl.VertexAttribPointer(
            2, // layout (location = 2)
            2,
            gl::FLOAT,                                                    // data-type
            gl::FALSE,                                                    // normalized
            (5 * std::mem::size_of::<f32>()) as gl::types::GLint,         // stride
            (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid, // offset
        );

        gl.EnableVertexAttribArray(0);
        gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        gl.BindVertexArray(0);
        // gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

        gl.Enable(gl::DEPTH_TEST);
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

    // Shader Line
    let vert_shader_line =
        render_gl::Shader::from_vert_source(&gl, &CString::new(include_str!("ray.vert")).unwrap())
            .unwrap();

    let frag_shader_line =
        render_gl::Shader::from_frag_source(&gl, &CString::new(include_str!("ray.frag")).unwrap())
            .unwrap();

    let shader_program_line =
        render_gl::Program::from_shaders(&gl, &[vert_shader_line, frag_shader_line]).unwrap();

    // unsafe {
    //     gl::Viewport(0, 0, 900, 700);
    //     gl::ClearColor(0.5, 0.5, 0.5, 1.0);
    // }

    // let cube = cube::Cube::new(0, 0, 0);

    let s_per_update = 1.0 / 30.0;
    let mut previous = SystemTime::now();
    let mut lag = 0.0;

    let model_name = CString::new("model").unwrap();
    let view_name = CString::new("view").unwrap();
    let proj_name = CString::new("projection").unwrap();

    let mut rotation: f32 = 0.0;

    let camera_speed = 0.1;
    let mut camera_pos = glm::vec3(0., 0., 1.);
    let mut camera_front = glm::vec3(0., 0., -1.);
    let mut camera_up = glm::vec3(0., 1., 0.);
    let mut camera_movement = glm::vec2(0, 0);

    let mut last_cursor_coords = glm::vec2(0 as i32, 0 as i32);

    let mut cam_sensitive = 0.1;
    let mut yaw = 0.0; // y
    let mut pitch = 0.0; // x
                         // roll is on z
    let mut is_camera_movement = false;

    // Imgui
    // let mut ui = Context::create();
    // Window::new(im_str!("Hello world"))
    //     .size([300.0, 110.0], Condition::FirstUseEver)
    //     .build(ui, || {
    //         ui.text(im_str!("Hello world!"));
    //         ui.text(im_str!("こんにちは世界！"));
    //         ui.text(im_str!("This...is...imgui-rs!"));
    //         ui.separator();
    //         let mouse_pos = ui.io().mouse_pos;
    //         ui.text(format!(
    //             "Mouse Position: ({:.1},{:.1})",
    //             mouse_pos[0], mouse_pos[1]
    //         ));
    //     });

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        let current = SystemTime::now();
        let elapsed = current.duration_since(previous).unwrap();
        previous = current;
        lag += elapsed.as_secs_f64();

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::D),
                    ..
                } => {
                    camera_movement[0] = 1;
                }
                sdl2::event::Event::KeyUp {
                    keycode: Some(sdl2::keyboard::Keycode::D),
                    ..
                } => {
                    if camera_movement[0] == 1 {
                        camera_movement[0] = 0;
                    }
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::A),
                    ..
                } => {
                    camera_movement[0] = -1;
                }
                sdl2::event::Event::KeyUp {
                    keycode: Some(sdl2::keyboard::Keycode::A),
                    ..
                } => {
                    if camera_movement[0] == -1 {
                        camera_movement[0] = 0;
                    }
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::W),
                    ..
                } => {
                    camera_movement[1] = 1;
                }
                sdl2::event::Event::KeyUp {
                    keycode: Some(sdl2::keyboard::Keycode::W),
                    ..
                } => {
                    if camera_movement[1] == 1 {
                        camera_movement[1] = 0;
                    }
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::S),
                    ..
                } => {
                    camera_movement[1] = -1;
                }
                sdl2::event::Event::KeyUp {
                    keycode: Some(sdl2::keyboard::Keycode::S),
                    ..
                } => {
                    if camera_movement[1] == -1 {
                        camera_movement[1] = 0;
                    }
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Space),
                    ..
                } => {
                    camera_pos += &camera_up * camera_speed;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::LCtrl),
                    ..
                } => {
                    camera_pos -= &camera_up * camera_speed;
                }
                sdl2::event::Event::MouseButtonDown {
                    mouse_btn: sdl2::mouse::MouseButton::Right,
                    x,
                    y,
                    ..
                } => {
                    // sdl.mouse().show_cursor(false);
                    is_camera_movement = true;
                    last_cursor_coords = glm::vec2(x, y);
                }
                sdl2::event::Event::MouseButtonUp {
                    mouse_btn: sdl2::mouse::MouseButton::Right,
                    ..
                } => {
                    // sdl.mouse().show_cursor(true);
                    is_camera_movement = false;
                }
                sdl2::event::Event::MouseMotion { x, y, .. } => {
                    if is_camera_movement {
                        let x_offset = x - last_cursor_coords[0];
                        let y_offset = last_cursor_coords[1] - y;

                        last_cursor_coords = glm::vec2(x, y);

                        yaw += cam_sensitive * x_offset as f32;
                        pitch += cam_sensitive * y_offset as f32;

                        if pitch.abs() > 89.0 {
                            pitch = pitch.signum() * 89.0;
                        }

                        let direction = glm::vec3(
                            yaw.to_radians().cos() * pitch.to_radians().cos(),
                            pitch.to_radians().sin(),
                            yaw.to_radians().sin() * pitch.to_radians().cos(),
                        );

                        camera_front = direction.normalize();
                    }
                }
                _ => {}
            }
        }

        'logic: loop {
            if lag < s_per_update {
                break 'logic;
            }

            camera_pos += glm::normalize(&glm::cross(&camera_front, &camera_up))
                * camera_speed
                * camera_movement[0] as f32;
            camera_pos += &camera_front * camera_speed * camera_movement[1] as f32;

            // update
            lag -= s_per_update;
        }

        let _alpha = lag / s_per_update;

        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // MATRIXES
        let view = glm::look_at(&camera_pos, &(&camera_pos + &camera_front), &camera_up);
        let proj = glm::perspective((width / height) as f32, 45.0, 0.1, 100.0);

        // RENDER LINE
        shader_program_line.set_used();

        let mut model = glm::translate(&glm::identity(), &glm::vec3(0., 0., 0.));

        unsafe {
            gl.LineWidth(4.0);
            gl.BindVertexArray(vaoLine);

            // let line_origin = &glm::vec3(0.7, 0., 0.);
            // let line_dest = &glm::vec3(0.0, 0., 0.);

            let line_origin = &(&camera_pos - &camera_front + &glm::vec3(0., -0.5, 0.));
            let line_dest = &(&camera_pos + &camera_front * 3.);

            let new_z = (line_dest - line_origin).normalize();
            let mut new_y = glm::cross(&new_z, &glm::vec3(0., 0., 1.));

            if new_y.magnitude() < glm::epsilon() {
                new_y = glm::vec3(0., 1., 0.);
            }

            let new_x = glm::cross(&new_z, &new_y).normalize();
            {
                let x = new_x;
                let y = new_y;
                let z = new_z;
                let origin = line_origin;
                let length = (line_dest - line_origin).magnitude();

                model = glm::mat4(
                    x[0] * length,
                    y[0] * length,
                    z[0] * length,
                    origin[0], //
                    x[1] * length,
                    y[1] * length,
                    z[1] * length,
                    origin[1], //
                    x[2] * length,
                    y[2] * length,
                    z[2] * length,
                    origin[2], //
                    0.,
                    0.,
                    0.,
                    1., //
                );
            }

            let ray = cube::Ray::new(&line_origin, &(line_dest - line_origin).normalize());
            let cube = cube::Cube::new(&glm::vec3(0., 0., 0.));

            let is_intersect = cube.is_intersect(&ray);
            if is_intersect {
                rotation += 0.02;
            }

            gl.UniformMatrix4fv(
                gl.GetUniformLocation(shader_program_line.id(), view_name.as_ptr()),
                1,
                gl::FALSE,
                view.as_ptr(),
            );

            gl.UniformMatrix4fv(
                gl.GetUniformLocation(shader_program_line.id(), proj_name.as_ptr()),
                1,
                gl::FALSE,
                proj.as_ptr(),
            );

            gl.UniformMatrix4fv(
                gl.GetUniformLocation(shader_program_line.id(), model_name.as_ptr()),
                1,
                gl::FALSE,
                model.as_ptr(),
            );

            gl.DrawArrays(gl::LINES, 0, 2);
            gl.BindVertexArray(0);
        }

        // RENDER BOX
        shader_program.set_used();

        // matrixes
        let mut model = glm::translate(&glm::identity(), &glm::vec3(0., 0., 0.));
        model = glm::rotate_y(&model, rotation);

        unsafe {
            // cubes
            gl.BindTexture(gl::TEXTURE_2D, texture_id);
            gl.TexParameterf(gl::TEXTURE_2D, gl::TEXTURE_MAX_ANISOTROPY_EXT, filtering);

            gl.BindVertexArray(vao);

            gl.UniformMatrix4fv(
                gl.GetUniformLocation(shader_program.id(), view_name.as_ptr()),
                1,
                gl::FALSE,
                view.as_ptr(),
            );

            gl.UniformMatrix4fv(
                gl.GetUniformLocation(shader_program.id(), proj_name.as_ptr()),
                1,
                gl::FALSE,
                proj.as_ptr(),
            );

            // gl.DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, std::ptr::null());
            let mut i = 0;
            loop {
                i += 1;

                // model = glm::translate(&glm::identity(), &glm::vec3(i as f32 * 1.0, 0.0, 0.0));

                gl.UniformMatrix4fv(
                    gl.GetUniformLocation(shader_program.id(), model_name.as_ptr()),
                    1,
                    gl::FALSE,
                    model.as_ptr(),
                );

                // gl.DrawArrays(gl::LINE_STRIP_ADJACENCY, 0, 36);
                gl.DrawArrays(gl::TRIANGLES, 0, 36);

                if i > 0 {
                    break;
                }
            }

            gl.BindVertexArray(0);
        }

        window.gl_swap_window();
    }
}
