use crate::cube::{Line2D, Ray};

// http://geomalgorithms.com/a07-_distance.html
pub fn ray_ray_distance(r1: &Ray, r2: &Ray) -> f32 {
    let u = &r1.dir - &r1.origin;
    let v = &r2.dir - &r2.origin;
    let w = &r1.origin - &r2.origin;

    let a = glm::dot(&u, &u);
    let b = glm::dot(&u, &v);
    let c = glm::dot(&v, &v);
    let d = glm::dot(&u, &w);
    let e = glm::dot(&v, &w);

    let D = a * c - b * b;
    let sc: f32;
    let tc: f32;

    if D < glm::epsilon() {
        // the lines are almost parallel
        sc = 0.;
        tc = if b > c { d / b } else { e / c };
    } else {
        sc = (b * e - c * d) / D;
        tc = (a * e - b * d) / D;
    }

    // get the difference of the two closest points
    let dP: glm::Vec3 = w + (u * sc) - (v * tc);

    dP.magnitude()
}

pub fn is_rays_intersect(r1: &Ray, r2: &Ray) -> bool {
    ray_ray_distance(r1, r2) < glm::epsilon()
}

pub fn is_point_on_line2D(line: &Line2D, point: &glm::Vec2) -> bool {
    let dxc = point[0] - line.from[0];
    let dyc = point[1] - line.from[1];

    let dxl = line.to[0] - line.from[0];
    let dyl = line.to[1] - line.from[1];

    let cross = dxc * dyl - dyc * dxl;

    let treshold = 0.05;

    if cross.abs() > treshold {
        false
    } else {
        if dxl.abs() >= dyl.abs() {
            if dxl > 0. {
                line.from[0] <= point[0] && point[0] <= line.to[0]
            } else {
                line.to[0] <= point[0] && point[0] <= line.from[0]
            }
        } else {
            if dyl > 0. {
                line.from[1] <= point[1] && point[1] <= line.to[1]
            } else {
                line.to[1] <= point[1] && point[1] <= line.from[1]
            }
        }
    }

    // println!(
    //     "point {}",
    //     &(glm::distance(&line.from, &point) + glm::distance(&point, &line.to)
    //         - glm::distance(&line.from, &line.to))
    // );
    //
    // glm::distance(&line.from, &point) + glm::distance(&point, &line.to)
    //     - glm::distance(&line.from, &line.to)
    //     < 0.01
}
