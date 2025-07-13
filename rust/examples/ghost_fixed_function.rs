use asset_payload::payloads::BAYER_PNG;
use bath::render::raylib::RaylibRenderer;
use bath::render::raylib_util::{N64_HEIGHT, N64_WIDTH};
use bath::render::renderer::Renderer;
use raylib::color::Color;
use raylib::drawing::RaylibDraw;
use raylib::ffi::{
    rlActiveTextureSlot, rlBegin, rlColor4f, rlDisableColorBlend, rlEnableColorBlend, rlEnableTexture, rlEnd,
    rlLoadIdentity, rlMatrixMode, rlPopMatrix, rlPushMatrix, rlScalef, rlSetBlendFactors, rlTexCoord2f, rlTranslatef,
    rlVertex3f, RL_BLEND_EQUATION, RL_ONE, RL_ONE_MINUS_SRC_COLOR, RL_QUADS, RL_TEXTURE,
};
use raylib::math::Vector2;

fn main() {
    let mut render = RaylibRenderer::init(N64_WIDTH, N64_HEIGHT);
    let (_w, _h) = (render.handle.get_screen_width(), render.handle.get_screen_height());
    let mut i_time = 0.0f32;
    let radial_texture_id = make_radial_tex(&mut render, 64);
    let mask_texture_id = make_mask_tex(&mut render, 32);
    let mut dither_texture = render.load_texture(BAYER_PNG(), "png");
    render.tweak_texture_parameters(&mut dither_texture, true, true);
    while !render.handle.window_should_close() {
        i_time += render.handle.get_frame_time();
        let mut draw = render.handle.begin_drawing(&render.thread);
        draw.clear_background(Color::BLACK);
        unsafe {
            rlActiveTextureSlot(0);
            rlEnableTexture(radial_texture_id);
            rlMatrixMode(RL_TEXTURE as i32);
            rlPushMatrix();
            rlLoadIdentity();
            let scroll = radial_texture_scroll(i_time);
            rlTranslatef(scroll.x, scroll.y, 0.0);
            let scale = pulse_scale(i_time);
            rlScalef(scale, scale, 1.0);

            rlActiveTextureSlot(1);
            rlEnableTexture(mask_texture_id);
            rlMatrixMode(RL_TEXTURE as i32);
            rlPushMatrix();
            rlLoadIdentity();
            let mask_off = umbral_mask_scroll(i_time);
            rlTranslatef(mask_off.x, mask_off.y, 0.0);
            // ffi::glTexEnvi(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_REPLACE as i32, );
            rlActiveTextureSlot(1);
            // ffi::glTexEnvi(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_MODULATE as i32);
            rlActiveTextureSlot(0);
            draw_fullscreen_quad_rl();
            rlActiveTextureSlot(1);
            rlPopMatrix();
            rlActiveTextureSlot(0);
            rlPopMatrix();
        }

        unsafe {
            rlActiveTextureSlot(0);
            rlEnableTexture(dither_texture.id);
            rlMatrixMode(RL_TEXTURE as i32);
            rlPushMatrix();
            rlLoadIdentity();
            rlScalef(1.0 / DITHER_TEXTURE_SCALE, 1.0 / DITHER_TEXTURE_SCALE, 1.0);

            rlEnableColorBlend();
            rlSetBlendFactors(RL_ONE as i32, RL_ONE_MINUS_SRC_COLOR as i32, RL_BLEND_EQUATION as i32);
            rlColor4f(1.0, 1.0, 1.0, DITHER_BLEND_FACTOR);
            draw_fullscreen_quad_rl();

            rlDisableColorBlend();
            rlPopMatrix();
        }
    }
}

const TAU: f32 = 6.283185307179586;
const HALF: f32 = 0.5;

const GRID_SCALE: f32 = 4.0;
const CELL_DRIFT_AMPLITUDE: f32 = 0.2;

const LIGHT_WAVE_SPATIAL_FREQ_X: f32 = 8.0;
const LIGHT_WAVE_SPATIAL_FREQ_Y: f32 = 8.0;
const LIGHT_WAVE_TEMPORAL_FREQ_X: f32 = 80.0;
const LIGHT_WAVE_TEMPORAL_FREQ_Y: f32 = 2.3;
const LIGHT_WAVE_AMPLITUDE_X: f32 = 0.03;
const LIGHT_WAVE_AMPLITUDE_Y: f32 = 0.10;

const LIGHTBALL_OUTER_RADIUS: f32 = 0.40;
const LIGHTBALL_CENTER: Vector2 = Vector2 { x: HALF, y: HALF };
const LIGHTBALL_FADE_BAND: f32 = 0.025;

const UMBRAL_MASK_INNER_RADIUS: f32 = 0.08;
const UMBRAL_MASK_OFFSET_X: f32 = -(LIGHTBALL_OUTER_RADIUS / 1.0);
const UMBRAL_MASK_OFFSET_Y: f32 = -LIGHTBALL_OUTER_RADIUS;
const UMBRAL_MASK_PHASE_COEFFICIENT_X: f32 = 0.6;
const UMBRAL_MASK_PHASE_COEFFICIENT_Y: f32 = 0.2;
const UMBRAL_MASK_WAVE_AMPLITUDE_X: f32 = 0.1;
const UMBRAL_MASK_WAVE_AMPLITUDE_Y: f32 = 0.1;

