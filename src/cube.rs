extern crate nalgebra_glm as glm;

type Vec3f = glm::TVec3<f32>;

static CUBE_SIZE: f32 = 1.0;
static CUBE_HALF_SIZE: f32 = CUBE_SIZE * 0.5;

pub struct Cube {
    origin: Vec3f,
    min: Vec3f,
    max: Vec3f,
}

pub struct Ray {
    origin: Vec3f, // min = origin - HALF_CUBE, max = origin + HALF_CUBE
    dir: Vec3f,
}

impl Ray {
    pub fn new(origin: &Vec3f, dir: &Vec3f) -> Ray {
        Ray {
            origin: origin.clone(),
            dir: dir.clone(),
        }
    }
}

impl Cube {
    pub fn new(origin: &Vec3f) -> Cube {
        let half_cube = glm::vec3(1., 1., 1.) * CUBE_HALF_SIZE;
        let origin_f = origin * CUBE_SIZE;

        Cube {
            origin: origin_f,
            min: origin_f - half_cube,
            max: origin_f + half_cube,
        }
    }

    pub fn is_intersect(&self, ray: &Ray) -> bool {
        let mut tmin = -f32::INFINITY;
        let mut tmax = f32::INFINITY;

        let inv_dir = glm::vec3(1.0 / ray.dir[0], 1.0 / ray.dir[1], 1.0 / ray.dir[2]);

        for i in 0..3 {
            let t1: f32 = (self.min[i] - ray.origin[i]) * inv_dir[i];
            let t2: f32 = (self.max[i] - ray.origin[i]) * inv_dir[i];

            tmin = t1.min(t2).max(tmin);
            tmax = t1.max(t2).min(tmax);
        }

        tmax > tmin.max(0.)
    }
}
