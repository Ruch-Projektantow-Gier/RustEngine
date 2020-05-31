extern crate nalgebra_glm as glm;

type Vec3f = glm::TVec3<f32>;

static CUBE_SIZE: f32 = 1.0;
static CUBE_HALF_SIZE: f32 = CUBE_SIZE * 0.5;

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
    origin: Vec3f, // min = origin - HALF_CUBE, max = origin + HALF_CUBE
    inv_dir: Vec3f,
}

impl Ray {
    pub fn new(origin: &Vec3f, dir: &Vec3f) -> Ray {
        Ray {
            origin: origin.clone(),
            inv_dir: invert_vector(dir),
        }
    }
}

pub enum CubeFace {
    Right,
    Left,
    Top,
    Bottom,
    Front,
    Back,
    None,
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

    pub fn get_intersect_face(&self, ray: &Ray) {
        let contact = self.get_contact(&ray);
        let dir_to_camera = &(contact - self.origin).normalize();

        let faces = [
            (CubeFace::Right, glm::vec3(1., 0., 0.)),
            (CubeFace::Left, glm::vec3(-1., 0., 0.)),
            (CubeFace::Top, glm::vec3(0., 1., 0.)),
            (CubeFace::Bottom, glm::vec3(0., -1., 0.)),
            (CubeFace::Front, glm::vec3(0., 0., 1.)),
            (CubeFace::Back, glm::vec3(0., 0., -1.)),
        ];

        type Visible<'a> = (&'a CubeFace, f32);
        let mut target: Visible = (&CubeFace::None, -1.);

        for face in faces.iter() {
            let (face_type, normal) = face;
            let proj = glm::dot(&normal, dir_to_camera);

            if proj > target.1 {
                target = (face_type, proj);
            }
        }

        match target.0 {
            CubeFace::Right => {
                println!("Right! ");
            }
            CubeFace::Left => {
                println!("Left! ");
            }
            CubeFace::Top => {
                println!("Top! ");
            }
            CubeFace::Bottom => {
                println!("Bottom! ");
            }
            CubeFace::Front => {
                println!("Front! ");
            }
            CubeFace::Back => {
                println!("Back! ");
            }
            _ => {}
        }
    }
}
