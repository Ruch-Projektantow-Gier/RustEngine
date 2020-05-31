extern crate nalgebra_glm as glm;
extern crate sdl2;
extern crate stb_image;

use gl;
// use imgui::*;

use std::rc::Rc;
use std::time::SystemTime;

mod cube;
mod debug;
mod primitives;
mod render_gl;
mod sphere;
mod texture;

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

    // Render
    let debug = debug::Debug::new(&gl);
    texture::Texture::init(&gl); // anisotropic

    let texture = texture::Texture::new(&gl, "res/wall.jpg").unwrap();
    let render_cube = primitives::build_cube(&gl);
    // let render_sphere = primitives::build_sphere(&gl);

    unsafe {
        gl.Enable(gl::DEPTH_TEST);
    }

    // Shader
    let shader_program = render_gl::Program::from_files(
        &gl,
        include_str!("triangle.vert"),
        include_str!("triangle.frag"),
    )
    .unwrap();

    // props
    let s_per_update = 1.0 / 30.0;
    let mut previous = SystemTime::now();
    let mut lag = 0.0;

    let camera_speed = 0.1;
    let mut camera_pos = glm::vec3(0., 0., 1.);
    let mut camera_front = glm::vec3(0., 0., -1.);
    let mut camera_up = glm::vec3(0., 1., 0.);
    let mut camera_movement = glm::vec2(0, 0);

    let mut last_cursor_coords = glm::vec2(0 as i32, 0 as i32);

    let mut cam_sensitive = 0.1;
    let mut yaw = 0.0; // y
    let mut pitch = 0.0; // x

    let mut is_camera_movement = false;

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
        let debug_drawer = debug.setup_drawer(&view, &proj);

        // RENDER LINE
        let line_origin = &glm::vec3(-1.0, 0., 0.);
        let line_dest = &glm::vec3(1.0, 0., 0.);

        // let line_origin = &camera_pos;
        // let mut line_dest = &(&camera_pos + &camera_front);

        debug_drawer.draw(&line_dest, &line_origin);
        debug_drawer.draw(&glm::vec3(-1.0, -1., 0.), &line_origin);
        debug_drawer.draw(&glm::vec3(1.0, 1., 0.), &line_origin);

        // Ray cast
        // let ray = cube::Ray::new(&line_origin, &(line_dest - line_origin).normalize());
        // let cube = cube::Cube::new(&glm::vec3(0., 0., 0.));
        //
        // let is_intersect = cube.is_intersect(&ray);
        // if is_intersect {
        //     println!("Kolizja!");
        //     cube.get_intersect_face(&ray);
        // }

        // RENDER BOX
        shader_program.bind();

        // matrixes
        let mut model = glm::translate(&glm::identity(), &glm::vec3(0., 0., 0.));

        unsafe {
            // cubes
            texture.bind();

            shader_program.setMat4(&view, "view");
            shader_program.setMat4(&proj, "projection");

            let mut i = 0;
            loop {
                i += 1;

                shader_program.setMat4(&model, "model");
                render_cube.draw();
                // render_sphere.draw();

                if i > 0 {
                    break;
                }
            }
        }

        window.gl_swap_window();
    }
}
