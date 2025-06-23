use godot::builtin::Vector2i;
use godot::classes::sub_viewport::{ClearMode, UpdateMode};
use godot::classes::SubViewport;
use godot::obj::{Gd, NewAlloc};

pub fn create_buffer_viewport(i_resolution: Vector2i) -> Gd<SubViewport> {
    let mut subviewport = SubViewport::new_alloc();
    subviewport.set_size(i_resolution);
    subviewport.set_disable_3d(true);
    subviewport.set_use_hdr_2d(true);
    subviewport.set_clear_mode(ClearMode::ONCE);
    subviewport.set_update_mode(UpdateMode::ALWAYS);
    subviewport
}
