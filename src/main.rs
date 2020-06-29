mod text;

extern crate freetype;
extern crate nalgebra_glm as glm;
extern crate sdl2;
extern crate stb_image;

use gl;

use crate::camera::{Camera, CameraMovement};
use crate::components::TransformComponent;
use crate::cube::{Line2D, Ray};
use crate::double_buffer::{DoubleBuffered, SceneBuffer};
use crate::gizmo::Gizmo;
use crate::text::Font;
use crate::texture::{Texture, TextureKind};
use crate::utilities::{is_point_on_line2D, is_rays_intersect};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Instant, SystemTime};

mod camera;
mod components;
mod cube;
mod debug;
mod double_buffer;
mod gizmo;
mod primitives;
mod shader;
mod sphere;
mod texture;
mod utilities;

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
    let mut gizmo = Gizmo::new();
    texture::Texture::init(&gl); // anisotropic

    let diffuse_texture =
        Texture::from(&gl, "res/test/brickwall.jpg").expect("Cannot load texture");
    let specular_texture =
        Texture::from(&gl, "res/test/brickwall_specular.jpg").expect("Cannot load texture");
    let normal_texture =
        Texture::from(&gl, "res/test/brickwall_normal.jpg").expect("Cannot load texture");
    let height_texture =
        Texture::from(&gl, "res/test/brickwall_height.jpg").expect("Cannot load texture");

    let render_cube = primitives::build_cube(
        &gl,
        vec![
            (&diffuse_texture, TextureKind::Diffuse),
            (&specular_texture, TextureKind::Specular),
            (&normal_texture, TextureKind::Normal),
            (&height_texture, TextureKind::Height),
        ],
        1.0,
        1.0,
    );

    let render_pyramid = primitives::build_pyramid(&gl);
    let render_grid = primitives::build_grid(&gl, 30);
    let render_sphere = primitives::build_sphere(
        &gl,
        vec![
            (&diffuse_texture, TextureKind::Diffuse),
            (&specular_texture, TextureKind::Specular),
            (&normal_texture, TextureKind::Normal),
            (&height_texture, TextureKind::Height),
        ],
    );

    /////////////////////////////////////

    let basic_shader = shader::Program::from_files(
        &gl,
        include_str!("shaders/basic/basic.vert"),
        include_str!("shaders/basic/basic.frag"),
    )
    .unwrap();

    let screen_shader = shader::Program::from_files(
        &gl,
        include_str!("shaders/screen/screen.vert"),
        include_str!("shaders/screen/screen.frag"),
    )
    .unwrap();

    let color_shader = shader::Program::from_files(
        &gl,
        include_str!("shaders/color/color.vert"),
        include_str!("shaders/color/color.frag"),
    )
    .unwrap();

    /////////////////////////////////////
    let font = Font::new(&gl);
    let normal_font = font.load("res/fonts/Corbert-Regular.otf", 28);
    let bold_font = font.load("res/fonts/chinese_rocks.ttf", 28);

    /////////////////////////////////////

    /* Lights Rendering */
    type GlInt = gl::types::GLuint;

    // Framebuffer
    let mut fbo: GlInt = 0;
    let screen_texture_multisampled =
        Texture::new_multisampled(&gl, camera.screen_width, camera.screen_height, 4);

    unsafe {
        gl.GenFramebuffers(1, &mut fbo);
        gl.BindFramebuffer(gl::FRAMEBUFFER, fbo);

        gl.FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D_MULTISAMPLE,
            screen_texture_multisampled.id,
            0,
        );

        gl.BindFramebuffer(gl::FRAMEBUFFER, 0); // Added by me
    }

    // Framebuffer for postprocessing
    let mut fbo_postprocesing: GlInt = 0;
    let screen_texture = Texture::new(&gl, camera.screen_width, camera.screen_height);

    unsafe {
        gl.GenFramebuffers(1, &mut fbo_postprocesing);
        gl.BindFramebuffer(gl::FRAMEBUFFER, fbo_postprocesing);

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

        gl.RenderbufferStorageMultisample(
            gl::RENDERBUFFER,
            4,
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

        // and postprocessing
        gl.BindFramebuffer(gl::FRAMEBUFFER, fbo_postprocesing);
        if gl.CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("Framebuffer postprocessing is not complete!")
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
    let target_cube = TransformComponent::new(
        glm::vec3(0., 0.5, 0.),
        glm::quat_angle_axis(glm::half_pi(), &glm::vec3(1., 0., 0.)),
        glm::vec3(1., 1., 1.),
    );

    let light_cube = TransformComponent::new(
        glm::vec3(0., 2.0, 0.),
        // glm::quat_look_at(&glm::vec3(1., 0., 0.), &glm::vec3(0., 1., 0.)),
        glm::quat_identity(),
        glm::vec3(0.1, 0.1, 0.1),
    );
    let light_cube_ptr = Rc::new(RefCell::new(light_cube));
    let mut light_color = [1., 1., 1.];

    // let cube_ptr = Rc::new(RefCell::new(target_cube));
    let cube_ptr = Rc::new(RefCell::new(target_cube));

    let mut scene_buffer = SceneBuffer::new();
    let mut cubes = vec![];
    cubes.push(cube_ptr.clone());
    // gizmo.target(cube_ptr.clone());
    gizmo.target(light_cube_ptr.clone());

    /////////////////////////////////////
    let mut rays = vec![];
    let mut lines = vec![];

    // Time
    let s_per_update = 1.0 / 30.0;
    let mut previous = Instant::now();
    let mut lag = 0.0;

    // Frames counter
    let mut previous_frame_time = Instant::now();
    let mut frames = 0;
    let mut updates = 0;
    let mut frames_counter = 0;
    let mut updates_counter = 0;

    // Events
    let mut event_pump = sdl.event_pump().unwrap();

    /////////////////////////////////////

    'main: loop {
        let current = Instant::now();
        let elapsed = current.duration_since(previous);
        previous = current;
        lag += elapsed.as_secs_f32();

        // frames counter
        frames += 1;
        if current.duration_since(previous_frame_time).as_secs() >= 1 {
            frames_counter = frames;
            updates_counter = updates;
            frames = 0;
            updates = 0;
            previous_frame_time = current;
        }

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Z),
                    ..
                } => {
                    camera.set_direction(glm::vec3(0., 0., -1.));
                    camera.set_position(glm::vec3(camera.position.x, 0., camera.position.z));
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::R),
                    ..
                } => {
                    light_color[0] -= 0.1;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::G),
                    ..
                } => {
                    light_color[1] -= 0.1;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::B),
                    ..
                } => {
                    light_color[2] -= 0.1;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::T),
                    ..
                } => {
                    light_color = [1., 1., 1.];
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
                    gizmo.click(&camera, x, y);
                }
                sdl2::event::Event::MouseButtonDown {
                    mouse_btn: sdl2::mouse::MouseButton::Right,
                    x,
                    y,
                    ..
                } => {
                    sdl.mouse().show_cursor(false);
                    camera.click(x, y);
                }
                sdl2::event::Event::MouseButtonUp {
                    mouse_btn: sdl2::mouse::MouseButton::Right,
                    ..
                } => {
                    sdl.mouse().show_cursor(true);
                    camera.unclick();
                }
                sdl2::event::Event::MouseButtonUp {
                    mouse_btn: sdl2::mouse::MouseButton::Left,
                    ..
                } => {
                    gizmo.unclick(&camera, |a1, a2| {
                        // lines.push((a1, a2));
                    });
                }
                sdl2::event::Event::MouseMotion { x, y, .. } => {
                    camera.handle_mouse(x, y);
                    gizmo.drag(&camera, x, y);
                }
                _ => {}
            }
        }

        'logic: loop {
            if lag < s_per_update {
                break 'logic;
            }

            camera.update();

            updates += 1;
            // update
            lag -= s_per_update;
        }

        let alpha: f32 = lag / s_per_update;

        // ************************* RENDERING **********************8**
        let bg = utilities::color_from_rgba(172, 196, 191, 1.);

        unsafe {
            gl.ClearColor(bg.x, bg.y, bg.z, bg.w);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // 1. Drawing on added offscreen framebuffer (with depth and stencil)
        unsafe {
            gl.BindFramebuffer(gl::FRAMEBUFFER, fbo);

            gl.ClearColor(bg.x, bg.y, bg.z, bg.w);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl.Enable(gl::DEPTH_TEST);

            // gl.Enable(gl::CULL_FACE);
            gl.Disable(gl::CULL_FACE); // Disable faceculling when drawing normal maps
            gl.FrontFace(gl::CW);
        }

        // render light
        color_shader.bind();
        color_shader.setVec4Float(
            &glm::vec4(light_color[0], light_color[1], light_color[2], 1.),
            "color",
        );
        color_shader.setMat4(&light_cube_ptr.borrow().mat4(), "model");
        render_sphere.draw(&screen_shader);

        // Render to offscreen buffer
        basic_shader.bind();
        basic_shader.setMat4(&camera.projection, "projection");
        basic_shader.setMat4(&camera.view, "view");

        // material
        basic_shader.setVec3Float(&glm::vec3(1.0, 0.5, 0.31), "material.ambient");
        basic_shader.setVec3Float(&glm::vec3(1.0, 0.5, 0.31), "material.diffuse");
        basic_shader.setVec3Float(&glm::vec3(0.5, 0.5, 0.5), "material.specular");
        basic_shader.setFloat(32.0, "material.shininess");
        // basic_shader.setFloat(0.015, "height_scale");
        basic_shader.setFloat(0.03, "height_scale");

        // light
        basic_shader.setVec3Float(&light_cube_ptr.borrow().position, "light.position");
        basic_shader.setVec3Float(&glm::vec3(0.5, 0.5, 0.5), "light.ambient");
        basic_shader.setVec3Float(
            &glm::vec3(light_color[0], light_color[1], light_color[2]),
            "light.diffuse",
        );
        basic_shader.setVec3Float(&glm::vec3(0.5, 0.5, 0.5), "light.specular");
        basic_shader.setVec3Float(&camera.position, "viewPos");

        for cube in &cubes {
            basic_shader.setMat4(&cube.borrow().mat4(), "model");
            render_cube.draw(&basic_shader);
        }

        let drawer = debug.setup_drawer(&camera.view, &camera.projection);
        let floor = TransformComponent::new(
            glm::vec3(0., 0., 0.),
            glm::quat_identity(),
            glm::vec3(5.0, 0.1, 5.0),
        );
        basic_shader.setMat4(&floor.mat4(), "model");
        render_cube.draw(&screen_shader);

        // sphere
        unsafe {
            gl.FrontFace(gl::CCW);
        }

        let sphere = TransformComponent::new(
            glm::vec3(-2., 0.5, 0.),
            // glm::quat_angle_axis(glm::half_pi(), &glm::vec3(0., 0., 0.)),
            glm::quat_identity(),
            glm::vec3(0.5, 0.5, 0.5),
        );
        basic_shader.setMat4(&sphere.mat4(), "model");
        render_sphere.draw(&screen_shader);
        // render_sphere.draw_mesh(1.5);

        // let mut sphere_model = glm::translate(&glm::one(), &glm::vec3(0., 0., 0.));
        // sphere_model *= glm::scaling(&glm::vec3(1., 1., 1.));
        //
        // unsafe {
        //     gl.FrontFace(gl::CCW);
        // }
        //
        // color_shader.bind();
        // color_shader.setMat4(&camera.projection, "projection");
        // color_shader.setMat4(&camera.view, "view");
        // color_shader.setMat4(&sphere_model, "model");
        // color_shader.setVec4Float(&glm::vec4(1., 1., 1., 0.5), "color");

        for ray in &rays {
            drawer.draw_ray(ray, 50.);
        }

        for &(a1, a2) in &lines {
            drawer.draw(&a1, &a2);
        }

        // Grid
        let mut grid_model = glm::translate(&glm::one(), &glm::vec3(0., 0., 0.));
        grid_model *= glm::scaling(&glm::vec3(5., 5., 5.));

        color_shader.bind();
        color_shader.setMat4(&camera.projection, "projection");
        color_shader.setMat4(&camera.view, "view");
        color_shader.setMat4(&grid_model, "model");
        color_shader.setVec4Float(&glm::vec4(1., 1., 1., 0.1), "color");
        render_grid.draw_lines(2.);

        drawer.draw_color(
            &glm::vec3(0., 0.01, -5.),
            &glm::vec3(0., 0.01, 5.),
            &glm::vec4(1.0, 1.0, 1.0, 0.2),
            0.5,
        );

        drawer.draw_color(
            &glm::vec3(-5., 0.01, 0.),
            &glm::vec3(5., 0.01, 0.),
            &glm::vec4(1.0, 1.0, 1.0, 0.2),
            0.5,
        );

        // 2. Clear main framebuffer
        unsafe {
            gl.BindFramebuffer(gl::READ_FRAMEBUFFER, fbo);
            gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, fbo_postprocesing);
            gl.BlitFramebuffer(
                0,
                0,
                camera.screen_width as gl::types::GLint,
                camera.screen_height as gl::types::GLint,
                0,
                0,
                camera.screen_width as gl::types::GLint,
                camera.screen_height as gl::types::GLint,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );

            gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl.ClearColor(1., 1., 1., 1.);
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }

        unsafe {
            gl.Disable(gl::CULL_FACE);
            gl.Disable(gl::DEPTH_TEST);
        }

        screen_shader.bind();
        screen_shader.setVec3Float(
            &glm::vec3(camera.screen_width as f32, camera.screen_height as f32, 0.),
            "resolution",
        );
        render_quad.draw_no_scaled(&screen_shader);

        bold_font.render_with_shadow(
            &camera,
            "RUST ENGINE",
            |width| (camera.screen_width as f32 - width - 30., 50.),
            0.7,
            &glm::vec3(1., 1., 1.),
        );

        normal_font.render_with_shadow(
            &camera,
            "BLOGOKODZIE.PL",
            |width| (camera.screen_width as f32 - width - 30., 30.),
            0.5,
            &glm::vec3(1., 1., 1.),
        );

        normal_font.render_with_shadow(
            &camera,
            format!(
                "X: {:.2} Y: {:.2} Z: {:.2}",
                &camera.position.x, &camera.position.y, &camera.position.z
            )
            .as_ref(),
            |_| (90., 40.),
            0.45,
            &glm::vec3(1., 1., 1.),
        );
        normal_font.render_with_shadow(
            &camera,
            format!("{} FPS | {} UPDATES", frames_counter, updates_counter).as_ref(),
            |_| (90., 20.),
            0.45,
            &glm::vec3(1., 1., 1.),
        );

        // gui
        let test = glm::unproject(
            &glm::vec3(0.05, 0.05, 0.5),
            &camera.view,
            &camera.projection,
            glm::vec4(0., 0., 1., 1.),
        );
        drawer.draw_gizmo(&test, 0.01, 0.5);
        gizmo.draw(&drawer, &camera);

        // ************************* RENDERING **********************8**

        window.gl_swap_window();
    }
}
