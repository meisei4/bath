extends CompositorEffect
class_name TiltMaskCompositorEffect


func _render_callback(_type: int, _rd: RenderData) -> void:
    SpriteAnimations._dispatch_compute()
