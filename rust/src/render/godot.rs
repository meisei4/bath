use crate::render::godot_util::create_buffer_viewport;
use crate::render::renderer::{FeedbackBufferContext, Renderer, RendererMatrix, RendererVector2};
use godot::builtin::{real, Vector2, Vector2i};
use godot::classes::canvas_item::{TextureFilter, TextureRepeat};
use godot::classes::texture_rect::StretchMode;
use godot::classes::{ColorRect, Node, ResourceLoader, Texture2D, TextureRect};
use godot::classes::{Shader, ShaderMaterial, SubViewport};
use godot::meta::ToGodot;
use godot::obj::{Base, Gd, NewAlloc, NewGd, WithBaseField};
use godot::prelude::GodotClass;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct GodotRenderer {
    base: Base<Node>,
}

const EMPTY: &str = "";

impl Renderer for GodotRenderer {
    type RenderTarget = Gd<SubViewport>;
    type Texture = Gd<Texture2D>;
    type Shader = Gd<ShaderMaterial>;

    fn init(_width: i32, _height: i32) -> Self {
        unreachable!("Godot instantiates this node; Renderer::init() is never called")
    }

    fn init_i_resolution(&mut self) -> RendererVector2 {
        let scene_tree = self.base().get_tree().unwrap();
        let root_window = scene_tree.get_root().unwrap();
        let resolution_manager = root_window.get_node_as::<Node>("ResolutionManager");
        let godot_resolution = resolution_manager.get("resolution").try_to::<Vector2>().unwrap();
        let i_resolution = RendererVector2::new(godot_resolution.x, godot_resolution.y);
        i_resolution
    }

    fn update_mask(&mut self, i_time: f32) {
        let scene_tree = self.base().get_tree().unwrap();
        let root_window = scene_tree.get_root().unwrap();
        let mut mask_manager = root_window.get_node_as::<Node>("MaskManager");
        mask_manager.set("iTime", &i_time.to_variant());
    }

    fn init_render_target(&mut self, size: RendererVector2, hdr: bool) -> Self::RenderTarget {
        let mut subviewport = create_buffer_viewport(Vector2i::new(size.x as i32, size.y as i32));
        subviewport.set_use_hdr_2d(hdr);
        subviewport
    }

    fn load_texture_file_path(&mut self, path: &str) -> Self::Texture {
        ResourceLoader::singleton().load(path).unwrap().cast()
    }

    fn load_texture(&mut self, _data: &[u8], _file_ext: &str) -> Self::Texture {
        todo!()
    }

    fn tweak_texture_parameters(&mut self, _texture: &mut Self::Texture, _repeat: bool, _nearest: bool) {
        todo!()
        //TODO: Texture for GodotRenderer we some how need to get the CanvasItem
        // I think we have to use a TextureRect with its filter and repeat somehow
        // Otherwise use viewport but somehow make the rendertarget filter and repeat (but that breaks from how raylib does it)
    }

    fn load_shader_fragment(&mut self, frag_path: &str) -> Self::Shader {
        self.load_shader_full(EMPTY, frag_path)
    }

    fn load_shader_vertex(&mut self, _vert_path: &str) -> Self::Shader {
        todo!()
    }

    //TODO: GODOT IS STILL ALWAYS FILE PATHS!!!!!!!!!! FIX IT AT SOME POINT!!!!
    fn load_shader_full(&mut self, _vert_path: &str, frag_path: &str) -> Self::Shader {
        let shader = ResourceLoader::singleton().load(frag_path).unwrap().cast::<Shader>();
        let mut shader_material = ShaderMaterial::new_gd();
        shader_material.set_shader(&shader);
        shader_material
    }

    fn set_uniform_float(&mut self, shader: &mut Self::Shader, uniform_name: &str, value: f32) {
        shader.set_shader_parameter(uniform_name, &value.to_variant());
    }

    fn set_uniform_int(&mut self, shader: &mut Self::Shader, uniform_name: &str, value: i32) {
        shader.set_shader_parameter(uniform_name, &value.to_variant());
    }

    fn set_uniform_vec2(&mut self, shader: &mut Self::Shader, uniform_name: &str, vec2: RendererVector2) {
        shader.set_shader_parameter(uniform_name, &Vector2::new(vec2.x, vec2.y).to_variant());
    }

    fn set_uniform_mat2(&mut self, _shader: &mut Self::Shader, _uniform_name: &str, _mat2: RendererMatrix) {
        todo!()
    }

    fn set_uniform_mat4(&mut self, _shader: &mut Self::Shader, _uniform_name: &str, _mat4: RendererMatrix) {
        todo!()
    }

    fn set_uniform_sampler2d(&mut self, shader: &mut Self::Shader, uniform_name: &str, texture: &Self::Texture) {
        shader.set_shader_parameter(uniform_name, &texture.to_variant());
    }

