use crate::render::raylib_util::{
    create_rgba16_render_texture, feedback_buffer_pass, flip_framebuffer, image_pass, APPLE_DPI, ORIGIN, ORIGIN_X,
    ORIGIN_Y,
};
use crate::render::renderer::{
    FeedbackBufferContext, Renderer, RendererMatrix, RendererVector2, RendererVector3, RendererVector4,
};
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibShaderModeExt, RaylibTextureModeExt};
use raylib::ffi::TextureFilter::TEXTURE_FILTER_POINT;
use raylib::ffi::TextureWrap::TEXTURE_WRAP_REPEAT;
use raylib::ffi::{
    rlActiveTextureSlot, rlEnableTexture, LoadImage, LoadImageFromMemory, LoadTextureFromImage, SetTextureFilter,
    SetTextureWrap, UnloadImage, RL_QUADS, RL_TEXTURE,
};
use raylib::math::{Matrix, Vector3};
use raylib::shaders::{RaylibShader, Shader};
use raylib::texture::{RaylibTexture2D, RenderTexture2D, Texture2D};
use raylib::{ffi, init, RaylibHandle, RaylibThread};
use std::f32::consts::FRAC_PI_2;
use std::ffi::{c_char, CString};
use std::sync::Once;

pub struct RaylibRenderer {
    pub handle: RaylibHandle,
    pub thread: RaylibThread,
    pub sampler2d_count: i32,
}

static LOG_ITIME_LOCATION: Once = Once::new();

impl Renderer for RaylibRenderer {
    type RenderTarget = RenderTexture2D;
    type Texture = Texture2D;
    type Shader = Shader;

