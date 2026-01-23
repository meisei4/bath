use raylib::prelude::*;

const OUTPUT_BASE: &str = "/home/adduser/fu4seoi3/src/fu4seoi3/romdisk/assets";
const SIZES: &[i32] = &[128, 256, 512];

fn n(a: f32, b: f32, s: i32) -> f32 {
    let x = a as i32;
    let y = b as i32;
    let u_raw = a - x as f32;
    let v_raw = b - y as f32;

    let mut q = [0.0f32; 4];
    for i in 0..4 {
        let dx = (i & 1) as i32;
        let dy = (i >> 1) as i32;
        let mut h = ((x + dx) as u32)
            .wrapping_mul(374761393)
            .wrapping_add(((y + dy) as u32).wrapping_mul(668265263))
            .wrapping_add(s as u32);
        h ^= h >> 13;
        h = h.wrapping_mul(1274126177);
        h ^= h >> 16;
        q[i] = (h & 0xFFFF) as f32 / 65535.0;
    }

    let u = u_raw * u_raw * (3.0 - 2.0 * u_raw);
    let v = v_raw * v_raw * (3.0 - 2.0 * v_raw);

    q[0] + (q[1] - q[0]) * u + (q[2] - q[0]) * v + (q[0] - q[1] - q[2] + q[3]) * u * v
}

fn generate_rings(w: i32, h: i32) -> Image {
    let mut image = Image::gen_image_color(w, h, Color::WHITE);
    let c = w as f32 * 0.5;
    let d = h as f32 * 0.5;
    let r = c.min(d);

    let saddle_brown = Color::new(139, 69, 19, 255);
    let wheat = Color::new(245, 222, 179, 255);

    for j in 0..h {
        for i in 0..w {
            let dx = i as f32 - c;
            let dy = j as f32 - d;
            let e = (dx * dx + dy * dy).sqrt() / r;
            let ring_index = (e * 8.0) as i32;
            let color = if (ring_index & 1) == 1 {
                saddle_brown
            } else {
                wheat
            };
            image.draw_pixel(i, j, color);
        }
    }

    image
}

fn generate_aged(w: i32, h: i32) -> Image {
    let mut image = Image::gen_image_color(w, h, Color::WHITE);

    for j in 0..h {
        for i in 0..w {
            let u = i as f32 / w as f32;
            let v = j as f32 / h as f32;

            let bn = n(u * 2.0, v * 2.0, 42) * 0.5
                + n(u * 4.0, v * 4.0, 43) * 0.25
                + n(u * 8.0, v * 8.0, 44) * 0.25;

            let br = 255.0 + (248.0 - 255.0) * bn;
            let bg = 255.0 + (248.0 - 255.0) * bn;
            let bb = 240.0 + (220.0 - 240.0) * bn;

            let fn_val = n(u * 8.0, v * 8.0, 123) * 0.5
                + n(u * 16.0, v * 16.0, 124) * 0.25
                + n(u * 32.0, v * 32.0, 125) * 0.25;
            let fm = n(u * 4.0, v * 4.0, 456);

            let fs = if fn_val > 0.65 && fm > 0.5 {
                let t = (fn_val - 0.65) / 0.35;
                t * t
            } else {
                0.0
            };

            let ex = (u - 0.5).abs() * 2.0;
            let ey = (v - 0.5).abs() * 2.0;
            let ex4 = ex * ex * ex * ex;
            let ey4 = ey * ey * ey * ey;
            let ed = (ex4 + ey4).powf(0.25);
            let vig = if ed > 0.8 { (ed - 0.8) / 0.2 * 0.2 } else { 0.0 };

            let mut r_out = br / 255.0;
            let mut g_out = bg / 255.0;
            let mut b_out = bb / 255.0;

            r_out = r_out * (1.0 - fs) + (47.0 / 255.0) * fs;
            g_out = g_out * (1.0 - fs) + (31.0 / 255.0) * fs;
            b_out = b_out * (1.0 - fs) + (20.0 / 255.0) * fs;

            r_out *= 1.0 - vig;
            g_out *= 1.0 - vig;
            b_out *= 1.0 - vig;

            let color = Color::new(
                (r_out * 255.0) as u8,
                (g_out * 255.0) as u8,
                (b_out * 255.0) as u8,
                255,
            );
            image.draw_pixel(i, j, color);
        }
    }

    image
}

fn main() {
    println!("Procedural Texture Generator for fu4seoi3");
    println!("Output: {}", OUTPUT_BASE);
    println!();

    let (mut rl, thread) = raylib::init().size(100, 100).title("Generator").build();

    for &size in SIZES {
        println!("Generating aged_{}...", size);
        let mut aged = generate_aged(size, size);
        aged.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
        let path = format!("{}/aged_{}.png", OUTPUT_BASE, size);
        aged.export_image(&path);
        println!("  Exported: {}", path);
    }

    for &size in SIZES {
        println!("Generating rings_{}...", size);
        let mut rings = generate_rings(size, size);
        rings.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
        let path = format!("{}/rings_{}.png", OUTPUT_BASE, size);
        rings.export_image(&path);
        println!("  Exported: {}", path);
    }

    println!();
    println!("Done! Generated 6 textures.");
}
