use crate::cube::Ray;

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
