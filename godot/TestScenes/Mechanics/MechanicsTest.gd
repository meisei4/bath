extends Node2D
class_name MechanicsTest

var tilt_mask: PerspectiveTiltMask


func _ready() -> void:
    if tilt_mask == null:
        tilt_mask = PerspectiveTiltMask.new()
        add_child(tilt_mask)
    var capsule_scene: PackedScene = (
        ResourceLoader.load("res://godot/TestScenes/Entities/Characters/CapsuleDummy.tscn")
        as PackedScene
    )
    var capsule_dummy: CapsuleDummy = capsule_scene.instantiate() as CapsuleDummy
    capsule_dummy.z_index = 1
    #TODO: HACKEDDDD
    capsule_dummy.tilt_mask = tilt_mask
    add_child(capsule_dummy)

    #var viewport_size: Vector2 = get_viewport().get_visible_rect().size
    var viewport_size: Vector2 = Resolution.resolution
    var sprite_node: Sprite2D = capsule_dummy.get_node("Sprite2D") as Sprite2D
    var sprite_size: Vector2 = sprite_node.texture.get_size()
    capsule_dummy.position = Vector2(viewport_size.x * 0.5, viewport_size.y - sprite_size.y * 0.5)
