use crate::render::raylib_util::{
    create_rgba16_render_texture, load_shader_with_includes, APPLE_DPI, WINDOW_HEIGHT, WINDOW_WIDTH,
};
use crate::render::renderer::Renderer;
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibShaderModeExt, RaylibTextureModeExt};
use raylib::ffi::{
    rlTextureParameters, RL_TEXTURE_FILTER_NEAREST, RL_TEXTURE_MAG_FILTER, RL_TEXTURE_MIN_FILTER,
    RL_TEXTURE_WRAP_REPEAT, RL_TEXTURE_WRAP_S, RL_TEXTURE_WRAP_T,
};
use raylib::math::{Rectangle, Vector2};
use raylib::shaders::{RaylibShader, Shader};
use raylib::texture::{RaylibTexture2D, RenderTexture2D, Texture2D};
use raylib::{init, RaylibHandle, RaylibThread};

pub struct RaylibRenderer {
    handle: RaylibHandle,
    thread: RaylibThread,
}

impl Renderer for RaylibRenderer {
    type Shader = Shader;
    type Texture = Texture2D;
    type RenderTarget = RenderTexture2D;

    fn init() -> Self {
        let (mut handle, thread) = init()
            .size(WINDOW_WIDTH / APPLE_DPI, WINDOW_HEIGHT / APPLE_DPI)
            .title("raylib renderer")
            .build();
        handle.set_target_fps(60);
        Self {
            handle,
            thread,
        }
    }

    fn load_shader(&mut self, path: &str) -> Self::Shader {
        let src = load_shader_with_includes(path);
        self.handle.load_shader_from_memory(&self.thread, None, Some(&src))
    }

    fn load_texture(&mut self, path: &str) -> Self::Texture {
        self.handle
            .load_texture(&self.thread, path)
            .expect("texture load failed")
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

    fn create_render_target(&mut self, size: Vector2, hdr: bool) -> Self::RenderTarget {
        if hdr {
            create_rgba16_render_texture(size.x as i32, size.y as i32)
        } else {
            self.handle
                .load_render_texture(&self.thread, size.x as u32, size.y as u32)
                .expect("render-target alloc failed")
        }
    }

    fn target_texture_for_render(render_target: &Self::RenderTarget) -> &Self::Texture {
        todo!()
    }

    fn set_uniform_vec2(shader: &mut Self::Shader, name: &str, v: Vector2) {
        let uniform_location = shader.get_shader_location(name);
        shader.set_shader_value_v(uniform_location, &[v]);
    }

    fn set_uniform_texture(shader: &mut Self::Shader, name: &str, texture: &Self::Texture) {
        let uniform_location = shader.get_shader_location(name);

        shader.set_shader_value_texture(uniform_location, texture);
    }

    fn begin_render(&mut self, rt: &mut Self::RenderTarget) {
        let _ = self.handle.begin_texture_mode(&self.thread, rt);
    }

    fn end_render(&mut self) {}

    fn begin_frame(&mut self) -> bool {
        if self.handle.window_should_close() {
            return false;
        }
        true
    }

    fn draw(&mut self, shader: &mut Self::Shader, texture: &Self::Texture) {
        let rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: texture.width() as f32,
            height: -(texture.height() as f32),
        };
        let mut draw_handle = self.handle.begin_drawing(&self.thread);
        let mut shader_mode = draw_handle.begin_shader_mode(shader);
        shader_mode.draw_texture_rec(texture, rect, Vector2::zero(), Color::WHITE);
    }

    fn end_frame(&mut self) {}
}
