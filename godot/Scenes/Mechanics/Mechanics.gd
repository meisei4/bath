extends Node2D
class_name Mechanics

#TODO: this is where things are just insanity, im trying to fight Object Oriented so much...
# and i think im close, just needs something to click

var capsule_dummy: CapsuleDummy
var capsule_scene: PackedScene = preload(ResourcePaths.CAPSULE_DUMMY)


func _ready() -> void:
    add_perspective_tilt_mask_fragment_scene()
    capsule_dummy = capsule_scene.instantiate() as CapsuleDummy
    capsule_dummy.z_index = 1
    add_child(capsule_dummy)
    var viewport_size: Vector2 = ResolutionManager.resolution
    var sprite_node: Sprite2D = capsule_dummy.get_node("Sprite2D") as Sprite2D
    var sprite_size: Vector2 = sprite_node.texture.get_size()
    capsule_dummy.position = Vector2(viewport_size.x * 0.5, viewport_size.y - sprite_size.y * 0.5)
    #MechanicManager.register_controller(capsule_dummy)


#TODO: please remove the dependancy
func add_perspective_tilt_mask_fragment_scene() -> void:
    var perspective_tilt_fragment_scene: PackedScene = preload(
        ResourcePaths.PERSPECTIVE_TILT_MASK_FRAGMENT
    )
    var perspective_tilt_mask_fragment: PerspectiveTiltMaskFragment = (
        perspective_tilt_fragment_scene.instantiate() as PerspectiveTiltMaskFragment
    )
    add_child(perspective_tilt_mask_fragment)
