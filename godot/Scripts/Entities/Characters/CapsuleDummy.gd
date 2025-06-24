extends CharacterBody2D
class_name CapsuleDummy

@onready var collision_shape: CollisionShape2D = $CollisionShape2D
#var collision_shape: CollisionShape2D

@onready var sprite: Sprite2D = $Sprite2D
#var sprite: Sprite2D

var mut_ref_velocity: MutRefVelocity

var mechanic_controller: MechanicController

var collision_controller: CollisionController

func _ready() -> void:
    for child_node: Node in get_children():
        if child_node is CollisionShape2D:
            collision_shape = child_node
            break

    for child_node: Node in get_children():
        if child_node is Sprite2D:
            sprite = child_node
            break
    mut_ref_velocity = MutRefVelocity.new()
    mechanic_controller = MechanicController.new()
    mechanic_controller.mut_ref_velocity = self.mut_ref_velocity
    add_child(mechanic_controller)

    var animation_controller: AnimationController = AnimationController.new()
    animation_controller.sprite = self.sprite
    animation_controller.mechanics = mechanic_controller.get_children()
    add_child(animation_controller)

    collision_controller = CollisionController.new()
    collision_controller.mechanics = mechanic_controller.get_children()
    add_child(collision_controller)

    if MaskManager.perspective_tilt_mask_fragment:
        if !MaskManager.sprite_to_mask_index.has(self.sprite):
            var mask_index: int = (
                MaskManager.perspective_tilt_mask_fragment.register_sprite_texture(sprite.texture)
            )
            MaskManager.sprite_to_mask_index[self.sprite] = mask_index


func _physics_process(delta: float) -> void:
    self.velocity = mut_ref_velocity.val
    self.collision_shape.disabled = collision_controller.collision_shape_disabled()
    move_and_slide()
    # NOTE: this next line is neccessary to let all the Mechanics know the
    # state of the velocity after move_and_slide() phsyics
    mut_ref_velocity.val = self.velocity
