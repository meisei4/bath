use asset_payload::payloads::BAYER_PNG;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{N64_HEIGHT, N64_WIDTH};
use bath::render::renderer::Renderer;
use raylib::consts::BlendMode;
use raylib::drawing::{RaylibBlendModeExt, RaylibTextureModeExt};
use raylib::math::{Rectangle, Vector2};
use raylib::{color::Color, drawing::RaylibDraw, prelude::Image};

const HALF: f32 = 0.5;
const GRID_SCALE: f32 = 4.0;
const GRID_CELL_SIZE: f32 = 1.0 / GRID_SCALE;
const GRID_ORIGIN_INDEX: Vector2 = Vector2::new(0.0, 0.0);
const GRID_ORIGIN_OFFSET_CELLS: Vector2 = Vector2::new(2.0, 2.0);
const GRID_ORIGIN_UV_OFFSET: Vector2 = Vector2::new(
    (GRID_ORIGIN_INDEX.x + GRID_ORIGIN_OFFSET_CELLS.x) * GRID_CELL_SIZE,
    (GRID_ORIGIN_INDEX.y + GRID_ORIGIN_OFFSET_CELLS.y) * GRID_CELL_SIZE,
);
const CELL_DRIFT_AMPLITUDE: f32 = 0.2;
const LIGHT_WAVE_SPATIAL_FREQ_X: f32 = 8.0;
const LIGHT_WAVE_SPATIAL_FREQ_Y: f32 = 8.0;
const LIGHT_WAVE_TEMPORAL_FREQ_X: f32 = 80.0;
const LIGHT_WAVE_TEMPORAL_FREQ_Y: f32 = 2.3;
const LIGHT_WAVE_AMPLITUDE_X: f32 = 0.03;
const LIGHT_WAVE_AMPLITUDE_Y: f32 = 0.1;
const UMBRAL_MASK_OUTER_RADIUS: f32 = 0.40;
const UMBRAL_MASK_INNER_RADIUS: f32 = 0.08;
const UMBRAL_MASK_FADE_BAND: f32 = 0.025;
const UMBRAL_MASK_CENTER: Vector2 = Vector2::new(HALF, HALF);
const UMBRAL_MASK_OFFSET_X: f32 = -UMBRAL_MASK_OUTER_RADIUS;
const UMBRAL_MASK_OFFSET_Y: f32 = -UMBRAL_MASK_OUTER_RADIUS;
const UMBRAL_MASK_PHASE_COEFFICIENT_X: f32 = 0.6;
const UMBRAL_MASK_PHASE_COEFFICIENT_Y: f32 = 0.2;
const UMBRAL_MASK_WAVE_AMPLITUDE_X: f32 = 0.1;
const UMBRAL_MASK_WAVE_AMPLITUDE_Y: f32 = 0.1;
const DITHER_TEXTURE_SCALE: f32 = 2.0;
const DITHER_BLEND_FACTOR: f32 = 0.75;

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn make_radial_lut() -> Image {
    const SIZE: i32 = 64;
    let mut img = Image::gen_image_color(SIZE, SIZE, Color::BLANK);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let u = (x as f32 + 0.5) / SIZE as f32 - 0.5;
            let v = (y as f32 + 0.5) / SIZE as f32 - 0.5;
            let d = (u * u + v * v).sqrt();
            let a = 1.0
                - smoothstep(
                    UMBRAL_MASK_OUTER_RADIUS - UMBRAL_MASK_FADE_BAND,
                    UMBRAL_MASK_OUTER_RADIUS,
                    d,
                );
            img.draw_pixel(
                x,
                y,
                Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: (a * 255.0).round() as u8,
                },
            );
        }
    }
    img
}

fn uv_to_grid_space(uv: Vector2) -> Vector2 {
    (uv - GRID_ORIGIN_UV_OFFSET) * GRID_SCALE
}

fn warp_and_drift_cell(gc: Vector2, t: f32) -> Vector2 {
    CELL_DRIFT_AMPLITUDE * Vector2::new((t + gc.y).sin(), (t + gc.x).sin())
}

fn spatial_phase(gc: Vector2) -> Vector2 {
    Vector2::new(gc.y * LIGHT_WAVE_SPATIAL_FREQ_X, gc.x * LIGHT_WAVE_SPATIAL_FREQ_Y)
}

fn temporal_phase(t: f32) -> Vector2 {
    Vector2::new(t * LIGHT_WAVE_TEMPORAL_FREQ_X, t * LIGHT_WAVE_TEMPORAL_FREQ_Y)
}

fn add_phase(p: Vector2) -> Vector2 {
    Vector2::new(LIGHT_WAVE_AMPLITUDE_X * p.x.cos(), LIGHT_WAVE_AMPLITUDE_Y * p.y.sin())
}

fn light_radial_fade(gc: Vector2) -> f32 {
    let d = gc.distance(UMBRAL_MASK_CENTER);
    1.0 - smoothstep(
        UMBRAL_MASK_OUTER_RADIUS - UMBRAL_MASK_FADE_BAND,
        UMBRAL_MASK_OUTER_RADIUS,
        d,
    )
}

