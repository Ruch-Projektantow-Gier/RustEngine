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

type Mat4 = glm::Mat4;
type Vec3f = glm::Vec3;

struct SceneBuffer {
    is_first: bool,
}

impl SceneBuffer {
    pub fn new() -> Self {
        SceneBuffer { is_first: true }
    }

    pub fn swap(&mut self) {
        self.is_first = !self.is_first;
    }
}

struct DoubleBuffered<T> {
    first: T,
    second: T,
}

impl<T: std::clone::Clone> DoubleBuffered<T> {
    pub fn new(obj: T) -> Self {
        DoubleBuffered {
            first: obj.clone(),
            second: obj,
        }
    }

    pub fn get(&self, scene_buffer: &SceneBuffer) -> &T {
        if scene_buffer.is_first {
            &self.first
        } else {
            &self.second
        }
    }

    pub fn set(&mut self, val: T, scene_buffer: &SceneBuffer) {
        if scene_buffer.is_first {
            self.first = val
        } else {
            self.second = val
        }
    }

    pub fn get_last(&self, scene_buffer: &SceneBuffer) -> &T {
        if !scene_buffer.is_first {
            &self.first
        } else {
            &self.second
        }
    }
}

#[derive(Clone)]
struct TransformComponent {
    position: Vec3f,
    rotation: glm::Quat,
    scale: Vec3f,
}

impl TransformComponent {
    fn get_mat4(&self) -> Mat4 {
        &glm::one::<Mat4>()
            * glm::translation(&self.position)
            * glm::scaling(&self.scale)
            * glm::quat_to_mat4(&self.rotation)
    }

    fn interpolate(&self, transform: &TransformComponent, alpha: f32) -> TransformComponent {
        TransformComponent {
            position: glm::lerp(&self.position, &transform.position, alpha),
            rotation: glm::quat_slerp(&self.rotation, &transform.rotation, alpha),
            scale: glm::lerp(&self.scale, &transform.scale, alpha),
        }
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

    // Render
    let debug = debug::Debug::new(&gl);
    texture::Texture::init(&gl); // anisotropic

    let texture = texture::Texture::new(&gl, "res/wall.jpg").unwrap();
    // let texture = texture::Texture::new(&gl, "res/dirt.png").unwrap();
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

    let mut scene_buffer = SceneBuffer::new();
    let mut cubes: Vec<DoubleBuffered<TransformComponent>> = vec![];
    cubes.push(DoubleBuffered::new(TransformComponent {
        position: glm::vec3(0., 0., 0.),
        rotation: glm::quat_identity(),
        scale: glm::vec3(1., 1., 1.),
    }));

    let mut candidate: Option<glm::Vec3> = None;

    // props
    let s_per_update = 1.0 / 30.0;
    let mut previous = SystemTime::now();
    let mut lag = 0.0;

    let camera_speed = 0.1;
    let mut camera_pos = glm::vec3(0., 0., 3.);
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
        lag += elapsed.as_secs_f32();

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
                // sdl2::event::Event::MouseButtonDown {
                //     mouse_btn: sdl2::mouse::MouseButton::Left,
                //     ..
                // } => {
                //     match &candidate {
                //         Some(pos) => {
                //             cubes.push(pos * cube::CUBE_SIZE);
                //         }
                //         None => {}
                //     }
                //
                //     ()
                // }
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

            // Cubes
            for transforms in &mut cubes {
                let transform = transforms.get(&scene_buffer);

                transforms.set(
                    TransformComponent {
                        position: transform.position,
                        // rotation: transform.rotation + glm::vec3(0., 0.3, 0.),
                        rotation: glm::quat_rotate(
                            &transform.rotation,
                            0.2,
                            &glm::vec3(0., 1., 0.),
                        ),
                        scale: transform.scale,
                    },
                    &scene_buffer,
                );
            }
            scene_buffer.swap();

            // update
            lag -= s_per_update;
        }

        let alpha: f32 = lag / s_per_update;

        unsafe {
            gl.Enable(gl::CULL_FACE);
            gl.CullFace(gl::BACK);
            gl.FrontFace(gl::CW);

            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // MATRIXES
        let view = glm::look_at(&camera_pos, &(&camera_pos + &camera_front), &camera_up);
        let proj = glm::perspective((width / height) as f32, 45.0, 0.1, 100.0);
        let debug_drawer = debug.setup_drawer(&view, &proj);

        // RENDER LINE
        // let line_origin = &glm::vec3(-1.0, 0., 0.);
        // let line_dest = &glm::vec3(1.0, 0., 0.);

        let line_origin = &camera_pos;
        let mut line_dest = &(&camera_pos + &camera_front * 3.);

        // debug_drawer.draw(&line_dest, &line_origin);
        // debug_drawer.draw(&glm::vec3(-1.0, -1., 0.), &line_origin);
        // debug_drawer.draw(&glm::vec3(1.0, 1., 0.), &line_origin);

        // RENDER BOX
        for cube in &cubes {
            // println!("{}", alpha);

            let transform = cube
                .get(&scene_buffer)
                .interpolate(cube.get_last(&scene_buffer), alpha);

            // let transform = cube.get(&scene_buffer);

            // Ray cast
            let ray = cube::Ray::new(&line_origin, &(line_dest - line_origin).normalize());
            let cube = cube::Cube::new(&transform.position);

            let is_intersect = cube.is_intersect(&ray);
            if is_intersect {
                let (.., normal) = cube.get_intersect_face(&ray);

                debug_drawer.draw(
                    &(&transform.position + &normal * cube::CUBE_HALF_SIZE),
                    &(&transform.position + &normal * cube::CUBE_HALF_SIZE * 1.7),
                );

                candidate = Some(&transform.position + normal);
            }

            // Render
            shader_program.bind();
            texture.bind();

            shader_program.setMat4(&view, "view");
            shader_program.setMat4(&proj, "projection");
            shader_program.setMat4(&transform.get_mat4(), "model");

            render_cube.draw();
        }

        window.gl_swap_window();
    }
}
