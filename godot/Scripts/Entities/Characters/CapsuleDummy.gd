extends CharacterBody2D
class_name CapsuleDummy

@onready var collision_shape: CollisionShape2D = $CollisionShape2D
#var collision_shape: CollisionShape2D

@onready var sprite: Sprite2D = $Sprite2D
#var sprite: Sprite2D


func _ready() -> void:
    var mechanic_controller: MechanicController = MechanicController.new()
    mechanic_controller.controller_host = self
    add_child(mechanic_controller)

    for child_node: Node in get_children():
        if child_node is CollisionShape2D:
            collision_shape = child_node
            break

    for child_node: Node in get_children():
        if child_node is Sprite2D:
            sprite = child_node
            break

    if AnimationManager.perspective_tilt_mask_fragment:
        if !AnimationManager.character_body_to_mask_index.has(self):
            var mask_index: int = (
                AnimationManager
                . perspective_tilt_mask_fragment
                . register_sprite_texture(sprite.texture)
            )
            AnimationManager.character_body_to_mask_index[self] = mask_index


func _physics_process(delta: float) -> void:
    move_and_slide()
