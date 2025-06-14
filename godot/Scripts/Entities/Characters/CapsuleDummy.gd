extends CharacterBody2D
class_name CapsuleDummy

var collision_shape: CollisionShape2D

var mechanic_scenes: Array[PackedScene] = [
    preload(ResourcePaths.LATERAL_MOVEMENT_MECHANIC),
    preload(ResourcePaths.JUMP_MECHANIC),
    preload(ResourcePaths.SWIM_MECHANIC),
]

var mechanic_controller: MechanicController


func _ready() -> void:
    mechanic_controller = MechanicController.new()
    mechanic_controller.mechanic_scenes = mechanic_scenes
    add_child(mechanic_controller)
    for child_node: Node in get_children():
        if child_node is CollisionShape2D:
            collision_shape = child_node
            break

    if AnimationManager.perspective_tilt_mask_fragment:
        if !AnimationManager.character_body_to_mask_index.has(self):
            var sprite_node: Sprite2D = self.get_node("Sprite2D")
            var mask_index: int = (
                AnimationManager
                . perspective_tilt_mask_fragment
                . register_sprite_texture(sprite_node.texture)
            )
            AnimationManager.character_body_to_mask_index[self] = mask_index


func _physics_process(delta: float) -> void:
    move_and_slide()
