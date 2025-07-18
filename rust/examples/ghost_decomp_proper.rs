use asset_payload::{BAYER_PNG_PATH, SPHERE_PATH};
use bath::fixed_func::ghost::{
    add_dither, add_phase, load_bayer_png, spatial_phase, temporal_phase, uv_to_grid_space, warp_and_drift_cell,
    UMBRAL_MASK_CENTER, UMBRAL_MASK_FADE_BAND, UMBRAL_MASK_OUTER_RADIUS,
};
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{flip_framebuffer, N64_HEIGHT, N64_WIDTH, ORIGIN};
use bath::render::renderer::Renderer;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::consts::CameraProjection;
use raylib::drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::math::{Vector2, Vector3};
use raylib::prelude::Image;
use raylib::texture::{RaylibTexture2D, Texture2D};

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_HEIGHT);
    let screen_w = render.handle.get_screen_width();
    let screen_h = render.handle.get_screen_height();
    let i_resolution = Vector2::new(screen_w as f32, screen_h as f32);
    let mut i_time = 0.0f32;
    let circle_img = generate_circle_image(screen_w, screen_h, i_time);
    let texture = render
        .handle
        .load_texture_from_image(&render.thread, &circle_img)
        .unwrap();
    let model = render.handle.load_model(&render.thread, SPHERE_PATH).unwrap();
    let observer = Camera3D {
        position: Vector3::new(1.0, 0.0, 1.0),
        target: Vector3::ZERO,
        up: Vector3::Y,
        fovy: 80.0,
        projection: CameraProjection::CAMERA_PERSPECTIVE,
    };
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_texture_rec(
            &texture,
            flip_framebuffer(i_resolution.x, i_resolution.y),
            ORIGIN,
            Color::WHITE,
        );
        let mut rl3d = draw_handle.begin_mode3D(observer);

        rl3d.draw_model_wires_ex(
            &model,
            Vector3::new(0.25, 0.30, -0.25), // this is hand hacked in i dont know how to derive it otherwise to match where the circle is, in fact this just puts it on the mirrored side of the canvas acutally
            Vector3::Y,
            0.0,
            Vector3::new(0.25, 0.25, 0.25), //for some reason i found this to be able to make the sphere small enough, but i want to derive it from the pseudo grid space
            Color::WHITE,
        );
    }
}

#[inline]
fn generate_circle_image(width: i32, height: i32, _i_time: f32) -> Image {
    let img = Image::gen_image_color(width, height, Color::BLANK);
    let total_bytes = (width * height * 4) as usize;
    let pixels: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(img.data as *mut u8, total_bytes) };
    for y in 0..height {
        for x in 0..width {
            let s = (x as f32 + 0.5) / width as f32;
            let t = (y as f32 + 0.5) / height as f32;
            let uv = Vector2::new(s, t);
            let grid = uv_to_grid_space(uv);
            let body_radius = grid.distance(UMBRAL_MASK_CENTER);
            let lum = if body_radius <= UMBRAL_MASK_OUTER_RADIUS {
                255u8
            } else {
                0u8
            };
            let idx = 4 * (y as usize * width as usize + x as usize);
            pixels[idx] = lum; // R
            pixels[idx + 1] = lum; // G
            pixels[idx + 2] = lum; // B
            pixels[idx + 3] = 255u8; // A
        }
    }
    img
}

#[inline]
fn generate_circle_image_fade(width: i32, height: i32, i_time: f32) -> Image {
    let img = Image::gen_image_color(width, height, Color::BLANK);
    let total_bytes = (width * height * 4) as usize;
    let color: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(img.data as *mut u8, total_bytes) };
    for y in 0..height {
        for x in 0..width {
            let s = (x as f32 + 0.5) / width as f32;
            let t = (y as f32 + 0.5) / height as f32;
            let frag_tex_coord = Vector2::new(s, t);
            let mut grid_coords = uv_to_grid_space(frag_tex_coord);
            let body_radius = grid_coords.distance(UMBRAL_MASK_CENTER);
            let fade = 1.0 - {
                let outline_fade_radius = UMBRAL_MASK_OUTER_RADIUS - UMBRAL_MASK_FADE_BAND;
                let t = ((body_radius - outline_fade_radius) / (UMBRAL_MASK_OUTER_RADIUS - outline_fade_radius))
                    .clamp(0.0, 1.0);
                t * t * (3.0 - 2.0 * t)
            };
            let lum = (fade.clamp(0.0, 1.0) * 255.0) as u8;
            let idx = 4 * (y as usize * width as usize + x as usize);
            color[idx] = lum; // R
            color[idx + 1] = lum; // G
            color[idx + 2] = lum; // B
            color[idx + 3] = 255; // A
        }
    }
    img
}

#[inline]
fn dither_image_bayer(render: &mut RaylibRenderer, image: Image, w: i32, h: i32) -> Texture2D {
    let blank_img = Image::gen_image_color(w, h, Color::BLANK);
    let mut texture = render
        .handle
        .load_texture_from_image(&render.thread, &blank_img)
        .unwrap();
    let (bayer_data, bayer_w, bayer_h) = load_bayer_png(BAYER_PNG_PATH);
    let mut colors: Vec<u8> = image.get_image_data_u8(false);
    for y in 0..image.height() {
        for x in 0..image.width() {
            let idx = 4 * (y * image.width() + x) as usize;
            let lum = colors[idx] as f32 / 255.0;
            let dither = add_dither(lum, x, y, &bayer_data, bayer_w, bayer_h);
            let v = (dither * 255.0).round() as u8;
            colors[idx] = v; // R
            colors[idx + 1] = v; // G
            colors[idx + 2] = v; // B
            colors[idx + 3] = 255; // A
        }
    }
    texture.update_texture(&colors).unwrap();
    texture
}
