extern crate nalgebra_glm as glm;
extern crate sdl2;
extern crate stb_image;

use gl;
// use imgui::*;

use crate::camera::{Camera, CameraMovement};
use crate::cube::{Line2D, Ray};
use crate::texture::{Texture, TextureKind};
use crate::utilities::{is_point_on_line2D, is_rays_intersect};
use glm::translate;
use std::env::current_dir;
use std::ptr::null;
use std::rc::Rc;
use std::time::SystemTime;

mod camera;
mod cube;
mod debug;
mod primitives;
mod render_gl;
mod sphere;
mod texture;
mod utilities;

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

    pub fn get_prev(&self, scene_buffer: &SceneBuffer) -> &T {
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
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    sdl.mouse().capture(true);

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5);

    // antialiasing
    gl_attr.set_multisample_buffers(1);
    gl_attr.set_multisample_samples(8);

    // Camera
    let mut camera = Camera::new();

    let window = video_subsystem
        .window("Game", camera.screen_width, camera.screen_height)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let gl: gl::GlPtr = Rc::new(gl::Gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
    }));

    unsafe {
        gl.Enable(gl::DEPTH_TEST);
        gl.DepthMask(gl::TRUE);

        gl.Enable(gl::CULL_FACE);
        // gl.FrontFace(gl::CW);

        gl.Enable(gl::MULTISAMPLE);
    }

    /////////////////////////////////////
    let debug = debug::Debug::new(&gl);
    texture::Texture::init(&gl); // anisotropic

    let diffuse_texture = Texture::from(&gl, "res/wall.jpg").expect("Cannot load texture");
    let diffuse_texture2 = Texture::from(&gl, "res/dirt.png").expect("Cannot load texture");
    let render_cube = primitives::build_cube(&gl, vec![(&diffuse_texture, TextureKind::Diffuse)]);
    let render_pyramid = primitives::build_pyramid(&gl);
    let render_grid = primitives::build_grid(&gl, 30);
    let render_sphere =
        primitives::build_sphere(&gl, vec![(&diffuse_texture, TextureKind::Diffuse)]);

    let basic_shader = render_gl::Program::from_files(
        &gl,
        include_str!("shaders/basic/basic.vert"),
        include_str!("shaders/basic/basic.frag"),
    )
    .unwrap();

    let screen_shader = render_gl::Program::from_files(
        &gl,
        include_str!("shaders/screen/screen.vert"),
        include_str!("shaders/screen/screen.frag"),
    )
    .unwrap();

    let color_shader = render_gl::Program::from_files(
        &gl,
        include_str!("shaders/color/color.vert"),
        include_str!("shaders/color/color.frag"),
    )
    .unwrap();

    /* Lights Rendering */
    type GlInt = gl::types::GLuint;

    // Framebuffer
    let mut fbo: GlInt = 0;
    let screen_texture = Texture::new(&gl, camera.screen_width, camera.screen_height);
    unsafe {
        gl.GenFramebuffers(1, &mut fbo);
        gl.BindFramebuffer(gl::FRAMEBUFFER, fbo);

        gl.FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            screen_texture.id,
            0,
        );

        gl.BindFramebuffer(gl::FRAMEBUFFER, 0); // Added by me
    }

    // Renderbuffer
    let mut rbo: GlInt = 0;
    unsafe {
        gl.GenRenderbuffers(1, &mut rbo);
        gl.BindRenderbuffer(gl::RENDERBUFFER, rbo);

        gl.RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH24_STENCIL8,
            camera.screen_width as gl::types::GLint,
            camera.screen_height as gl::types::GLint,
        );

        // attaching, also added by me
        gl.BindFramebuffer(gl::FRAMEBUFFER, fbo);
        gl.FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_STENCIL_ATTACHMENT,
            gl::RENDERBUFFER,
            rbo,
        );
        gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
        // end - attaching

        gl.BindRenderbuffer(gl::RENDERBUFFER, 0);
    }

    // Check if framebuffer is complete
    unsafe {
        gl.BindFramebuffer(gl::FRAMEBUFFER, fbo);
        if gl.CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("Framebuffer is not complete!")
        }
        gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    /* Projection quad */
    let render_quad = primitives::build_quad(&gl, vec![(&screen_texture, TextureKind::Diffuse)]);

    unsafe {
        gl.Viewport(
            0,
            0,
            camera.screen_width as gl::types::GLint,
            camera.screen_height as gl::types::GLint,
        );
        gl.ClearColor(0.1, 0.1, 0.1, 1.);
    }

    /////////////////////////////////////

    // Cubes
    let mut scene_buffer = SceneBuffer::new();
    let mut cubes: Vec<DoubleBuffered<TransformComponent>> = vec![];
    // cubes.push(DoubleBuffered::new(TransformComponent {
    //     position: glm::vec3(0., 2., 0.),
    //     rotation: glm::quat_identity(),
    //     scale: glm::vec3(0.1, 0.1, 0.1),
    // }));
    // cubes.push(DoubleBuffered::new(TransformComponent {
    //     position: glm::vec3(0., 0., 0.),
    //     rotation: glm::quat_identity(),
    //     scale: glm::vec3(1., 1., 1.),
    // }));
    cubes.push(DoubleBuffered::new(TransformComponent {
        position: glm::vec3(0., 0.5, 0.),
        rotation: glm::quat_identity(),
        scale: glm::vec3(1., 1., 1.),
    }));
    // cubes.push(DoubleBuffered::new(TransformComponent {
    //     position: glm::vec3(2., 1.5, 0.),
    //     rotation: glm::quat_identity(),
    //     scale: glm::vec3(1., 1., 1.),
    // }));
    // cubes.push(DoubleBuffered::new(TransformComponent {
    //     position: glm::vec3(0., -1., 0.),
    //     rotation: glm::quat_identity(),
    //     scale: glm::vec3(100., 1., 100.),
    // }));

    let mut rays = vec![];

    // dragging gizmo
    let mut is_dragging = false;
    let mut drag_start = glm::vec2(0, 0);
    let mut drag_end = glm::vec2(0, 0);

    let mut last_cursor_coords = glm::vec2(0 as i32, 0 as i32);
    let mut is_camera_movement = false;

    let mut cam_sensitive = 0.1;
    let mut yaw = 0.0; // y
    let mut pitch = 0.0; // x

    // Time
    let s_per_update = 1.0 / 30.0;
    let mut previous = SystemTime::now();
    let mut lag = 0.0;

    // Events
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
                    keycode: Some(sdl2::keyboard::Keycode::Q),
                    ..
                } => {
                    // let ray = (camera_pos.clone(), &camera_pos - dir * t);
                    // rays.push(ray);
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::D),
                    ..
                } => {
                    camera.set_move_x(CameraMovement::POSITIVE);
                }
                sdl2::event::Event::KeyUp {
                    keycode: Some(sdl2::keyboard::Keycode::D),
                    ..
                } => {
                    match camera.move_x {
                        CameraMovement::POSITIVE => camera.set_move_x(CameraMovement::NONE),
                        _ => {}
                    };
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::A),
                    ..
                } => {
                    camera.set_move_x(CameraMovement::NEGATIVE);
                }
                sdl2::event::Event::KeyUp {
                    keycode: Some(sdl2::keyboard::Keycode::A),
                    ..
                } => {
                    match camera.move_x {
                        CameraMovement::NEGATIVE => camera.set_move_x(CameraMovement::NONE),
                        _ => {}
                    };
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::W),
                    ..
                } => {
                    camera.set_move_z(CameraMovement::POSITIVE);
                }
                sdl2::event::Event::KeyUp {
                    keycode: Some(sdl2::keyboard::Keycode::W),
                    ..
                } => {
                    match camera.move_z {
                        CameraMovement::POSITIVE => camera.set_move_z(CameraMovement::NONE),
                        _ => {}
                    };
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::S),
                    ..
                } => {
                    camera.set_move_z(CameraMovement::NEGATIVE);
                }
                sdl2::event::Event::KeyUp {
                    keycode: Some(sdl2::keyboard::Keycode::S),
                    ..
                } => {
                    match camera.move_z {
                        CameraMovement::NEGATIVE => camera.set_move_z(CameraMovement::NONE),
                        _ => {}
                    };
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Space),
                    ..
                } => {
                    camera.set_move_y(CameraMovement::POSITIVE);
                }
                sdl2::event::Event::KeyUp {
                    keycode: Some(sdl2::keyboard::Keycode::Space),
                    ..
                } => {
                    match camera.move_y {
                        CameraMovement::POSITIVE => camera.set_move_y(CameraMovement::NONE),
                        _ => {}
                    };
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::LCtrl),
                    ..
                } => {
                    camera.set_move_y(CameraMovement::NEGATIVE);
                }
                sdl2::event::Event::KeyUp {
                    keycode: Some(sdl2::keyboard::Keycode::LCtrl),
                    ..
                } => {
                    match camera.move_y {
                        CameraMovement::NEGATIVE => camera.set_move_y(CameraMovement::NONE),
                        _ => {}
                    };
                }
                sdl2::event::Event::MouseButtonDown {
                    mouse_btn: sdl2::mouse::MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    // Creating line
                    let ray = Ray::new(&glm::vec3(0., 0.5, 0.), &glm::vec3(1., 0., 0.));
                    let line = camera.line_from_ray(&ray, 1.0);

                    if is_point_on_line2D(&line, &camera.cursor_to_screen(&glm::vec2(x, y)), 1.0) {
                        drag_start = glm::vec2(x, y);
                        drag_end = drag_start.clone();

                        is_dragging = true;
                    }

                    ()
                }
                sdl2::event::Event::MouseButtonDown {
                    mouse_btn: sdl2::mouse::MouseButton::Right,
                    x,
                    y,
                    ..
                } => {
                    sdl.mouse().show_cursor(false);
                    is_camera_movement = true;
                    last_cursor_coords = glm::vec2(x, y);
                }
                sdl2::event::Event::MouseButtonUp {
                    mouse_btn: sdl2::mouse::MouseButton::Right,
                    ..
                } => {
                    sdl.mouse().show_cursor(true);
                    is_camera_movement = false;
                }
                sdl2::event::Event::MouseButtonUp {
                    mouse_btn: sdl2::mouse::MouseButton::Left,
                    ..
                } => {
                    drag_start = glm::vec2(0, 0);
                    drag_end = drag_start.clone();
                    is_dragging = false;
                }
                sdl2::event::Event::MouseMotion { x, y, .. } => {
                    if is_camera_movement {
                        let x_offset = x - last_cursor_coords[0];
                        let y_offset = last_cursor_coords[1] - y;

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

                        camera.set_direction(direction.normalize());
                    }

                    if is_dragging {
                        drag_end = glm::vec2(x, y);
                    }

                    last_cursor_coords = glm::vec2(x, y);
                }
                _ => {}
            }
        }

        'logic: loop {
            if lag < s_per_update {
                break 'logic;
            }

            camera.update();

            // update
            lag -= s_per_update;
        }

        let alpha: f32 = lag / s_per_update;

        // ************************* RENDERING **********************8**
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // 1. Drawing on added offscreen framebuffer (with depth and stencil)
        unsafe {
            gl.BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl.ClearColor(0.1, 0.1, 0.1, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl.Enable(gl::DEPTH_TEST);

            gl.Enable(gl::CULL_FACE);
            gl.FrontFace(gl::CW);
        }

        // Render to offscreen buffer
        basic_shader.bind();
        basic_shader.setMat4(&camera.projection, "projection");
        basic_shader.setMat4(&camera.view, "view");

        let drawer = debug.setup_drawer(&camera.view, &camera.projection);

        for cube in &cubes {
            let transform = cube.get(&scene_buffer);

            let plane_normal = glm::vec3(0., 1., 0.);
            let p1 = camera.cast_cursor_on_plane(&drag_start, &plane_normal);
            let p2 = camera.cast_cursor_on_plane(&drag_end, &plane_normal);

            let diff = p2 - p1;

            let axis = &glm::vec3(1.0, 0., 0.);
            let offset_proj_length = glm::dot(&diff, &axis);
            let offset_proj = axis * offset_proj_length;

            let mut result_mat4;

            if is_dragging {
                result_mat4 = glm::translate(&transform.get_mat4(), &offset_proj);
            } else {
                result_mat4 = transform.get_mat4();
            }

            basic_shader.setMat4(&result_mat4, "model");
            render_cube.draw(&basic_shader);
        }

        // gizmos
        unsafe {
            gl.Disable(gl::DEPTH_TEST);
        }
        for cube in &cubes {
            let transform = cube.get(&scene_buffer);

            let plane_normal = glm::vec3(0., 1., 0.);
            let p1 = camera.cast_cursor_on_plane(&drag_start, &plane_normal);
            let p2 = camera.cast_cursor_on_plane(&drag_end, &plane_normal);

            let diff = p2 - p1;

            let axis = &glm::vec3(1.0, 0., 0.);
            let offset_proj_length = glm::dot(&diff, &axis);
            let offset_proj = &transform.position + axis * offset_proj_length;

            drawer.draw_gizmo(&offset_proj, 1., 1.);
        }
        unsafe {
            gl.Enable(gl::DEPTH_TEST);
        }

        let mut sphere_model = glm::translate(&glm::one(), &glm::vec3(0., 0., 0.));
        sphere_model *= glm::scaling(&glm::vec3(1., 1., 1.));

        unsafe {
            gl.FrontFace(gl::CCW);
        }

        color_shader.bind();
        color_shader.setMat4(&camera.projection, "projection");
        color_shader.setMat4(&camera.view, "view");
        color_shader.setMat4(&sphere_model, "model");
        color_shader.setVec4Float(&glm::vec4(1., 1., 1., 0.5), "color");

        // for &(from, to) in &rays {
        //     drawer.draw(&from, &to);
        // }

        for ray in &rays {
            drawer.draw_ray(ray, 50.);
        }

        // Grid
        let mut grid_model = glm::translate(&glm::one(), &glm::vec3(0., 0., 0.));
        grid_model *= glm::scaling(&glm::vec3(5., 5., 5.));

        color_shader.bind();
        color_shader.setMat4(&camera.projection, "projection");
        color_shader.setMat4(&camera.view, "view");
        color_shader.setMat4(&grid_model, "model");
        color_shader.setVec4Float(&glm::vec4(1., 1., 1., 0.1), "color");
        render_grid.draw_lines(1.);

        drawer.draw_color(
            &glm::vec3(0., 0., -5.),
            &glm::vec3(0., 0., 5.),
            &glm::vec4(0.2, 0.2, 0.2, 1.0),
            1.,
        );

        drawer.draw_color(
            &glm::vec3(-5., 0., 0.),
            &glm::vec3(5., 0., 0.),
            &glm::vec4(0.2, 0.2, 0.2, 1.0),
            1.,
        );

        // gui
        let test = glm::unproject(
            &glm::vec3(0.05, 0.05, 0.5),
            &camera.view,
            &camera.projection,
            glm::vec4(0., 0., 1., 1.),
        );
        drawer.draw_gizmo(&test, 0.01, 0.5);

        // 2. Clear main framebuffer
        unsafe {
            gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl.ClearColor(1., 1., 1., 1.);
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }

        unsafe {
            gl.Disable(gl::CULL_FACE);
            gl.Disable(gl::DEPTH_TEST);
        }

        screen_shader.bind();
        render_quad.draw(&screen_shader);

        // ************************* RENDERING **********************8**

        window.gl_swap_window();
    }
}