    fn draw_texture(&mut self, texture: &mut Self::Texture, render_target: &mut Self::RenderTarget) {
        let mut buffer_a_node = TextureRect::new_alloc();
        buffer_a_node.set_stretch_mode(StretchMode::TILE);
        buffer_a_node.set_texture_filter(TextureFilter::NEAREST);
        buffer_a_node.set_texture_repeat(TextureRepeat::ENABLED);
        buffer_a_node.set_texture(&*texture);
        let i_resolution = Vector2::new(render_target.get_size().x as real, render_target.get_size().y as real);
        buffer_a_node.set_size(i_resolution);
        render_target.add_child(&buffer_a_node);
        self.base_mut().add_child(&*render_target);
    }

    fn draw_shader_texture(&mut self, shader: &mut Self::Shader, render_target: &mut Self::RenderTarget) {
        let mut buffer_a_shader_node = ColorRect::new_alloc();
        let i_resolution = Vector2::new(render_target.get_size().x as real, render_target.get_size().y as real);
        buffer_a_shader_node.set_size(i_resolution);
        buffer_a_shader_node.set_texture_filter(TextureFilter::NEAREST);
        buffer_a_shader_node.set_material(&*shader);
        render_target.add_child(&buffer_a_shader_node);
        self.base_mut().add_child(&*render_target);
    }

    fn draw_screen(&mut self, render_target: &Self::RenderTarget) {
        let mut main_image = TextureRect::new_alloc();
        main_image.set_texture_filter(TextureFilter::NEAREST);
        main_image.set_texture(&render_target.get_texture().unwrap());
        main_image.set_flip_v(true);
        self.base_mut().add_child(&main_image);
    }

    fn draw_shader_screen(&mut self, shader: &mut Self::Shader, render_target: &mut Self::RenderTarget) {
        let mut buffer_a_shader_node = ColorRect::new_alloc();
        let i_resolution = Vector2::new(render_target.get_size().x as real, render_target.get_size().y as real);
        buffer_a_shader_node.set_size(i_resolution);
        buffer_a_shader_node.set_texture_filter(TextureFilter::NEAREST);
        buffer_a_shader_node.set_material(&*shader);
        render_target.add_child(&buffer_a_shader_node);
        self.draw_screen(render_target);
    }

    fn draw_shader_screen_pseudo_ortho_geom(
        &mut self,
        _shader: &mut Self::Shader,
        _render_target: &mut Self::RenderTarget,
    ) {
        todo!()
    }

    fn draw_shader_screen_tilted_geom(
        &mut self,
        _shader: &mut Self::Shader,
        _render_target: &mut Self::RenderTarget,
        _tilt: f32,
    ) {
        todo!()
    }

    fn init_feedback_buffer(
        &mut self,
        resolution: RendererVector2,
        feedback_pass_shader_path: &str,
        main_pass_shader_path: &str,
    ) -> FeedbackBufferContext<Self> {
        let mut buffer_a = self.init_render_target(resolution, false);
        let mut buffer_b = self.init_render_target(resolution, false);

        // TODO: if you are going to convert feedback shaders HERE is where you need to update the
        //  uniform sampler2D iChannel0 : hint_screen_texture;
        //  hint tagging in godot
        let mut feedback_pass_shader = self.load_shader_fragment(feedback_pass_shader_path);
        self.set_uniform_vec2(&mut feedback_pass_shader, "iResolution", resolution);

        let mut main_pass_shader = self.load_shader_fragment(main_pass_shader_path);
        self.set_uniform_vec2(&mut main_pass_shader, "iResolution", resolution);

        let mut buffer_a_rect = ColorRect::new_alloc();
        buffer_a_rect.set_size(Vector2::new(resolution.x, resolution.y));
        buffer_a_rect.set_material(&feedback_pass_shader);
        buffer_a.add_child(&buffer_a_rect);
        self.base_mut().add_child(&buffer_a);

        let mut buffer_b_rect = ColorRect::new_alloc();
        buffer_b_rect.set_size(Vector2::new(resolution.x, resolution.y));
        buffer_b_rect.set_material(&feedback_pass_shader);
        buffer_b.add_child(&buffer_b_rect);
        self.base_mut().add_child(&buffer_b);
        FeedbackBufferContext {
            buffer_a,
            buffer_b,
            feedback_pass_shader,
            main_pass_shader,
        }
    }

    fn render_feedback_pass(&mut self, context: &mut FeedbackBufferContext<Self>) {
        let texture = context.buffer_a.get_texture().unwrap().upcast::<Texture2D>();
        self.set_uniform_sampler2d(&mut context.feedback_pass_shader, "iChannel0", &texture);
        let mut screen = TextureRect::new_alloc();
        screen.set_texture(&context.buffer_b.get_texture().unwrap());
        screen.set_flip_v(true);
        self.base_mut().add_child(&screen);
        context.swap();
    }
}
