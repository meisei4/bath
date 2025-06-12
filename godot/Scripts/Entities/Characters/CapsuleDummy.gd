extends CharacterBody2D
class_name CapsuleDummy

var collision_shape: CollisionShape2D

@export var mechanic_scenes: Array[PackedScene] = [
    preload(ResourcePaths.LATERAL_MOVEMENT_MECHANIC),
    preload(ResourcePaths.JUMP_MECHANIC),
    preload(ResourcePaths.SWIM_MECHANIC),
]

var mechanic_controller: MechanicController


func _ready() -> void:
    mechanic_controller = MechanicController.new()
    mechanic_controller.mechanic_scenes = mechanic_scenes
    add_child(mechanic_controller)
    add_child(AnimationController.new())
    for child_node: Node in get_children():
        if child_node is CollisionShape2D:
            collision_shape = child_node
            break


func _physics_process(delta: float) -> void:
    mechanic_controller.handle_input()
    for mechanic: Mechanic in mechanic_controller.mechanics:
        mechanic.update_position_delta_pixels(delta)
        mechanic.update_collision(collision_shape)
        self.position += mechanic.delta_pixels
        mechanic.emit_mechanic_data(delta)

    move_and_slide()
