extern crate nalgebra_glm as glm;

use nalgebra_glm::{atan, sin, vec3};

type Vec3f = glm::TVec3<f32>;

/* http://www.songho.ca/opengl/gl_sphere.html */
pub fn gen_sphere_vertices(subdivision: u32) -> Vec<f32> {
    let deg_to_rad = std::f32::consts::PI / 180.;
    let pointsPerRow = 2_i32.pow(subdivision) + 1;
    let mut vertices = Vec::with_capacity((pointsPerRow * pointsPerRow) as usize);
    // let mut vertices = Vec::;

    let mut n1 = [0., 0., 0.];
    let mut n2 = [0., 0., 0.];
    let mut v = [0., 0., 0.];
    let mut a1 = 0.;
    let mut a2 = 0.;

    for i in 0..pointsPerRow {
        a2 = deg_to_rad * (45. - 90. * (i as f32) / ((pointsPerRow - 1) as f32));
        n2[0] = -(a2.sin());
        n2[1] = a2.cos();
        n2[2] = 0.;

        for j in 0..pointsPerRow {
            a1 = deg_to_rad * (-45. + 90. * (j as f32) / ((pointsPerRow - 1) as f32));
            n1[0] = -(a1.sin());
            n1[1] = 0.;
            n1[2] = -(a1.cos());

            v[0] = n1[1] * n2[2] - n1[2] * n2[1];
            v[1] = n1[2] * n2[0] - n1[0] * n2[2];
            v[2] = n1[0] * n2[1] - n1[1] * n2[0];

            let scale = 1. / (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            v[0] *= scale;
            v[1] *= scale;
            v[2] *= scale;

            // let index = ((i * pointsPerRow + j) * 3) as usize;
            // vertices[index] = v[0];
            // vertices[index + 1] = v[1];
            // vertices[index + 2] = v[2];

            vertices.push(v[0]);
            vertices.push(v[1]);
            vertices.push(v[2]);
        }
    }

    vertices
}

pub fn gen_sphere(radius: f32, stack_count: u32, sector_count: u32) -> (Vec<f32>, Vec<u32>) {
    let mut bundle = vec![];
    let mut indices = vec![];

    use crate::glm::RealField;

    let pi = f32::pi();
    let two_pi = f32::pi() * 2.;
    let half_pi = f32::pi() / 2.;

    let mut x: f32;
    let mut y: f32;
    let mut z: f32;
    let mut xy: f32;

    let mut nx: f32;
    let mut ny: f32;
    let mut nz: f32;
    let length_inv = 1. / radius;

    let mut s: f32;
    let mut t: f32;

    let sector_step = two_pi / (sector_count as f32);
    let stack_step = pi / (stack_count as f32);

    let mut sector_angle: f32;
    let mut stack_angle: f32;

    // vert, normals, coords
    for i in 0..stack_count {
        stack_angle = half_pi - (i as f32) * stack_step;
        xy = radius * stack_angle.cos();
        z = radius * stack_angle.sin();

        for j in 0..=sector_count {
            sector_angle = (j as f32) * sector_step;

            x = xy * sector_angle.cos();
            y = xy * sector_angle.sin();

            // vertices
            bundle.push(x);
            bundle.push(y);
            bundle.push(z);

            nx = x * length_inv;
            ny = y * length_inv;
            nz = z * length_inv;

            // normals
            bundle.push(nx);
            bundle.push(ny);
            bundle.push(nz);

            s = (j / sector_count) as f32;
            t = (i / stack_count) as f32;

            // coords
            bundle.push(s);
            bundle.push(t);
        }
    }

    // indices
    let mut k1: u32;
    let mut k2: u32;

    for i in 0..=stack_count {
        k1 = i * sector_count + 1;
        k2 = k1 + sector_count + 1;

        for j in 0..=sector_count {
            if i != 0 {
                indices.push(k1);
                indices.push(k2);
                indices.push(k1 + 1);
            }

            if i != (stack_count - 1) {
                indices.push(k1 + 1);
                indices.push(k2);
                indices.push(k2 + 1);
            }

            k1 += 1;
            k2 += 1;
        }
    }

    // bundle vertices
    (bundle, indices)
}

pub fn isosahedron_vertices() -> Vec<f32> {
    use crate::glm::RealField;

    let radius = 1.;

    let pi = f32::pi();
    let h_angle = pi / 180. * 72.; // 72 degree = 360 / 5
    let v_angle = (0.5 as f32).atan(); // elevation = 26.565 degree

    let mut vertices = [0.; 12 * 3];
    let mut i1: u32; // indices
    let mut i2: u32;

    let mut z: f32;
    let mut xy: f32;

    let mut h_angle_1 = -pi / 2. - h_angle / 2.; // start from -126 deg at 1st row
    let mut h_angle_2 = -pi / 2.; // start from -90 deg at 2nd row

    vertices[0] = 0.;
    vertices[1] = 0.;
    vertices[2] = radius;

    for i in 1..=5 {
        i1 = i * 3;
        i2 = (i + 5) * 3;

        z = radius * v_angle.sin();
        xy = radius * v_angle.cos();

        // x
        vertices[i1 as usize] = xy * h_angle_1.cos();
        vertices[i2 as usize] = xy * h_angle_2.cos();

        // y
        vertices[(i1 + 1) as usize] = xy * h_angle_1.sin();
        vertices[(i2 + 1) as usize] = xy * h_angle_2.sin();

        // z
        vertices[(i1 + 2) as usize] = z;
        vertices[(i2 + 2) as usize] = -z;

        h_angle_1 += h_angle;
        h_angle_2 += h_angle;
    }

    // the last bottom vertex at (0, 0, -r)
    i1 = 11 * 3;
    vertices[i1 as usize] = 0.;
    vertices[(i1 + 1) as usize] = 0.;
    vertices[(i1 + 2) as usize] = -radius;

    vertices.to_vec()
}

pub fn build_isosphere() -> ((Vec<f32>, Vec<f32>, Vec<f32>), Vec<u32>) {
    let s_step = 186. / 2048.; // horizontal texture step
    let t_step = 322. / 1024.; // vertical texture step

    let mut base_vertices = isosahedron_vertices();

    let mut out_vertices = vec![];
    let mut out_normals = vec![];
    let mut out_tex_coords = vec![];
    let mut out_indices = vec![];

    let mut add_vertices = |v1: *const f32, v2: *const f32, v3: *const f32| unsafe {
        out_vertices.push(*v1.offset(0)); // x
        out_vertices.push(*v1.offset(1)); // y
        out_vertices.push(*v1.offset(2)); // z
        out_vertices.push(*v2.offset(0));
        out_vertices.push(*v2.offset(1));
        out_vertices.push(*v2.offset(2));
        out_vertices.push(*v3.offset(0));
        out_vertices.push(*v3.offset(1));
        out_vertices.push(*v3.offset(2));
    };

    let mut add_normals = |n1: [f32; 3], n2: [f32; 3], n3: [f32; 3]| {
        out_normals.push(n1[0]); // x
        out_normals.push(n1[1]); // y
        out_normals.push(n1[2]); // z
        out_normals.push(n2[0]);
        out_normals.push(n2[1]);
        out_normals.push(n2[2]);
        out_normals.push(n3[0]);
        out_normals.push(n3[1]);
        out_normals.push(n3[2]);
    };

    let mut add_tex_coords = |t1: [f32; 2], t2: [f32; 2], t3: [f32; 2]| {
        out_tex_coords.push(t1[0]); // x
        out_tex_coords.push(t1[1]); // y
        out_tex_coords.push(t2[0]);
        out_tex_coords.push(t2[1]);
        out_tex_coords.push(t3[0]);
        out_tex_coords.push(t3[1]);
    };

    let mut add_indices = |i1: u32, i2: u32, i3: u32| {
        out_indices.push(i1);
        out_indices.push(i2);
        out_indices.push(i3);
    };

    let get_normal = |v1: *const f32, v2: *const f32, v3: *const f32| unsafe {
        let epsilon = 0.000001;
        let mut normal = [0.; 3];

        let ex1 = *v2.offset(0) - *v1.offset(0);
        let ey1 = *v2.offset(1) - *v1.offset(1);
        let ez1 = *v2.offset(2) - *v1.offset(2);
        let ex2 = *v3.offset(0) - *v1.offset(0);
        let ey2 = *v3.offset(1) - *v1.offset(1);
        let ez2 = *v3.offset(2) - *v1.offset(2);

        let nx = ey1 * ez2 - ez1 * ey2;
        let ny = ez1 * ex2 - ex1 * ez2;
        let nz = ex1 * ey2 - ey1 * ex2;

        let length = (nx * nx + ny * ny + nz * nz).sqrt();

        if length > epsilon {
            // normalize
            let length_inv = 1. / length;
            normal[0] = nx * length_inv;
            normal[1] = ny * length_inv;
            normal[2] = nz * length_inv;
        }

        normal
    };

    // vertex positions
    let mut v0: *const f32;
    let mut v1: *const f32;
    let mut v2: *const f32;
    let mut v3: *const f32;
    let mut v4: *const f32;
    let mut v11: *const f32;

    // texCoords
    let mut t0 = [0.; 2];
    let mut t1 = [0.; 2];
    let mut t2 = [0.; 2];
    let mut t3 = [0.; 2];
    let mut t4 = [0.; 2];
    let mut t11 = [0.; 2];

    let mut index = 0;

    // compute and add 20 tiangles of icosahedron first
    v0 = &base_vertices[0]; // 1st vertex
    v11 = &base_vertices[11 * 3]; // 12th vertex

    for i in 1..=5 {
        v1 = &base_vertices[i * 3];
        if i < 5 {
            v2 = &base_vertices[(i + 1) * 3];
        } else {
            v2 = &base_vertices[3];
        }

        v3 = &base_vertices[(i + 5) * 3];
        if (i + 5) < 10 {
            v4 = &base_vertices[(i + 6) * 3];
        } else {
            v4 = &base_vertices[6 * 3];
        }

        // texture coords
        t0[0] = (2. * (i as f32) - 1.) * s_step;
        t0[1] = 0.;
        t1[0] = (2. * (i as f32) - 2.) * s_step;
        t1[1] = t_step;
        t2[0] = (2. * (i as f32) - 0.) * s_step;
        t2[1] = t_step;
        t3[0] = (2. * (i as f32) - 1.) * s_step;
        t3[1] = t_step * 2.;
        t4[0] = (2. * (i as f32) + 1.) * s_step;
        t4[1] = t_step * 2.;
        t11[0] = 2. * (i as f32) * s_step;
        t11[1] = t_step * 3.;

        // add a triangle in 1st row
        add_vertices(v0, v1, v2);
        let n = get_normal(v0, v1, v2);
        add_normals(n, n, n);
        add_tex_coords(t0, t1, t2);
        add_indices(index, index + 1, index + 2);

        // add 2 triangles in 2nd row
        add_vertices(v1, v3, v2);
        let n = get_normal(v1, v3, v2);
        add_normals(n, n, n);
        add_tex_coords(t1, t3, t2);
        add_indices(index + 3, index + 4, index + 5);

        add_vertices(v2, v3, v4);
        let n = get_normal(v2, v3, v4);
        add_normals(n, n, n);
        add_tex_coords(t2, t3, t4);
        add_indices(index + 6, index + 7, index + 8);

        // add a triangle in 3rd row
        add_vertices(v3, v11, v4);
        let n = get_normal(v3, v11, v4);
        add_normals(n, n, n);
        add_tex_coords(t3, t11, t4);
        add_indices(index + 9, index + 10, index + 11);

        // add 6 edge lines per iteration
        // ignore

        index += 12;
    }

    // assert_eq!(out_vertices.len(), 12 * 3);

    // subdivision
    let mut result = (out_vertices, out_normals, out_tex_coords, out_indices);

    for _ in 0..3 {
        let (out_vertices, _, out_tex_coords, out_indices) = result;
        result = subdivide(out_vertices, out_tex_coords, out_indices);
    }

    let (out_vertices, out_normals, out_tex_coords, out_indices) = result;

    // let bundle = bundle(out_vertices, out_normals, out_tex_coords);
    ((out_vertices, out_normals, out_tex_coords), out_indices)
}

pub fn bundle(vertices: Vec<f32>, normals: Vec<f32>, text_coords: Vec<f32>) -> Vec<f32> {
    let mut result = vec![];

    let mut i = 0;
    let mut j = 0;

    while i < vertices.len() && j < text_coords.len() {
        result.push(vertices[i]);
        result.push(vertices[i + 1]);
        result.push(vertices[i + 2]);

        result.push(normals[i]);
        result.push(normals[i + 1]);
        result.push(normals[i + 2]);

        result.push(text_coords[j]);
        result.push(text_coords[j + 1]);

        i += 3;
        j += 2;
    }

    result
}

fn subdivide(
    vertices: Vec<f32>,
    text_coords: Vec<f32>,
    indices: Vec<u32>,
) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<u32>) {
    let mut out_vertices = vec![];
    let mut out_normals = vec![];
    let mut out_tex_coords = vec![];
    let mut out_indices = vec![];

    let mut v1: *const f32;
    let mut v2: *const f32;
    let mut v3: *const f32;

    let mut t1: *const f32;
    let mut t2: *const f32;
    let mut t3: *const f32;

    let mut new_v1 = [0.; 3];
    let mut new_v2 = [0.; 3];
    let mut new_v3 = [0.; 3];

    let mut new_t1 = [0.; 2];
    let mut new_t2 = [0.; 2];
    let mut new_t3 = [0.; 2];

    let mut compute_half_vertex = |v1: *const f32, v2: *const f32, new_v: &mut [f32; 3]| unsafe {
        new_v[0] = *v1.offset(0) + *v2.offset(0);
        new_v[1] = *v1.offset(1) + *v2.offset(1);
        new_v[2] = *v1.offset(2) + *v2.offset(2);

        let scale = 1. / (new_v[0] * new_v[0] + new_v[1] * new_v[1] + new_v[2] * new_v[2]).sqrt();

        new_v[0] *= scale;
        new_v[1] *= scale;
        new_v[2] *= scale;
    };

    let mut compute_half_coord = |t1: *const f32, t2: *const f32, new_t: &mut [f32; 2]| unsafe {
        new_t[0] = *t1.offset(0) + *t2.offset(0);
        new_t[1] = *t1.offset(1) + *t2.offset(1);

        new_t[0] *= 0.5;
        new_t[1] *= 0.5;
    };

    let mut add_vertices = |v1: *const f32, v2: *const f32, v3: *const f32| unsafe {
        &out_vertices.push(*v1.offset(0)); // x
        &out_vertices.push(*v1.offset(1)); // y
        &out_vertices.push(*v1.offset(2)); // z
        &out_vertices.push(*v2.offset(0));
        &out_vertices.push(*v2.offset(1));
        &out_vertices.push(*v2.offset(2));
        &out_vertices.push(*v3.offset(0));
        &out_vertices.push(*v3.offset(1));
        &out_vertices.push(*v3.offset(2));
    };

    let mut add_normals = |n1: [f32; 3], n2: [f32; 3], n3: [f32; 3]| {
        out_normals.push(n1[0]); // x
        out_normals.push(n1[1]); // y
        out_normals.push(n1[2]); // z
        out_normals.push(n2[0]);
        out_normals.push(n2[1]);
        out_normals.push(n2[2]);
        out_normals.push(n3[0]);
        out_normals.push(n3[1]);
        out_normals.push(n3[2]);
    };

    let mut add_tex_coords = |t1: *const f32, t2: *const f32, t3: *const f32| unsafe {
        out_tex_coords.push(*t1.offset(0)); // x
        out_tex_coords.push(*t1.offset(1)); // y
        out_tex_coords.push(*t2.offset(0));
        out_tex_coords.push(*t2.offset(1));
        out_tex_coords.push(*t3.offset(0));
        out_tex_coords.push(*t3.offset(1));
    };

    let mut add_indices = |i1: u32, i2: u32, i3: u32| {
        out_indices.push(i1);
        out_indices.push(i2);
        out_indices.push(i3);
    };

    let get_normal = |v1: *const f32, v2: *const f32, v3: *const f32| unsafe {
        let epsilon = 0.000001;
        let mut normal = [0.; 3];

        let ex1 = *v2.offset(0) - *v1.offset(0);
        let ey1 = *v2.offset(1) - *v1.offset(1);
        let ez1 = *v2.offset(2) - *v1.offset(2);
        let ex2 = *v3.offset(0) - *v1.offset(0);
        let ey2 = *v3.offset(1) - *v1.offset(1);
        let ez2 = *v3.offset(2) - *v1.offset(2);

        let nx = ey1 * ez2 - ez1 * ey2;
        let ny = ez1 * ex2 - ex1 * ez2;
        let nz = ex1 * ey2 - ey1 * ex2;

        let length = (nx * nx + ny * ny + nz * nz).sqrt();

        if length > epsilon {
            // normalize
            let length_inv = 1. / length;
            normal[0] = nx * length_inv;
            normal[1] = ny * length_inv;
            normal[2] = nz * length_inv;
        }

        normal
    };

    // Subdivision
    let mut index = 0;
    let mut index_count = indices.len();

    let mut j = 0;
    while j < index_count {
        // get 3 vertice and texcoords of a triangle
        v1 = &vertices[(indices[j] * 3) as usize];
        v2 = &vertices[(indices[j + 1] * 3) as usize];
        v3 = &vertices[(indices[j + 2] * 3) as usize];
        t1 = &text_coords[(indices[j] * 2) as usize];
        t2 = &text_coords[(indices[j + 1] * 2) as usize];
        t3 = &text_coords[(indices[j + 2] * 2) as usize];

        // get 3 new vertices by spliting half on each edge
        compute_half_vertex(v1, v2, &mut new_v1);
        compute_half_vertex(v2, v3, &mut new_v2);
        compute_half_vertex(v1, v3, &mut new_v3);

        compute_half_coord(t1, t2, &mut new_t1);
        compute_half_coord(t2, t3, &mut new_t2);
        compute_half_coord(t1, t3, &mut new_t3);

        // add 4 new triangles
        // add a triangle in 1st row
        add_vertices(v1, new_v1.as_ptr(), new_v3.as_ptr());
        add_tex_coords(t1, new_t1.as_ptr(), new_t3.as_ptr());
        let normal = get_normal(v1, new_v1.as_ptr(), new_v3.as_ptr());
        add_normals(normal, normal, normal);
        add_indices(index, index + 1, index + 2);

        add_vertices(new_v1.as_ptr(), v2, new_v2.as_ptr());
        add_tex_coords(new_t1.as_ptr(), t2, new_t2.as_ptr());
        let normal = get_normal(new_v1.as_ptr(), v2, new_v2.as_ptr());
        add_normals(normal, normal, normal);
        add_indices(index + 3, index + 4, index + 5);

        add_vertices(new_v1.as_ptr(), new_v2.as_ptr(), new_v3.as_ptr());
        add_tex_coords(new_t1.as_ptr(), new_t2.as_ptr(), new_t3.as_ptr());
        let normal = get_normal(new_v1.as_ptr(), new_v2.as_ptr(), new_v3.as_ptr());
        add_normals(normal, normal, normal);
        add_indices(index + 6, index + 7, index + 8);

        add_vertices(new_v3.as_ptr(), new_v2.as_ptr(), v3);
        add_tex_coords(new_t3.as_ptr(), new_t2.as_ptr(), t3);
        let normal = get_normal(new_v3.as_ptr(), new_v2.as_ptr(), v3);
        add_normals(normal, normal, normal);
        add_indices(index + 9, index + 10, index + 11);

        index += 12;
        j += 3;
    }

    (out_vertices, out_normals, out_tex_coords, out_indices)
}
