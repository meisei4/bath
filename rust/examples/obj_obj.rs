use std::f64::consts::PI;

enum TexcoordMapping {
    PlanarProjectionXY,
    SphericalEquirectangularAnalytic,
    SphericalEquirectangularUnwrapped,
}

fn main() {
    let radius = 0.5;
    let stacks = 8;
    let slices = 8;
    let mode = TexcoordMapping::SphericalEquirectangularUnwrapped;
    // let mode = TexcoordMapping::SphericalEquirectangularAnalytic;
    // let mode = TexcoordMapping::PlanarProjectionXY;

    println!("o sphere");
    let mut verts = Vec::new();
    verts.push([0.0, radius, 0.0]);

    for i in 1..stacks {
        let t = PI * (i as f64) / (stacks as f64);
        let y = radius * t.cos();
        let r = radius * t.sin();

        for j in 0..slices {
            let p = 2.0 * PI * (j as f64) / (slices as f64);
            let x = r * p.cos();
            let z = r * p.sin();
            verts.push([x, y, z]);
        }
    }
    verts.push([0.0, -radius, 0.0]);
    for v in &verts {
        println!("v {:.6} {:.6} {:.6}", v[0], v[1], v[2]);
    }
    emit_sphere_texcoords(&verts, stacks, slices, &mode);
    println!("s off");
    emit_sphere_indexed_triangles(stacks, slices, &mode);
}

fn emit_sphere_texcoords(verts: &[[f64; 3]], stacks: usize, slices: usize, mode: &TexcoordMapping) {
    match mode {
        TexcoordMapping::PlanarProjectionXY => {
            for v in verts {
                let (s, t) = planar_projection_xy_st(v[0], v[1], v[2]);
                println!("vt {:.6} {:.6}", s, t);
            }
        },
        TexcoordMapping::SphericalEquirectangularAnalytic => {
            for v in verts {
                let (s, t) = spherical_equirectangular_analytic_st(v[0], v[1], v[2]);
                println!("vt {:.6} {:.6}", s, t);
            }
        },
        TexcoordMapping::SphericalEquirectangularUnwrapped => {
            emit_sphere_equirectangular_unwrapped_texcoords(stacks, slices);
        },
    }
}

fn emit_sphere_indexed_triangles(stacks: usize, slices: usize, mode: &TexcoordMapping) {
    match mode {
        TexcoordMapping::PlanarProjectionXY | TexcoordMapping::SphericalEquirectangularAnalytic => {
            emit_indexed_triangles_shared_texcoords(stacks, slices);
        },
        TexcoordMapping::SphericalEquirectangularUnwrapped => {
            emit_indexed_triangles_equirectangular_unwrapped(stacks, slices);
        },
    }
}

fn emit_indexed_triangles_shared_texcoords(stacks: usize, slices: usize) {
    let top = 1;
    let rings = stacks - 1;
    let bottom = 2 + (rings * slices);
    for j in 0..slices {
        let k = (j + 1) % slices;
        let a = 2 + j;
        let b = 2 + k;
        println!("f {0}/{0} {1}/{1} {2}/{2}", top, b, a);
    }

    for s in 0..(rings - 1) {
        for j in 0..slices {
            let k = (j + 1) % slices;
            let u0 = 2 + s * slices + j;
            let u1 = 2 + s * slices + k;
            let l0 = 2 + (s + 1) * slices + j;
            let l1 = 2 + (s + 1) * slices + k;
            println!("f {0}/{0} {1}/{1} {2}/{2}", u0, u1, l0);
            println!("f {0}/{0} {1}/{1} {2}/{2}", u1, l1, l0);
        }
    }

    let base = 2 + (rings - 1) * slices;
    for j in 0..slices {
        let k = (j + 1) % slices;
        let a = base + j;
        let b = base + k;
        println!("f {0}/{0} {1}/{1} {2}/{2}", bottom, a, b);
    }
}

fn emit_sphere_equirectangular_unwrapped_texcoords(stacks: usize, slices: usize) {
    for j in 0..slices {
        let s = (j as f64 + 0.5) / (slices as f64);
        println!("vt {:.6} {:.6}", s, 1.0);
    }

    for i in 1..stacks {
        let t = 1.0 - (i as f64) / (stacks as f64);

        for j in 0..=slices {
            let s = (j as f64) / (slices as f64);
            println!("vt {:.6} {:.6}", s, t);
        }
    }

    for j in 0..slices {
        let s = (j as f64 + 0.5) / (slices as f64);
        println!("vt {:.6} {:.6}", s, 0.0);
    }
}

fn emit_indexed_triangles_equirectangular_unwrapped(stacks: usize, slices: usize) {
    let rings = stacks - 1;
    for j in 0..slices {
        let v_a = 2 + j;
        let v_b = 2 + ((j + 1) % slices);
        let st_top = 1 + j;
        let st_a = slices + 1 + j;
        let st_b = slices + 1 + j + 1;
        println!("f {}/{} {}/{} {}/{}", 1, st_top, v_b, st_b, v_a, st_a);
    }

    for s in 0..(rings - 1) {
        let ring_st_base = slices + 1 + s * (slices + 1);
        let next_ring_st_base = slices + 1 + (s + 1) * (slices + 1);
        for j in 0..slices {
            let v_u0 = 2 + s * slices + j;
            let v_u1 = 2 + s * slices + ((j + 1) % slices);
            let v_l0 = 2 + (s + 1) * slices + j;
            let v_l1 = 2 + (s + 1) * slices + ((j + 1) % slices);
            let st_u0 = ring_st_base + j;
            let st_u1 = ring_st_base + j + 1;
            let st_l0 = next_ring_st_base + j;
            let st_l1 = next_ring_st_base + j + 1;
            println!("f {}/{} {}/{} {}/{}", v_u0, st_u0, v_u1, st_u1, v_l0, st_l0);
            println!("f {}/{} {}/{} {}/{}", v_u1, st_u1, v_l1, st_l1, v_l0, st_l0);
        }
    }

    let base_ring_st = 1 + slices + (rings - 1) * (slices + 1);
    let bottom_pole_st_base = 1 + slices + rings * (slices + 1);
    for j in 0..slices {
        let v_a = 2 + (rings - 1) * slices + j;
        let v_b = 2 + (rings - 1) * slices + ((j + 1) % slices);
        let v_bottom = 2 + rings * slices;
        let st_a = base_ring_st + j;
        let st_b = base_ring_st + j + 1;
        let st_bottom = bottom_pole_st_base + j;

        println!("f {}/{} {}/{} {}/{}", v_bottom, st_bottom, v_a, st_a, v_b, st_b);
    }
}

fn spherical_equirectangular_analytic_st(x: f64, y: f64, z: f64) -> (f64, f64) {
    let r = (x * x + y * y + z * z).sqrt();
    if r == 0.0 {
        return (0.0, 0.0);
    }
    let mut s = f64::atan2(z, x) / (2.0 * PI);
    if s < 0.0 {
        s += 1.0;
    }
    let ny = (y / r).clamp(-1.0, 1.0);
    let t = 1.0 - ny.acos() / PI;
    (s, t)
}

fn planar_projection_xy_st(x: f64, y: f64, _z: f64) -> (f64, f64) {
    let s = (x + 0.5) / 1.0;
    let t = (y + 0.5) / 1.0;
    (s, t)
}
