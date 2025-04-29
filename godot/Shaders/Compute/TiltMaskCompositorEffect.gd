extends CompositorEffect
class_name TiltMaskCompositorEffect

var tilt_mask: PerspectiveTiltMask


func _render_callback(_type: int, _rd: RenderData) -> void:
    tilt_mask._dispatch_compute()
