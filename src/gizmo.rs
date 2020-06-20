use crate::camera::Camera;
use crate::components::TransformComponent;
use crate::cube::Ray;
use crate::debug::DebugDrawer;
use crate::utilities::is_point_on_line2D;
use std::cell::RefCell;
use std::rc::Rc;

type MousePos = glm::TVec2<i32>;
type TargetPtr = Rc<RefCell<TransformComponent>>;

pub struct Gizmo {
    target: Option<TargetPtr>,

    is_dragging: bool,
    mouse_start: MousePos,
    mouse_end: MousePos,
}

impl Gizmo {
    pub fn new() -> Self {
        Self {
            target: None,
            is_dragging: false,
            mouse_start: glm::vec2(0, 0),
            mouse_end: glm::vec2(0, 0),
        }
    }

    pub fn target(&mut self, target: TargetPtr) {
        self.target = Some(target);
    }

    pub fn clear(&mut self) {
        self.target = None;
    }

    pub fn click(&mut self, camera: &Camera, x: i32, y: i32) {
        match &self.target {
            Some(target) => {
                let target = target.borrow();
                let point = glm::vec2(x, y);

                // Creating line
                let ray_x = Ray::new(&target.position, &glm::vec3(1., 0., 0.));
                let line_x = camera.line_from_ray(&ray_x, 1.0);

                // todo y,z
                if is_point_on_line2D(&line_x, &camera.cursor_to_screen(&point), 1.0) {
                    self.mouse_start = point.clone();
                    self.mouse_end = point.clone();

                    self.is_dragging = true;
                }
            }
            _ => {}
        }
    }

    pub fn unclick(&mut self) {
        self.is_dragging = false;
    }

    pub fn drag(&mut self, camera: &Camera, x: i32, y: i32) {
        if !self.is_dragging {
            return;
        }

        self.mouse_end = glm::vec2(x, y);

        match &mut self.target {
            Some(target) => {
                let mut target = target.borrow_mut();

                // todo y, z
                let plane_normal = glm::vec3(0., 1., 0.);
                let p1 = camera.cast_cursor_on_plane(&self.mouse_start, &plane_normal);
                let p2 = camera.cast_cursor_on_plane(&self.mouse_end, &plane_normal);

                let diff = p2 - p1;

                // todo y, z
                let axis = &glm::vec3(1.0, 0., 0.);
                let offset_proj_length = glm::dot(&diff, &axis);
                let offset_proj = axis * offset_proj_length;

                target.position += offset_proj;
            }
            _ => {}
        }
    }

    pub fn draw(&self, drawer: &DebugDrawer) {
        match &self.target {
            Some(target) => {
                let target = target.borrow();
                drawer.draw_gizmo(&target.position, 1., 1.);
            }
            _ => {}
        }
    }
}
