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

struct PointLight {
    color: glm::Vec4,
    position: glm::Vec4,
    padding_and_radius: glm::Vec4,
}

struct VisibleIndex {
    index: i32,
}

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
    let near = 0.1;
    let far = 300.;

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

    // Render
    let debug = debug::Debug::new(&gl);
    texture::Texture::init(&gl); // anisotropic

    let texture = texture::Texture::new(&gl, "res/wall.jpg").unwrap();
    let render_cube = primitives::build_cube(&gl);
    let quad = primitives::build_quad(&gl);

    unsafe {
        gl.Enable(gl::DEPTH_TEST);
        gl.DepthMask(gl::TRUE);

        gl.Enable(gl::CULL_FACE);
        gl.CullFace(gl::BACK);
        // gl.FrontFace(gl::CW);

        gl.Enable(gl::MULTISAMPLE);
    }

    /* Forward+ Rendering */
    type GlInt = gl::types::GLuint;

    /* 1. Load shaders */
    let depth_program = render_gl::Program::from_files(
        &gl,
        include_str!("shaders/depth.vert"),
        include_str!("shaders/depth.frag"),
    )
    .unwrap();

    let light_culling_program = render_gl::Program::from_compute_shader_file(
        &gl,
        include_str!("shaders/light_culling.comp"),
    )
    .unwrap();

    let light_accumulation_shader = render_gl::Program::from_files(
        &gl,
        include_str!("shaders/light_accumulation.vert"),
        include_str!("shaders/light_accumulation.frag"),
    )
    .unwrap();

    let hdr_shader = render_gl::Program::from_files(
        &gl,
        include_str!("shaders/hdr.vert"),
        include_str!("shaders/hdr.frag"),
    )
    .unwrap();

    /* 2. Depth map - framebuffer */
    let mut depth_map_fbo: GlInt = 0;
    let mut depth_map: GlInt = 0; // texture

    unsafe {
        gl.GenBuffers(1, &mut depth_map_fbo);

        gl.GenTextures(1, &mut depth_map);
        gl.BindTexture(gl::TEXTURE_2D, depth_map);
        gl.TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::DEPTH_COMPONENT as gl::types::GLint,
            width as gl::types::GLint,
            height as gl::types::GLint,
            0,
            gl::DEPTH_COMPONENT,
            gl::FLOAT,
            std::ptr::null(),
        );
        gl.TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::NEAREST as gl::types::GLint,
        );
        gl.TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            gl::NEAREST as gl::types::GLint,
        );
        gl.TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_BORDER as gl::types::GLint,
        );
        gl.TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_BORDER as gl::types::GLint,
        );
        gl.TexParameterfv(
            gl::TEXTURE_2D,
            gl::TEXTURE_BORDER_COLOR,
            [1., 1., 1., 1.].as_ptr(),
        );

        gl.BindFramebuffer(gl::FRAMEBUFFER, depth_map_fbo);
        gl.FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::DEPTH_ATTACHMENT,
            gl::TEXTURE_2D,
            depth_map,
            0,
        );
        // gl.DrawBuffer(gl::NONE);
        // gl.ReadBuffer(gl::NONE);
        gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    /* 3. HDR FBO */
    let mut hdr_fbo: GlInt = 0;
    let mut color_buffer: GlInt = 0;
    let mut rbo_depth: GlInt = 0;

    unsafe {
        gl.GenFramebuffers(1, &mut hdr_fbo);

        // color buffer
        gl.GenTextures(1, &mut color_buffer);
        gl.BindTexture(gl::TEXTURE_2D, color_buffer);
        gl.TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB16F as gl::types::GLint,
            width as gl::types::GLint,
            height as gl::types::GLint,
            0,
            gl::RGB,
            gl::FLOAT,
            std::ptr::null(),
        );
        gl.TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR as gl::types::GLint,
        );
        gl.TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR as gl::types::GLint,
        );

        // It will also need a depth component as a render buffer, attached to the hdrFBO
        gl.GenRenderbuffers(1, &mut rbo_depth);
        gl.BindRenderbuffer(gl::RENDERBUFFER, rbo_depth);
        gl.RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH_COMPONENT,
            width as gl::types::GLint,
            height as gl::types::GLint,
        );

        gl.BindFramebuffer(gl::FRAMEBUFFER, hdr_fbo);
        gl.FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            color_buffer,
            0,
        );
        gl.FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_ATTACHMENT,
            gl::RENDERBUFFER,
            rbo_depth,
        );
        gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    /* 4. Light on scene */
    let num_lights = 1;
    let mut light_buffer: GlInt = 0;
    let mut visible_light_indices_buffer: GlInt = 0;

    // X and Y work group dimension variables for compute shader
    let work_groups_x: GlInt = (width + (width % 16)) / 16;
    let work_groups_y: GlInt = (height + (height % 16)) / 16;
    let number_of_tiles = (work_groups_x * work_groups_y) as usize;

    unsafe {
        // Generate our shader storage buffers
        gl.GenBuffers(1, &mut light_buffer);
        gl.GenBuffers(1, &mut visible_light_indices_buffer);

        // Bind light buffer
        gl.BindBuffer(gl::SHADER_STORAGE_BUFFER, light_buffer);
        gl.BufferData(
            gl::SHADER_STORAGE_BUFFER,
            (num_lights * std::mem::size_of::<PointLight>()) as gl::types::GLsizeiptr,
            std::ptr::null(),
            gl::DYNAMIC_DRAW,
        );

        // Bind visible light indices buffer
        gl.BindBuffer(gl::SHADER_STORAGE_BUFFER, visible_light_indices_buffer);
        gl.BufferData(
            gl::SHADER_STORAGE_BUFFER,
            (number_of_tiles * std::mem::size_of::<VisibleIndex>()  * 1024) // was 1024
                as gl::types::GLsizeiptr,
            std::ptr::null(),
            gl::STATIC_DRAW,
        );
    }

    // SetupLights
    unsafe {
        gl.BindBuffer(gl::SHADER_STORAGE_BUFFER, light_buffer);

        let point_lights = std::slice::from_raw_parts_mut(
            gl.MapBuffer(gl::SHADER_STORAGE_BUFFER, gl::READ_WRITE) as *mut PointLight,
            num_lights,
        );

        for point in point_lights {
            point.position = glm::vec4(-1., 0., 0., 1.0);
            point.color = glm::vec4(1., 1., 1., 1.0);
            point.padding_and_radius = glm::vec4(0., 0., 0., 1.0);
        }

        gl.UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
        gl.BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);

        gl.BindBuffer(gl::SHADER_STORAGE_BUFFER, 0); // doubled (unnecessary)
    }

    /* 5. Projection quad */
    let screen_model_mat4 = &glm::one::<Mat4>() * glm::scaling(&glm::vec3(0.1, 0.1, 0.1));

    depth_program.bind();
    depth_program.setMat4(&screen_model_mat4, "model");

    light_culling_program.bind();
    light_culling_program.setInt(num_lights as i32, "lightCount");
    light_culling_program.setVec2Int(&glm::vec2(width as i32, height as i32), "screenSize");

    light_accumulation_shader.bind();
    light_accumulation_shader.setMat4(&screen_model_mat4, "model");
    light_accumulation_shader.setInt(work_groups_x as i32, "numberOfTilesX");

    unsafe {
        gl.Viewport(0, 0, width as gl::types::GLint, height as gl::types::GLint);
        gl.ClearColor(0.1, 0.1, 0.1, 1.);
    }

    /////////////////////////////////////

    let mut scene_buffer = SceneBuffer::new();
    let mut cubes: Vec<DoubleBuffered<TransformComponent>> = vec![];
    cubes.push(DoubleBuffered::new(TransformComponent {
        position: glm::vec3(0., 0., 0.),
        rotation: glm::quat_identity(),
        scale: glm::vec3(1., 1., 1.),
    }));
    cubes.push(DoubleBuffered::new(TransformComponent {
        position: glm::vec3(2., 0., 0.),
        rotation: glm::quat_identity(),
        scale: glm::vec3(1., 1., 1.),
    }));
    cubes.push(DoubleBuffered::new(TransformComponent {
        position: glm::vec3(2., 1., 0.),
        rotation: glm::quat_identity(),
        scale: glm::vec3(1., 1., 1.),
    }));
    cubes.push(DoubleBuffered::new(TransformComponent {
        position: glm::vec3(0., -1., 0.),
        rotation: glm::quat_identity(),
        scale: glm::vec3(100., 1., 100.),
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

            // update
            lag -= s_per_update;
        }

        let alpha: f32 = lag / s_per_update;

        // ************************* RENDERING **********************8**
        // MATRIXES
        let proj = glm::perspective((width / height) as f32, 45.0, near, far);
        let view = glm::look_at(&camera_pos, &(&camera_pos + &camera_front), &camera_up);

        // Step 1: Render the depth of the scene to a depth map
        depth_program.bind();
        depth_program.setMat4(&proj, "projection");
        depth_program.setMat4(&view, "view");

        // Bind the depth map's frame buffer and draw the depth map to it
        unsafe {
            gl.BindFramebuffer(gl::FRAMEBUFFER, depth_map_fbo);
            gl.Clear(gl::DEPTH_BUFFER_BIT);

            for cube in &cubes {
                let transform = cube.get(&scene_buffer);
                depth_program.setMat4(&screen_model_mat4, "model");
                render_cube.draw();
            }

            gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        // Step 2: Perform light culling on point lights in the scene
        light_culling_program.bind();
        light_culling_program.setMat4(&proj, "projection");
        light_culling_program.setMat4(&view, "view");

        unsafe {
            // Bind depth map texture to texture location 4 (which will not be used by any model texture)
            gl.ActiveTexture(gl::TEXTURE4);
            light_culling_program.setInt(4, "depthMap");
            gl.BindTexture(gl::TEXTURE_2D, depth_map);

            // Bind shader storage buffer objects for the light and indice buffers
            gl.BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, light_buffer);
            gl.BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, visible_light_indices_buffer);

            // Dispatch the compute shader, using the workgroup values calculated earlier
            gl.DispatchCompute(work_groups_x, work_groups_y, 1);

            // Unbind the depth map
            gl.ActiveTexture(gl::TEXTURE4);
            gl.BindTexture(gl::TEXTURE_2D, 0);
        }

        // Step 3: Accumulate the remaining lights after culling and render (or execute one of the debug views of a flag is enabled
        unsafe {
            gl.BindFramebuffer(gl::FRAMEBUFFER, hdr_fbo);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            light_accumulation_shader.bind();
            light_accumulation_shader.setMat4(&proj, "projection");
            light_accumulation_shader.setMat4(&view, "view");
            light_accumulation_shader.setVec3Float(&camera_pos, "viewPosition");
        }

        // RENDER BOX
        for cube in &cubes {
            let transform = cube.get(&scene_buffer);
            light_accumulation_shader.setMat4(&transform.get_mat4(), "model");
            render_cube.draw();
        }

        unsafe {
            gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        // Tonemap the HDR colors to the default framebuffer
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            hdr_shader.bind();
            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, color_buffer);
            hdr_shader.setFloat(1.0, "exposure");

            quad.draw();

            gl.BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, 0);
            gl.BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, 0);
        }

        // ************************* RENDERING **********************8**

        window.gl_swap_window();
    }
}