    fn init(width: i32, height: i32) -> Self {
        let (mut handle, thread) = init()
            .size(width / APPLE_DPI, height / APPLE_DPI)
            .title("raylib renderer")
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
            sampler2d_count: 1_i32,
        }
    }

    fn init_i_resolution(&mut self) -> RendererVector2 {
        todo!()
    }

    fn update_mask(&mut self, _i_time: f32) {
        todo!()
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

    fn load_texture_file_path(&mut self, path: &str) -> Self::Texture {
        let image_texture = unsafe {
            let path_in_c = CString::new(format!("{}", path)).unwrap();
            let image_raw = LoadImage(path_in_c.as_ptr() as *const c_char);
            let image_texture_raw = LoadTextureFromImage(image_raw);
            UnloadImage(image_raw);
            Texture2D::from_raw(image_texture_raw)
        };
        image_texture
        // TODO: watch it
        // self.handle.load_texture(&self.thread, path).unwrap()
    }

    fn load_texture(&mut self, data: &[u8], file_ext: &str) -> Self::Texture {
        let image_texture = unsafe {
            let c_ext = CString::new(format!(".{file_ext}")).unwrap();
            let image_raw = LoadImageFromMemory(c_ext.as_ptr() as *const c_char, data.as_ptr(), data.len() as i32);
            let image_texture_raw = LoadTextureFromImage(image_raw);
            UnloadImage(image_raw);
            Texture2D::from_raw(image_texture_raw)
        };
        image_texture
    }

    fn tweak_texture_parameters(&mut self, texture: &mut Self::Texture, repeat: bool, nearest: bool) {
        unsafe {
            // let id = texture.id;
            if repeat {
                SetTextureWrap(**texture, TEXTURE_WRAP_REPEAT as i32);
                // rlTextureParameters(id, RL_TEXTURE_WRAP_S as i32, RL_TEXTURE_WRAP_REPEAT as i32);
                // rlTextureParameters(id, RL_TEXTURE_WRAP_T as i32, RL_TEXTURE_WRAP_REPEAT as i32);
            }
            if nearest {
                SetTextureFilter(**texture, TEXTURE_FILTER_POINT as i32);
                // rlTextureParameters(id, RL_TEXTURE_MAG_FILTER as i32, RL_TEXTURE_FILTER_NEAREST as i32);
                // rlTextureParameters(id, RL_TEXTURE_MIN_FILTER as i32, RL_TEXTURE_FILTER_NEAREST as i32);
            }
        }
    }

    fn load_shader_fragment(&mut self, frag_src: &str) -> Self::Shader {
        let frag_src_include_expansion = asset_payload::expand_includes(frag_src);
        self.handle
            .load_shader_from_memory(&self.thread, None, Some(frag_src_include_expansion))
    }

    fn load_shader_vertex(&mut self, vert_src: &str) -> Self::Shader {
        let vert_src_include_expansion = asset_payload::expand_includes(vert_src);
        self.handle
            .load_shader_from_memory(&self.thread, Some(vert_src_include_expansion), None)
    }

    fn load_shader_full(&mut self, vert_src: &str, frag_src: &str) -> Self::Shader {
        let vert_src_include_expansion = asset_payload::expand_includes(vert_src);
        let frag_src_include_expansion = asset_payload::expand_includes(frag_src);
        self.handle.load_shader_from_memory(
            &self.thread,
            Some(vert_src_include_expansion),
            Some(frag_src_include_expansion),
        )
    }

    fn set_uniform_float(&mut self, shader: &mut Self::Shader, uniform_name: &str, value: f32) {
        let location = shader.get_shader_location(uniform_name);
        match uniform_name {
            "iTime" => {
                LOG_ITIME_LOCATION.call_once(|| {
                    println!("{} uniform location = {}", uniform_name, location);
                });
            },
            _ => {
                println!("{} uniform location = {}", uniform_name, location);
            },
        }
        shader.set_shader_value(location, value);
    }
    fn set_uniform_int(&mut self, shader: &mut Self::Shader, uniform_name: &str, value: i32) {
        let location = shader.get_shader_location(uniform_name);
        println!("{} uniform location = {}", uniform_name, location);
        shader.set_shader_value(location, value);
    }

    fn set_uniform_vec2(&mut self, shader: &mut Self::Shader, uniform_name: &str, value: RendererVector2) {
        let location = shader.get_shader_location(uniform_name);
        println!("{} uniform location = {}", uniform_name, location);
        shader.set_shader_value_v(location, &[value]);
    }

    fn set_uniform_vec3(&mut self, _shader: &mut Self::Shader, _uniform_name: &str, _vec3: RendererVector3) {
        todo!()
    }

    fn set_uniform_vec4(&mut self, _shader: &mut Self::Shader, _uniform_name: &str, _vec4: RendererVector4) {
        todo!()
    }

    fn set_uniform_vec3_array(
        &mut self,
        shader: &mut Self::Shader,
        uniform_name: &str,
        vec3_array: &[RendererVector3],
    ) {
        let location = shader.get_shader_location(uniform_name);
        println!("{} uniform location = {}", uniform_name, location);
        shader.set_shader_value_v(location, vec3_array);
    }

    fn set_uniform_mat2(&mut self, _shader: &mut Self::Shader, _uniform_name: &str, _mat2: RendererMatrix) {
        todo!()
    }

    fn set_uniform_mat4(&mut self, shader: &mut Self::Shader, uniform_name: &str, mat4: RendererMatrix) {
        let location = shader.get_shader_location(uniform_name);
        println!("{} uniform location = {}", uniform_name, location);
        shader.set_shader_value_matrix(location, mat4);
    }

    fn set_uniform_sampler2d(&mut self, shader: &mut Self::Shader, uniform_name: &str, texture: &Self::Texture) {
        let location = shader.get_shader_location(uniform_name);
        println!("{} uniform location = {}", uniform_name, location);
        //TODO: JUST MOVE TOI UNSTABLE BRANCH ALREADY AHHHHH!!!
        unsafe {
            rlActiveTextureSlot(self.sampler2d_count);
            rlEnableTexture(texture.id);
        }
        shader.set_shader_value(location, self.sampler2d_count);
        self.sampler2d_count += 1_i32;
        unsafe {
            rlActiveTextureSlot(0); //TODO: THIS IS VERY IMPORTANT
        }
    }

    fn draw_texture(&mut self, texture: &mut Self::Texture, render_target: &mut Self::RenderTarget) {
        let width = render_target.width() as f32;
        let height = render_target.height() as f32;
        let mut texture_mode = self.handle.begin_texture_mode(&self.thread, render_target);
        texture_mode.clear_background(Color::BLACK);
        texture_mode.draw_texture_rec(texture, flip_framebuffer(width, height), ORIGIN, Color::WHITE);
    }

    fn draw_shader_texture(&mut self, shader: &mut Self::Shader, render_target: &mut Self::RenderTarget) {
        let width = render_target.width() as f32;
        let height = render_target.height() as f32;
        let rect = flip_framebuffer(width, height);
        //NOTE 1: issue with draw_texture_rec -> Trait `AsRef<Texture>` is not implemented for `Texture2D`
        // let render_target_texture: ffi::Texture2D = render_target.texture;
        //NOTE 2: issue with begin_texture_mode -> cannot borrow `*render_target` as mutable because already immutably borrowed
        //let render_target_texture: &WeakTexture2D = render_target.texture();
        //NOTE 3 this one works, but it involves function parens and a clone to get a WeakTexture
        //let render_target_texture: WeakTexture2D  = render_target.texture().clone();
        let mut texture_mode = self.handle.begin_texture_mode(&self.thread, render_target);
        texture_mode.clear_background(Color::BLACK);
        let mut shader_mode = texture_mode.begin_shader_mode(shader);
        shader_mode.draw_rectangle(
            rect.x as i32,
            rect.y as i32,
            rect.width as i32,
            rect.height as i32,
            Color::WHITE,
        );
    }

    fn draw_screen(&mut self, render_target: &Self::RenderTarget) {
        let mut draw_handle = self.handle.begin_drawing(&self.thread);
        draw_handle.draw_texture(render_target, ORIGIN_X, ORIGIN_Y, Color::WHITE);
    }

    fn draw_shader_screen(&mut self, shader: &mut Self::Shader, render_target: &mut Self::RenderTarget) {
        let width = self.handle.get_screen_width() as f32;
        let height = self.handle.get_screen_height() as f32;
        let mut draw_handle = self.handle.begin_drawing(&self.thread);
        draw_handle.clear_background(Color::BLACK);
        let mut shader_mode = draw_handle.begin_shader_mode(shader);
        // let width = render_target.width() as f32;
        // let height = render_target.height() as f32;
        shader_mode.draw_texture_rec(render_target, flip_framebuffer(width, height), ORIGIN, Color::WHITE);
    }

    fn draw_shader_screen_pseudo_ortho_geom(
        &mut self,
        shader: &mut Self::Shader,
        render_target: &mut Self::RenderTarget,
    ) {
        let mut draw_handle = self.handle.begin_drawing(&self.thread);
        draw_handle.clear_background(Color::BLACK);
        let width = render_target.width() as f32;
        let height = render_target.height() as f32;
        let observer_pos = Vector3::new(width / 2.0, height / 2.0, height / 2.0);
        let target = Vector3::new(width / 2.0, height / 2.0, 0.0);
        let up = Vector3::new(0.0, 1.0, 0.0);
        let projection = Matrix::perspective(FRAC_PI_2, width / height, 0.01, 1000.0);
        let view = Matrix::look_at(observer_pos, target, up);
        let _shader = draw_handle.begin_shader_mode(shader);
        unsafe {
            ffi::rlSetMatrixModelview(view.into());
            ffi::rlSetMatrixProjection(projection.into());
            //PERFECT
            ffi::rlTexCoord2f(0.0, 1.0);
            ffi::rlVertex3f(0.0, height, 0.0);
            ffi::rlTexCoord2f(0.0, 0.0);
            ffi::rlVertex3f(0.0, 0.0, 0.0);
            ffi::rlTexCoord2f(1.0, 0.0);
            ffi::rlVertex3f(width, 0.0, 0.0);
            ffi::rlTexCoord2f(1.0, 1.0);
            ffi::rlVertex3f(width, height, 0.0);

            // VERTICES ARE THE WRONG ORDER? (BATHYCENTRIC GRADIENT TEST FAILS)
            // ffi::rlTexCoord2f(0.0, 0.0);
            // ffi::rlVertex3f(0.0, 0.0, 0.0);
            // ffi::rlTexCoord2f(1.0, 0.0);
            // ffi::rlVertex3f(width, 0.0, 0.0);
            // ffi::rlTexCoord2f(1.0, 1.0);
            // ffi::rlVertex3f(width, height, 0.0);
            // ffi::rlTexCoord2f(0.0, 1.0);
            // ffi::rlVertex3f(0.0, height, 0.0);

            //BLACK FACE CULLING??? WHAT??
            // ffi::rlTexCoord2f(0.0, 1.0);
            // ffi::rlVertex3f(0.0, height, 0.0);
            // ffi::rlTexCoord2f(1.0, 1.0);
            // ffi::rlVertex3f(width, height, 0.0);
            // ffi::rlTexCoord2f(1.0, 0.0);
            // ffi::rlVertex3f(width, 0.0, 0.0);
            // ffi::rlTexCoord2f(0.0, 0.0);
            // ffi::rlVertex3f(0.0, 0.0, 0.0);
        }
    }

    fn draw_shader_screen_tilted_geom(
        &mut self,
        shader: &mut Self::Shader,
        render_target: &mut Self::RenderTarget,
        mut tilt_deg: f32,
    ) {
        let mut draw_handle = self.handle.begin_drawing(&self.thread);
        draw_handle.clear_background(Color::BLACK);
        let width = render_target.width() as f32;
        let height = render_target.height() as f32;
        let observer_pos = Vector3::new(width / 2.0, height / 2.0, height / 2.0);
        let target = Vector3::new(width / 2.0, height / 2.0, 0.0);
        let up = Vector3::new(0.0, 1.0, 0.0);
        let projection = Matrix::perspective(FRAC_PI_2, width / height, 0.01, 1000.0);
        let view = Matrix::look_at(observer_pos, target, up);
        let _shader = draw_handle.begin_shader_mode(shader);
        unsafe {
            ffi::rlSetMatrixModelview(view.into());
            ffi::rlSetMatrixProjection(projection.into());
            tilt_deg = tilt_deg.clamp(0.0, 89.0);

            // TODO: this is for allowing letter boxing
            let pivot_y = height / 2.0;
            ffi::rlPushMatrix();
            ffi::rlTranslatef(0.0, pivot_y, 0.0);
            ffi::rlRotatef(-tilt_deg, 1.0, 0.0, 0.0);
            ffi::rlTranslatef(0.0, -pivot_y, 0.0);

            // TODO: this is for scaling to JUST (minimally) prevent any letterboxing
            // let theta = tilt_deg.to_radians();
            // let scale = (1.0 + theta.sin()) / theta.cos();
            // let center_x = width / 2.0;
            // let center_y = height / 2.0;
            // ffi::rlPushMatrix();
            // ffi::rlTranslatef(center_x, center_y, 0.0);
            // ffi::rlScalef(scale, scale, 1.0);
            // ffi::rlTranslatef(-center_x, -center_y, 0.0);
            // ffi::rlTranslatef(0.0, center_y, 0.0);
            // ffi::rlRotatef(-tilt_deg, 1.0, 0.0, 0.0);
            // ffi::rlTranslatef(0.0, -center_y, 0.0);

            ffi::rlTexCoord2f(0.0, 1.0);
            ffi::rlVertex3f(0.0, height, 0.0);
            ffi::rlTexCoord2f(0.0, 0.0);
            ffi::rlVertex3f(0.0, 0.0, 0.0);
            ffi::rlTexCoord2f(1.0, 0.0);
            ffi::rlVertex3f(width, 0.0, 0.0);
            ffi::rlTexCoord2f(1.0, 1.0);
            ffi::rlVertex3f(width, height, 0.0);
            ffi::rlPopMatrix();
        }
    }

    fn draw_fixedfunc_screen_pseudo_ortho_geom(&mut self, texture: &Self::Texture) {
        let width = self.handle.get_screen_width() as f32;
        let height = self.handle.get_screen_height() as f32;

        let mut draw_handle = self.handle.begin_drawing(&self.thread);
        draw_handle.clear_background(Color::BLACK);
        unsafe {
            ffi::rlActiveTextureSlot(0);
            ffi::rlEnableTexture(texture.id);
            ffi::rlMatrixMode(RL_TEXTURE as i32);
            ffi::rlLoadIdentity();
            ffi::rlTranslatef(-0.5, -0.5, 0.0);
            ffi::rlScalef(4.0, 4.0, 1.0);

            ffi::rlBegin(RL_QUADS as i32);
            ffi::rlTexCoord2f(0.0, 1.0);
            ffi::rlVertex3f(0.0, height, 0.0);
            ffi::rlTexCoord2f(0.0, 0.0);
            ffi::rlVertex3f(0.0, 0.0, 0.0);
            ffi::rlTexCoord2f(1.0, 0.0);
            ffi::rlVertex3f(width, 0.0, 0.0);
            ffi::rlTexCoord2f(1.0, 1.0);
            ffi::rlVertex3f(width, height, 0.0);
            ffi::rlEnd();
        }
    }

    // TODO: figure out how to make this smoother:
    //  1.potentially just pass the MVP as a uniform and bypass all of raylibs stuff?
    //  1.5. learn how raylibs batch stuff works for the graphics pipeline
    //  2. figure out how custom MVP works on different versions of openGL
    //  3. figure out if there is any performance issue here with having to reset mvp every frame
    //  3.5 i dont htink we should have to do it every frame because the MVP never changes.
    //https://github.com/raylib-rs/raylib-rs/blob/unstable/showcase/src/example/others/rlgl_standalone.rs#L4
    //https://github.com/raysan5/raylib/blob/master/examples/others/rlgl_standalone.c
    // fn ambigious_template(&mut self, shader: &mut Self::Shader, render_target: &mut Self::RenderTarget) {
    //     let mut draw_handle = self.handle.begin_drawing(&self.thread);
    //     draw_handle.clear_background(Color::BLACK);
    //     let width  = render_target.width()  as f32;
    //     let height = render_target.height() as f32;
    //     let observer_pos = Vector3::new( , , );
    //     let target       = Vector3::new( , , );
    //     let up           = Vector3::new( , , );
    //     let projection = Matrix::perspective( , , , );
    //     let view = Matrix::look_at(observer_pos, target, up);
    //     //_shader_mode.draw_texture_rec(render_target, flip_framebuffer(width, height), ORIGIN, Color::WHITE);
    //     // TODO: NOPE
    //     // let mvp = projection * view;
    //     // let mvp_location = shader.get_shader_location("mvp");
    //     // println!("mvp uniform location = {}", mvp_location);
    //     // shader.set_shader_value_matrix(mvp_location, mvp);
    //     // TODO: you really need to learn how rust works, let _ = here will break everything
    //     let _shader = draw_handle.begin_shader_mode(shader);
    //     unsafe {
    //         ffi::rlSetMatrixModelview(view.into());
    //         ffi::rlSetMatrixProjection(projection.into());
    //         ffi::rlTexCoord2f( , );
    //         ffi::rlVertex3f( , , );
    //         ffi::rlTexCoord2f( , );
    //         ffi::rlVertex3f( , , );
    //         ffi::rlTexCoord2f( , );
    //         ffi::rlVertex3f( , , );
    //         ffi::rlTexCoord2f( , );
    //         ffi::rlVertex3f( , , );
    //      // TODO: no need to deallocate thats the whole point of the raylib-rs safety stuff??
    //      //  https://github.com/raylib-rs/raylib-rs/blob/unstable/raylib/src/core/drawing.rs#L326
    //     }
    // }

    fn init_feedback_buffer(
        &mut self,
        resolution: RendererVector2,
        feedback_pass_shader_path: &str,
        main_pass_shader_path: &str,
    ) -> FeedbackBufferContext<Self> {
        let buffer_a = create_rgba16_render_texture(resolution.x as i32, resolution.y as i32);
        let buffer_b = create_rgba16_render_texture(resolution.x as i32, resolution.y as i32);

        let feedback_pass_shader = self.load_shader_fragment(feedback_pass_shader_path);
        let main_pass_shader = self.load_shader_fragment(main_pass_shader_path);

        FeedbackBufferContext {
            buffer_a,
            buffer_b,
            feedback_pass_shader,
            main_pass_shader,
        }
    }

    fn render_feedback_pass(&mut self, context: &mut FeedbackBufferContext<Self>) {
        feedback_buffer_pass(
            &mut self.handle,
            &self.thread,
            &mut context.feedback_pass_shader,
            &mut context.buffer_b,
            &context.buffer_a,
        );

        context.swap();

        image_pass(
            &mut self.handle,
            &self.thread,
            &mut context.main_pass_shader,
            &context.buffer_a,
        );
    }
}