const DITHER_TEXTURE_SCALE: f32 = 8.0;
const DITHER_BLEND_FACTOR: f32 = 0.75;

#[inline]
fn spatial_phase(grid: Vector2) -> Vector2 {
    Vector2 {
        x: grid.y * LIGHT_WAVE_SPATIAL_FREQ_X,
        y: grid.x * LIGHT_WAVE_SPATIAL_FREQ_Y,
    }
}

#[inline]
fn temporal_phase(i_time: f32) -> Vector2 {
    Vector2 {
        x: i_time * LIGHT_WAVE_TEMPORAL_FREQ_X,
        y: i_time * LIGHT_WAVE_TEMPORAL_FREQ_Y,
    }
}

#[inline]
fn add_phase(phase: Vector2) -> Vector2 {
    Vector2 {
        x: LIGHT_WAVE_AMPLITUDE_X * phase.x.cos(),
        y: LIGHT_WAVE_AMPLITUDE_Y * phase.y.sin(),
    }
}

fn radial_texture_scroll(i_time: f32) -> Vector2 {
    let phase = spatial_phase(Vector2::new(0.0, 0.0)) + temporal_phase(i_time);
    add_phase(phase)
}

fn pulse_scale(i_time: f32) -> f32 {
    let beat_phase = (i_time * 2.0).fract();
    0.8 + 0.4 * ((beat_phase * TAU).sin() * 0.5 + 0.5)
}

fn umbral_mask_scroll(i_time: f32) -> Vector2 {
    let phase_x = UMBRAL_MASK_WAVE_AMPLITUDE_X * LIGHT_WAVE_SPATIAL_FREQ_X;
    let phase_y = UMBRAL_MASK_WAVE_AMPLITUDE_Y * LIGHT_WAVE_SPATIAL_FREQ_Y + i_time * LIGHT_WAVE_TEMPORAL_FREQ_Y;

    Vector2 {
        x: UMBRAL_MASK_PHASE_COEFFICIENT_X * phase_x.cos() + UMBRAL_MASK_OFFSET_X,
        y: UMBRAL_MASK_PHASE_COEFFICIENT_Y * phase_y.sin() + UMBRAL_MASK_OFFSET_Y,
    }
}

fn make_radial_tex(render: &mut RaylibRenderer, size: i32) -> u32 {
    let mut img = raylib::texture::Image::gen_image_color(size, size, Color::BLANK);
    let center = Vector2 {
        x: size as f32 * 0.5,
        y: size as f32 * 0.5,
    };
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center.x;
            let dy = y as f32 - center.y;
            let dist = (dx * dx + dy * dy).sqrt() / (size as f32 * 0.5);
            let mut min_val = f32::INFINITY;
            let steps = 100;
            for i in 0..=steps {
                let r = LIGHTBALL_OUTER_RADIUS - LIGHTBALL_FADE_BAND + (i as f32 / steps as f32) * LIGHTBALL_FADE_BAND;
                let val = (dist - r).max(0.0);
                if val < min_val {
                    min_val = val;
                }
            }
            let alpha = 1.0 - (min_val / LIGHTBALL_FADE_BAND);
            let val = (alpha.clamp(0.0, 1.0) * 255.0) as u8;
            img.draw_pixel(x, y, Color::new(val, val, val, val));
        }
    }

    let tex = render.handle.load_texture_from_image(&render.thread, &img).unwrap();
    tex.id
}

fn make_mask_tex(render: &mut RaylibRenderer, size: i32) -> u32 {
    let mut img = raylib::texture::Image::gen_image_color(size, size, Color::BLANK);
    let center = Vector2 {
        x: size as f32 * 0.5,
        y: size as f32 * 0.5,
    };
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center.x;
            let dy = y as f32 - center.y;
            let dist = ((dx * dx + dy * dy).sqrt() / (size as f32 * 0.5)).powf(0.5); // softer γ
            let alpha = 1.0 - dist.clamp(0.0, 1.0);
            let val = (alpha * 255.0) as u8;
            img.draw_pixel(x, y, Color::new(val, val, val, val));
        }
    }
    render.handle.load_texture_from_image(&render.thread, &img).unwrap().id
}

unsafe fn draw_fullscreen_quad_rl() {
    rlBegin(RL_QUADS as i32);
    rlTexCoord2f(0.0, 1.0);
    rlVertex3f(0.0, 1.0, 0.0);
    rlTexCoord2f(0.0, 0.0);
    rlVertex3f(0.0, 0.0, 0.0);
    rlTexCoord2f(1.0, 0.0);
    rlVertex3f(1.0, 0.0, 0.0);
    rlTexCoord2f(1.0, 1.0);
    rlVertex3f(1.0, 1.0, 0.0);
    rlEnd();
}
