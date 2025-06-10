extends Node
class_name ShaderToyUtil


static func create_buffer_viewport(size: Vector2) -> SubViewport:
    var subviewport: SubViewport = SubViewport.new()
    subviewport.size = size
    subviewport.disable_3d = true
    subviewport.use_hdr_2d = true
    RenderingServer.set_default_clear_color(Color.BLACK)
    subviewport.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    subviewport.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    return subviewport
