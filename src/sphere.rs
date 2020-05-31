extern crate nalgebra_glm as glm;

type Vec3f = glm::TVec3<f32>;

/* http://www.songho.ca/opengl/gl_sphere.html */
pub fn gen_sphere_vertices(subdivision: u32) -> Vec<f32> {
    let deg_to_rad = std::f32::consts::PI / 180.;
    let pointsPerRow = 2_i32.pow(subdivision) + 1;
    let mut vertices = Vec::with_capacity((pointsPerRow * pointsPerRow) as usize);

    let mut n1 = [0., 0., 0.];
    let mut n2 = [0., 0., 0.];
    let mut v = [0., 0., 0.];
    let mut a1 = 0.;
    let mut a2 = 0.;

    for i in 0..pointsPerRow {
        a2 = deg_to_rad * (45. - 90. * (i as f32) / ((pointsPerRow - 1) as f32));
        n2[0] = (-a2).sin();
        n2[1] = (a2).cos();
        n2[2] = 0.;

        for j in 0..pointsPerRow {
            a1 = deg_to_rad * (-45. + 90. * (j as f32) / ((pointsPerRow - 1) as f32));
            n2[0] = (-a1).sin();
            n2[1] = 0.;
            n2[2] = -(a1).cos();

            v[0] = n1[1] * n2[2] - n1[2] * n2[1];
            v[1] = n1[2] * n2[0] - n1[0] * n2[2];
            v[2] = n1[0] * n2[1] - n1[1] * n2[0];

            let scale = 1. / (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            v[0] *= scale;
            v[1] *= scale;
            v[2] *= scale;

            let index = ((i * pointsPerRow + j) * 3) as usize;

            vertices[index] = v[0];
            vertices[index + 1] = v[1];
            vertices[index + 2] = v[2];
        }
    }

    vertices
}
