extern crate nalgebra_glm as glm;

type Vec3f = glm::TVec3<f32>;

pub static CUBE_SIZE: f32 = 1.0;
pub static CUBE_HALF_SIZE: f32 = CUBE_SIZE * 0.5;

fn invert_vector(v: &Vec3f) -> Vec3f {
    glm::vec3(
        if v[0] != 0. { 1. / v[0] } else { f32::INFINITY },
        if v[1] != 0. { 1. / v[1] } else { f32::INFINITY },
        if v[2] != 0. { 1. / v[2] } else { f32::INFINITY },
    )
}

pub struct Cube {
    origin: Vec3f,
    min: Vec3f,
    max: Vec3f,
}

pub struct Ray {
    pub origin: Vec3f, // min = origin - HALF_CUBE, max = origin + HALF_CUBE
    pub inv_dir: Vec3f,
    pub dir: Vec3f,
}

impl Ray {
    pub fn new(origin: &Vec3f, dir: &Vec3f) -> Ray {
        Ray {
            origin: origin.clone(),
            dir: dir.clone(),
            inv_dir: invert_vector(dir),
        }
    }
}

#[derive(Copy, Clone)]
pub enum EFace {
    Right,
    Left,
    Top,
    Bottom,
    Front,
    Back,
    None,
}

pub type Face = (EFace, glm::Vec3);
type Coords3 = [f32; 3];
static FACES: [(EFace, Coords3); 6] = [
    (EFace::Right, [1., 0., 0.]),
    (EFace::Left, [-1., 0., 0.]),
    (EFace::Top, [0., 1., 0.]),
    (EFace::Bottom, [0., -1., 0.]),
    (EFace::Front, [0., 0., 1.]),
    (EFace::Back, [0., 0., -1.]),
];

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

    fn get_contacts_distances(&self, ray: &Ray) -> (f32, f32) {
        let mut tmin = -f32::INFINITY;
        let mut tmax = f32::INFINITY;

        for i in 0..3 {
            let t1: f32 = (self.min[i] - ray.origin[i]) * ray.inv_dir[i];
            let t2: f32 = (self.max[i] - ray.origin[i]) * ray.inv_dir[i];

            tmin = t1.min(t2).max(tmin);
            tmax = t1.max(t2).min(tmax);
        }

        tmin = tmin.max(0.);

        (tmin, tmax)
    }

    pub fn is_intersect(&self, ray: &Ray) -> bool {
        let (tmin, tmax) = self.get_contacts_distances(&ray);
        tmax > tmin
    }

    pub fn get_contact(&self, ray: &Ray) -> Vec3f {
        let (tmin, ..) = self.get_contacts_distances(&ray);
        ray.origin + invert_vector(&ray.inv_dir) * tmin
    }

    pub fn get_intersect_face(&self, ray: &Ray) -> Face {
        let contact = self.get_contact(&ray);
        let dir_to_camera = &(contact - self.origin).normalize();

        let mut max_proj = 0.;
        let mut target: Face = (EFace::None, glm::vec3(0., 0., 0.));

        for &face in &FACES {
            let (face_type, normal_coords) = face;
            let normal = glm::make_vec3(&normal_coords);
            let proj = glm::dot(&normal, dir_to_camera);

            if proj > max_proj {
                target = (face_type, normal);
                max_proj = proj;
            }
        }

        target
    }
}
