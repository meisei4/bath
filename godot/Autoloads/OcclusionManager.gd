extends Node


func scan_and_attach_occluders(root: Node) -> void:
    for child in root.get_children():
        _add_occluder_if_supported(child)
        scan_and_attach_occluders(child)


func _add_occluder_if_supported(node: Node) -> void:
    var rect: Rect2

    if node is Sprite2D and node.texture:
        var size = node.texture.get_size()
        rect = Rect2(node.get_global_position() - size / 2, size)
    else:
        return

    var light_occluder = LightOccluder2D.new()
    light_occluder.occluder_light_mask = 1

    var occluder = OccluderPolygon2D.new()
    occluder.polygon = PackedVector2Array(
        [
            rect.position,
            rect.position + Vector2(rect.size.x, 0),
            rect.position + rect.size,
            rect.position + Vector2(0, rect.size.y)
        ]
    )
    light_occluder.occluder = occluder

    node.get_parent().add_child(light_occluder)
