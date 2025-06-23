use crate::render::raylib_util::{
    create_rgba16_render_texture, load_shader_with_includes, APPLE_DPI, ORIGIN, ORIGIN_X, ORIGIN_Y, WINDOW_HEIGHT,
    WINDOW_WIDTH,
};
use crate::render::renderer::{Renderer, RendererVector2};
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibShaderModeExt, RaylibTextureModeExt};
use raylib::ffi::{
    rlTextureParameters, LoadImage, LoadTextureFromImage, UnloadImage, RL_TEXTURE_FILTER_NEAREST,
    RL_TEXTURE_MAG_FILTER, RL_TEXTURE_MIN_FILTER, RL_TEXTURE_WRAP_REPEAT, RL_TEXTURE_WRAP_S, RL_TEXTURE_WRAP_T,
};
use raylib::math::Rectangle;
use raylib::shaders::{RaylibShader, Shader};
use raylib::texture::{RenderTexture2D, Texture2D};
use raylib::{init, RaylibHandle, RaylibThread};
use std::ffi::CString;
pub struct RaylibRenderer {
    pub handle: RaylibHandle,
    thread: RaylibThread,
}

impl Renderer for RaylibRenderer {
    type Shader = Shader;
    type Texture = Texture2D;
    type RenderTarget = RenderTexture2D;

    fn init() -> Self {
        let (mut handle, thread) = init()
            .size(WINDOW_WIDTH / APPLE_DPI, WINDOW_HEIGHT / APPLE_DPI)
            .title("drekker effect")
            .build();
        handle.set_target_fps(60);
        let screen_width = handle.get_screen_width();
        let screen_height = handle.get_screen_height();
        let dpi = handle.get_window_scale_dpi();
        let render_width = handle.get_render_width();
        let render_height = handle.get_render_height();
        println!("screen: {}x{}", screen_width, screen_height);
        println!("render:{}x{}", render_width, render_height);
        println!("dpi: {:?}", dpi);
        Self {
            handle,
            thread,
        }
    }

    fn load_shader(&mut self, path: &str) -> Self::Shader {
        let source = load_shader_with_includes(path);
        self.handle.load_shader_from_memory(&self.thread, None, Some(&source))
    }

    fn load_texture(&mut self, path: &str) -> Self::Texture {
        let project_root_dir = env!("CARGO_MANIFEST_DIR");
        let image_path = CString::new(format!("{}{}", project_root_dir, path)).unwrap();
        let image_texture = unsafe {
            let image_raw = LoadImage(image_path.as_ptr());
            let image_texture_raw = LoadTextureFromImage(image_raw);
            UnloadImage(image_raw);
            Texture2D::from_raw(image_texture_raw)
        };
        image_texture
        //self.handle.load_texture(&self.thread, path).unwrap()
    }

    fn set_texture_params(texture: &mut Self::Texture, repeat: bool, nearest: bool) {
        unsafe {
            let id = texture.id;
            if repeat {
                rlTextureParameters(id, RL_TEXTURE_WRAP_S as i32, RL_TEXTURE_WRAP_REPEAT as i32);
                rlTextureParameters(id, RL_TEXTURE_WRAP_T as i32, RL_TEXTURE_WRAP_REPEAT as i32);
            }
            if nearest {
                rlTextureParameters(id, RL_TEXTURE_MAG_FILTER as i32, RL_TEXTURE_FILTER_NEAREST as i32);
                rlTextureParameters(id, RL_TEXTURE_MIN_FILTER as i32, RL_TEXTURE_FILTER_NEAREST as i32);
            }
        }
    }

    fn create_render_target(&mut self, size: RendererVector2, hdr: bool) -> Self::RenderTarget {
        if hdr {
            create_rgba16_render_texture(size.x as i32, size.y as i32)
        } else {
            self.handle
                .load_render_texture(&self.thread, size.x as u32, size.y as u32)
                .unwrap()
        }
    }

    fn set_uniform_vec2(shader: &mut Self::Shader, name: &str, value: RendererVector2) {
        let location = shader.get_shader_location(name);
        println!("{} uniform location = {}", name, location);
        shader.set_shader_value_v(location, &[value]);
    }

    fn set_uniform_texture(shader: &mut Self::Shader, name: &str, _texture: &Self::Texture) {
        let location = shader.get_shader_location(name);
        println!("{} uniform location = {}", name, location);
        //shader.set_shader_value_texture(location, texture);
    }

    fn begin_texture(
        &mut self,
        size: RendererVector2,
        render_target: &mut Self::RenderTarget,
        texture: &Self::Texture,
    ) {
        let mut texture_mode = self.handle.begin_texture_mode(&self.thread, render_target);
        texture_mode.clear_background(Color::BLACK);
        let flipped_rectangle = Rectangle {
            x: 0.0,
            y: 0.0,
            width: size.x,
            height: -1.0 * size.y,
        };
        texture_mode.draw_texture_rec(texture, flipped_rectangle, ORIGIN, Color::WHITE);
    }

    fn begin_frame(&mut self, render_target: &Self::RenderTarget) -> bool {
        let mut draw_handle = self.handle.begin_drawing(&self.thread);
        draw_handle.draw_texture(render_target, ORIGIN_X, ORIGIN_Y, Color::WHITE);
        true
    }

    fn shader_draw(
        &mut self,
        size: RendererVector2,
        shader: &mut Self::Shader,
        render_target: &mut Self::RenderTarget,
    ) {
        let mut draw_handle = self.handle.begin_drawing(&self.thread);
        draw_handle.clear_background(Color::BLACK);
        let mut shader_mode = draw_handle.begin_shader_mode(shader);
        let flipped_rectangle = Rectangle {
            x: 0.0,
            y: 0.0,
            width: size.x,
            height: -1.0 * size.y,
        };
        shader_mode.draw_texture_rec(render_target, flipped_rectangle, ORIGIN, Color::WHITE);
    }
}
