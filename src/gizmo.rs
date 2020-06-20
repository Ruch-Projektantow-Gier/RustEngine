use crate::camera::Camera;
use crate::components::TransformComponent;
use crate::cube::Ray;
use crate::debug::DebugDrawer;
use crate::utilities::is_point_on_line2D;
use std::cell::RefCell;
use std::rc::Rc;

type MousePos = glm::TVec2<i32>;
type Target = TransformComponent;
type TargetPtr = Rc<RefCell<Target>>;

// Y is dynamic, based on camera
struct GizmoTranslate {
    axis: glm::Vec3,
    normal: glm::Vec3, // normal of plane to project on
}

#[derive(Copy, Clone)]
enum GizmoTranslateMode {
    X,
    Y,
    Z,
}

pub struct Gizmo {
    target: Option<TargetPtr>,
    cached_target: Option<Target>,

    is_dragging: bool,
    mouse_start: MousePos,
    mouse_end: MousePos,

    translate_mode: GizmoTranslateMode,
}

impl Gizmo {
    pub fn new() -> Self {
        Self {
            target: None,
            cached_target: None,
            is_dragging: false,
            mouse_start: glm::vec2(0, 0),
            mouse_end: glm::vec2(0, 0),
            translate_mode: GizmoTranslateMode::X,
        }
    }

    #[inline]
    fn translate_mode(mode: GizmoTranslateMode, camera: &Camera) -> GizmoTranslate {
        match mode {
            GizmoTranslateMode::X => GizmoTranslate {
                axis: glm::vec3(1., 0., 0.),
                normal: glm::vec3(0., 1., 0.),
            },
            GizmoTranslateMode::Y => GizmoTranslate {
                axis: glm::vec3(0., 1., 0.),
                normal: camera.direction.clone(),
            },
            GizmoTranslateMode::Z => GizmoTranslate {
                axis: glm::vec3(0., 0., 1.),
                normal: glm::vec3(0., 1., 0.),
            },
        }
    }

    pub fn target(&mut self, target: TargetPtr) {
        let clone = target.borrow().clone();
        self.target = Some(target);
        self.cached_target = Some(clone);
    }

    pub fn clear(&mut self) {
        self.target = None;
        self.cached_target = None;
    }

    pub fn click(&mut self, camera: &Camera, x: i32, y: i32) {
        match &self.target {
            Some(target) => {
                let target = target.borrow();
                let cursor = glm::vec2(x, y);
                let screen = camera.cursor_to_screen(&cursor);
                let mut hit = false;

                let line = |mode: GizmoTranslateMode| {
                    let ray = Ray::new(&target.position, &Self::translate_mode(mode, camera).axis);
                    camera.line_from_ray(&ray, 1.0)
                };

                if is_point_on_line2D(&line(GizmoTranslateMode::X), &screen, 1.0) {
                    self.translate_mode = GizmoTranslateMode::X;
                    hit = true;
                } else if is_point_on_line2D(&line(GizmoTranslateMode::Y), &screen, 1.0) {
                    self.translate_mode = GizmoTranslateMode::Y;
                    hit = true;
                } else if is_point_on_line2D(&line(GizmoTranslateMode::Z), &screen, 1.0) {
                    self.translate_mode = GizmoTranslateMode::Z;
                    hit = true;
                }

                if hit {
                    self.mouse_start = cursor.clone();
                    self.mouse_end = cursor.clone();
                    self.is_dragging = true;
                }
            }
            _ => {}
        }
    }

    pub fn unclick(&mut self) {
        self.is_dragging = false;

        match &mut self.target {
            Some(target) => {
                let clone = target.borrow().clone();
                self.cached_target = Some(clone);
            }
            _ => {}
        }
    }

    pub fn drag(&mut self, camera: &Camera, x: i32, y: i32) {
        if !self.is_dragging {
            return;
        }

        self.mouse_end = glm::vec2(x, y);

        match &mut self.target {
            Some(target) => {
                let mut target = target.borrow_mut();
                let cached = self.cached_target.as_ref().unwrap();
                let mode = Self::translate_mode(self.translate_mode, &camera);

                let plane_origin = &mode.normal * glm::dot(&cached.position, &mode.normal);

                let p1 =
                    camera.cast_cursor_on_plane(&self.mouse_start, &mode.normal, &plane_origin);
                let p2 = camera.cast_cursor_on_plane(&self.mouse_end, &mode.normal, &plane_origin);

                let diff = p2 - p1;

                let offset_proj_length = glm::dot(&diff, &mode.axis);
                let offset_proj = &mode.axis * offset_proj_length;

                target.position = &cached.position + offset_proj;
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
