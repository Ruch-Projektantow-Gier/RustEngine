use crate::double_buffer::{DoubleBuffered, Interpolatable};

#[derive(Clone)]
pub struct TransformComponent {
    pub position: glm::Vec3,
    pub rotation: glm::Quat,
    pub scale: glm::Vec3,
}

impl TransformComponent {
    pub fn new(position: glm::Vec3, rotation: glm::Quat, scale: glm::Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn buffer(self) -> DoubleBuffered<Self> {
        DoubleBuffered::new(self)
    }

    pub fn mat4(&self) -> glm::Mat4 {
        &glm::one::<glm::Mat4>()
            * glm::translation(&self.position)
            * glm::scaling(&self.scale)
            * glm::quat_to_mat4(&self.rotation)
    }
}

impl Interpolatable for TransformComponent {
    fn interpolate(&self, prev: &Self, alpha: f32) -> Self {
        TransformComponent {
            position: glm::lerp(&self.position, &prev.position, alpha),
            rotation: glm::quat_slerp(&self.rotation, &prev.rotation, alpha),
            scale: glm::lerp(&self.scale, &prev.scale, alpha),
        }
    }
}
