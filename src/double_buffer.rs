/**
    SCENE BUFFER
**/

pub struct SceneBuffer {
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

/**
    DOUBLE BUFFERED
**/

pub trait Interpolatable {
    fn interpolate(&self, prev: &Self, alpha: f32) -> Self;
}

pub struct DoubleBuffered<T> {
    first: T,
    second: T,
}

impl<T: std::clone::Clone> DoubleBuffered<T> {
    pub fn new(obj: T) -> Self {
        DoubleBuffered {
            first: obj.clone(),
            second: obj.clone(),
        }
    }

    pub fn get(&self, scene_buffer: &SceneBuffer) -> &T {
        if scene_buffer.is_first {
            &self.first
        } else {
            &self.second
        }
    }

    pub fn get_mut(&mut self, scene_buffer: &SceneBuffer) -> &mut T {
        if scene_buffer.is_first {
            &mut self.first
        } else {
            &mut self.second
        }
    }
}

impl<T: Interpolatable> DoubleBuffered<T> {
    pub fn interpolate(&self, scene_buffer: &SceneBuffer, alpha: f32) -> T {
        if scene_buffer.is_first {
            self.first.interpolate(&self.second, alpha)
        } else {
            self.second.interpolate(&self.first, alpha)
        }
    }
}
