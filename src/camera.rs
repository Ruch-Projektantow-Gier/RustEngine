use crate::cube::{Line2D, Ray};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Copy, Clone)]
pub enum CameraMovement {
    NEGATIVE,
    POSITIVE,
    NONE,
}

pub struct Camera {
    pub screen_width: u32,
    pub screen_height: u32,

    near_plane: f32,
    far_plane: f32,

    pub projection: glm::Mat4,
    pub view: glm::Mat4,

    pub position: glm::Vec3,
    pub direction_to_camera: glm::Vec3, // normalized

    // movement
    speed: f32,
    pub move_x: CameraMovement,
    pub move_z: CameraMovement,
    pub move_y: CameraMovement,

    // rotating
    last_cursor: glm::TVec2<i32>,
    is_looking_around: bool,
    cam_sensitive: f32,
    yaw: f32,   // y
    pitch: f32, // x
}

impl Default for Camera {
    fn default() -> Self {
        let camera_up: glm::Vec3 = glm::vec3(0., 1., 0.);

        let position = glm::vec3(0., 4., 6.);
        let direction_to_camera = glm::vec3(0., -4., -6.).normalize(); // from 0,0,0 to camera
        let screen_width = 900;
        let screen_height = 700;
        let near_plane = 0.1;
        let far_plane = 300.0;

        Self {
            screen_width,
            screen_height,
            near_plane,
            far_plane,
            projection: glm::perspective(
                (screen_width / screen_height) as f32,
                45.0,
                near_plane,
                far_plane,
            ),
            view: glm::look_at(&position, &(&position + &direction_to_camera), &camera_up),
            position,
            direction_to_camera,
            speed: 0.1,
            move_x: CameraMovement::NONE,
            move_z: CameraMovement::NONE,
            move_y: CameraMovement::NONE,
            last_cursor: glm::vec2(0, 0),
            is_looking_around: false,
            cam_sensitive: 0.1,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl Camera {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn update(&mut self) {
        let camera_up: glm::Vec3 = glm::vec3(0., 1., 0.);

        fn dir_val(m: CameraMovement) -> f32 {
            match m {
                CameraMovement::NEGATIVE => -1.,
                CameraMovement::POSITIVE => 1.,
                CameraMovement::NONE => 0.,
            }
        }

        let move_x = dir_val(self.move_x);
        let move_z = dir_val(self.move_z);
        let move_y = dir_val(self.move_y);

        self.position += glm::normalize(&glm::cross(&self.direction_to_camera, &camera_up))
            * self.speed
            * move_x;
        self.position += &self.direction_to_camera * self.speed * move_z;
        self.position += camera_up * self.speed * move_y;

        self.view = glm::look_at(
            &self.position,
            &(&self.position + &self.direction_to_camera),
            &camera_up,
        );
    }

    pub fn click(&mut self, x: i32, y: i32) {
        self.last_cursor = glm::vec2(x, y);
        self.is_looking_around = true;
    }

    pub fn unclick(&mut self) {
        self.is_looking_around = false;
    }

    pub fn handle_mouse(&mut self, x: i32, y: i32) {
        if self.is_looking_around {
            let x_offset = x - self.last_cursor.x;
            let y_offset = self.last_cursor.y - y;

            self.last_cursor = glm::vec2(x, y);

            self.yaw += self.cam_sensitive * x_offset as f32;
            self.pitch += self.cam_sensitive * y_offset as f32;

            if self.pitch.abs() > 89.0 {
                self.pitch = self.pitch.signum() * 89.0;
            }

            let direction = glm::vec3(
                self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
                self.pitch.to_radians().sin(),
                self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
            );

            self.direction_to_camera = direction.normalize();
        }
    }

    pub fn set_direction(&mut self, direction: glm::Vec3) {
        self.direction_to_camera = direction;
    }

    pub fn set_position(&mut self, position: glm::Vec3) {
        self.position = position;
    }

    pub fn set_move_x(&mut self, movement: CameraMovement) {
        self.move_x = movement;
    }

    pub fn set_move_z(&mut self, movement: CameraMovement) {
        self.move_z = movement;
    }

    pub fn set_move_y(&mut self, movement: CameraMovement) {
        self.move_y = movement;
    }

    pub fn world_to_screen(&self, obj: &glm::Vec3) -> glm::Vec3 {
        glm::project(
            &obj,
            &self.view,
            &self.projection,
            glm::vec4(0., 0., self.screen_width as f32, self.screen_height as f32),
        )
    }

    pub fn screen_to_world(&self, obj: &glm::Vec3) -> glm::Vec3 {
        glm::unproject(
            &obj,
            &self.view,
            &self.projection,
            glm::vec4(0., 0., self.screen_width as f32, self.screen_height as f32),
        )
    }

    // from pixels to NDC
    pub fn screen_to_ndc(&self, screen: &glm::Vec2) -> glm::Vec2 {
        glm::vec2(
            screen.x / (self.screen_width as f32) * 2. - 1., //
            screen.y / (self.screen_height as f32) * 2. - 1.,
        )
    }

    // inverting y
    pub fn cursor_to_screen(&self, cursor: &glm::TVec2<i32>) -> glm::Vec2 {
        glm::vec2(
            cursor.x as f32, //
            self.screen_height as f32 - cursor.y as f32,
        )
    }

    pub fn line_from_ray(&self, ray: &Ray, length: f32) -> Line2D {
        let viewport = glm::vec4(0., 0., self.screen_width as f32, self.screen_height as f32);

        let from = glm::project(&ray.origin, &self.view, &self.projection, viewport.clone());
        let to = glm::project(
            &(&ray.origin + &ray.dir * length),
            &self.view,
            &self.projection,
            viewport.clone(),
        );

        // println!("{}", glm::vec3_to_vec2(&to));

        Line2D {
            from: glm::vec3_to_vec2(&from),
            to: glm::vec3_to_vec2(&to),
        }
    }

    /**
        y is not inverted
    */
    pub fn cast_cursor_on_plane(
        &self,
        cursor: &glm::TVec2<i32>,
        plane_normal: &glm::Vec3,
        plane_origin_offset: &glm::Vec3,
    ) -> glm::Vec3 {
        let screen = self.cursor_to_screen(cursor);
        //
        // // direction from camera
        let direction = self.screen_to_world(&glm::vec3(screen.x, screen.y, 1.))
            - self.screen_to_world(&glm::vec3(screen.x, screen.y, 0.));

        // create plane
        let nd = glm::dot(&direction, &plane_normal);
        let pn = glm::dot(&self.position, &plane_normal);
        // let t = (plane_origin_offset.magnitude() - pn) / nd; // distance
        let t = (pn) / nd; // distance

        // get point
        // &self.position + direction * t
        &self.position - direction * t
    }
}