fn add_umbral_mask_phase(time: f32) -> Vector2 {
    Vector2::new(
        UMBRAL_MASK_WAVE_AMPLITUDE_X * LIGHT_WAVE_SPATIAL_FREQ_X,
        UMBRAL_MASK_WAVE_AMPLITUDE_Y * LIGHT_WAVE_SPATIAL_FREQ_Y + time * LIGHT_WAVE_TEMPORAL_FREQ_Y,
    )
}

fn umbral_mask_position(pc_x: f32, pc_y: f32, mp: Vector2) -> Vector2 {
    Vector2::new(pc_x * mp.x.cos(), pc_y * mp.y.sin()) + UMBRAL_MASK_CENTER
}

fn add_umbral_mask(src: f32, gc: Vector2, mc: Vector2) -> f32 {
    let dist = gc.distance(mc + Vector2::new(UMBRAL_MASK_OFFSET_X, UMBRAL_MASK_OFFSET_Y)) * HALF;
    src * smoothstep(UMBRAL_MASK_INNER_RADIUS, UMBRAL_MASK_OUTER_RADIUS, dist)
}

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_HEIGHT);
    let radial_texture = render
        .handle
        .load_texture_from_image(&render.thread, &make_radial_lut())
        .unwrap();
    let i_channel_0 = render.load_texture(BAYER_PNG(), "png");
    let i_resolution = Vector2::new(N64_WIDTH as f32, N64_HEIGHT as f32);
    let cell_coord = Vector2::new(i_resolution.x / GRID_SCALE, i_resolution.y / GRID_SCALE);
    let mut i_time = 0.0_f32;
    let mut ghost_render_texture = render
        .handle
        .load_render_texture(&render.thread, N64_WIDTH as u32, N64_HEIGHT as u32)
        .unwrap();
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        {
            let mut texture_mode = render
                .handle
                .begin_texture_mode(&render.thread, &mut ghost_render_texture);
            texture_mode.clear_background(Color::BLANK);
            for gy in -1..=(GRID_SCALE as i32) {
                for gx in -1..=(GRID_SCALE as i32) {
                    let cell_origin = Vector2::new((gx as f32 + 0.5) * cell_coord.x, (gy as f32 + 0.5) * cell_coord.y);
                    let uv = cell_origin / i_resolution;
                    let mut grid_coords = uv_to_grid_space(uv);
                    let mut phase = spatial_phase(grid_coords);
                    phase += temporal_phase(i_time);
                    grid_coords += add_phase(phase);
                    let drift_uv = warp_and_drift_cell(grid_coords, i_time);
                    grid_coords += drift_uv;
                    let origin_px = cell_origin + drift_uv * cell_coord;
                    let phase_warp = spatial_phase(grid_coords) + temporal_phase(i_time);
                    let sx = 1.0 + LIGHT_WAVE_AMPLITUDE_X * phase_warp.x.cos();
                    let sy = 1.0 + LIGHT_WAVE_AMPLITUDE_Y * phase_warp.y.sin();
                    let mut lum = light_radial_fade(grid_coords);
                    lum = add_umbral_mask(
                        lum,
                        grid_coords,
                        umbral_mask_position(
                            UMBRAL_MASK_PHASE_COEFFICIENT_X,
                            UMBRAL_MASK_PHASE_COEFFICIENT_Y,
                            add_umbral_mask_phase(i_time),
                        ),
                    );
                    let l = (lum * 255.0).round() as u8;
                    let tint = Color {
                        r: l,
                        g: l,
                        b: l,
                        a: 255,
                    };
                    let dst = Rectangle {
                        x: origin_px.x - cell_coord.x * 0.5 * sx,
                        y: origin_px.y - cell_coord.y * 0.5 * sy,
                        width: cell_coord.x * sx,
                        height: cell_coord.y * sy,
                    };
                    let src = Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: radial_texture.width as f32,
                        height: -(radial_texture.height as f32),
                    };
                    texture_mode.draw_texture_pro(&radial_texture, src, dst, Vector2::ZERO, 0.0, tint);
                }
            }
        }
        let mut draw_handle = render.handle.begin_drawing(&render.thread);
        draw_handle.clear_background(Color::BLACK);
        draw_handle.draw_texture(&ghost_render_texture, 0, 0, Color::WHITE);
        let alpha = (DITHER_BLEND_FACTOR * 255.0).round() as u8;
        let tint = Color {
            r: 255,
            g: 255,
            b: 255,
            a: alpha,
        };
        let tile = Vector2::new(
            i_channel_0.width as f32 * DITHER_TEXTURE_SCALE,
            i_channel_0.height as f32 * DITHER_TEXTURE_SCALE,
        );
        for ty in 0..=((N64_HEIGHT as f32 / tile.y).ceil() as i32) {
            for tx in 0..=((N64_WIDTH as f32 / tile.x).ceil() as i32) {
                let dst = Rectangle {
                    x: tx as f32 * tile.x,
                    y: ty as f32 * tile.y,
                    width: tile.x,
                    height: tile.y,
                };
                let src = Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: i_channel_0.width as f32,
                    height: i_channel_0.height as f32,
                };
                let mut blend_mode = draw_handle.begin_blend_mode(BlendMode::BLEND_MULTIPLIED);
                blend_mode.draw_texture_pro(&i_channel_0, src, dst, Vector2::ZERO, 0.0, tint);
            }
        }
    }
}
