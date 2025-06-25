use crate::render::raylib_util::{
    create_rgba16_render_texture, flip_framebuffer, load_shader_with_includes, APPLE_DPI, ORIGIN, ORIGIN_X, ORIGIN_Y,
};
use crate::render::renderer::{Renderer, RendererVector2};
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibShaderModeExt, RaylibTextureModeExt};
use raylib::ffi::{
    rlTextureParameters, LoadImage, LoadTextureFromImage, UnloadImage, RL_TEXTURE_FILTER_NEAREST,
    RL_TEXTURE_MAG_FILTER, RL_TEXTURE_MIN_FILTER, RL_TEXTURE_WRAP_REPEAT, RL_TEXTURE_WRAP_S, RL_TEXTURE_WRAP_T,
};
use raylib::shaders::{RaylibShader, Shader};
use raylib::texture::{RaylibTexture2D, RenderTexture2D, Texture2D};
use raylib::{init, RaylibHandle, RaylibThread};
use std::ffi::{c_char, CString};

pub struct RaylibRenderer {
    pub handle: RaylibHandle,
    thread: RaylibThread,
}

impl Renderer for RaylibRenderer {
    type RenderTarget = RenderTexture2D;
    type Texture = Texture2D;
    type Shader = Shader;

    fn init(width: i32, height: i32) -> Self {
        let (mut handle, thread) = init()
            .size(width / APPLE_DPI, height / APPLE_DPI)
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
        Self { handle, thread }
    }
    fn init_render_target(&mut self, size: RendererVector2, hdr: bool) -> Self::RenderTarget {
        if hdr {
            create_rgba16_render_texture(size.x as i32, size.y as i32)
        } else {
            self.handle
                .load_render_texture(&self.thread, size.x as u32, size.y as u32)
                .unwrap()
        }
    }

    fn load_texture(&mut self, path: &str) -> Self::Texture {
        //TODO: I really dont like this, there has got to be a more effective way
        let path_in_c = CString::new(path).unwrap();
        let image_texture = unsafe {
            let image_raw = LoadImage(path_in_c.as_ptr() as *const c_char);
            let image_texture_raw = LoadTextureFromImage(image_raw);
            UnloadImage(image_raw);
            Texture2D::from_raw(image_texture_raw)
        };
        image_texture
        // TODO: watch it
        // self.handle.load_texture(&self.thread, path).unwrap()
    }

    fn tweak_texture_parameters(&mut self, texture: &mut Self::Texture, repeat: bool, nearest: bool) {
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

    fn load_shader(&mut self, vert_path: &str, frag_path: &str) -> Self::Shader {
        let vert_source_code = load_shader_with_includes(vert_path);
        let frag_source_code = load_shader_with_includes(frag_path);
        self.handle
            .load_shader_from_memory(&self.thread, Some(&vert_source_code), Some(&frag_source_code))
    }

    fn set_uniform_float(&mut self, shader: &mut Self::Shader, name: &str, value: f32) {
        let location = shader.get_shader_location(name);
        println!("{} uniform location = {}", name, location);
        shader.set_shader_value(location, value);
    }

    fn set_uniform_vec2(&mut self, shader: &mut Self::Shader, name: &str, value: RendererVector2) {
        let location = shader.get_shader_location(name);
        println!("{} uniform location = {}", name, location);
        shader.set_shader_value_v(location, &[value]);
    }

    fn set_uniform_mat2(&mut self, shader: &mut Self::Shader, name: &str, mat2: &[RendererVector2]) {
        let location = shader.get_shader_location(name);
        println!("{} uniform location = {}", name, location);
        //TODO: figure out how to make a Matrix in raylib
        //shader.set_shader_value_matrix(location, mat2);
    }

    fn set_uniform_sampler2d(&mut self, shader: &mut Self::Shader, name: &str, _texture: &Self::Texture) {
        let location = shader.get_shader_location(name);
        println!("{} uniform location = {}", name, location);
        //shader.set_shader_value_texture(location, texture);
    }

    fn draw_texture(&mut self, texture: &mut Self::Texture, render_target: &mut Self::RenderTarget) {
        let width = render_target.width() as f32;
        let height = render_target.height() as f32;
        let mut texture_mode = self.handle.begin_texture_mode(&self.thread, render_target);
        texture_mode.clear_background(Color::BLACK);
        texture_mode.draw_texture_rec(texture, flip_framebuffer(width, height), ORIGIN, Color::WHITE);
    }

    fn draw_screen(&mut self, render_target: &Self::RenderTarget) {
        let mut draw_handle = self.handle.begin_drawing(&self.thread);
        draw_handle.draw_texture(render_target, ORIGIN_X, ORIGIN_Y, Color::WHITE);
    }

    fn draw_shader_screen(&mut self, shader: &mut Self::Shader, render_target: &mut Self::RenderTarget) {
        let mut draw_handle = self.handle.begin_drawing(&self.thread);
        draw_handle.clear_background(Color::BLACK);
        let mut shader_mode = draw_handle.begin_shader_mode(shader);
        let width = render_target.width() as f32;
        let height = render_target.height() as f32;
        shader_mode.draw_texture_rec(render_target, flip_framebuffer(width, height), ORIGIN, Color::WHITE);
    }
}
