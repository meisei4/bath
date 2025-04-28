extends CompositorEffect
class_name TiltMaskCompositorEffect


func _render_callback(_type: int, _rd: RenderData) -> void:
    PerspectiveTiltMask._dispatch_compute()
