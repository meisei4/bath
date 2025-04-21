extends CharacterBody2D
class_name CapsuleDummy

var mechanics: Array[Mechanic] = []


func _ready() -> void:
    var lateral_movement: LateralMovement = LateralMovement.new()
    lateral_movement.character = self
    var later_movement_animation: ShaderMaterial = ShaderMaterial.new()
    #later_movement_animation.shader = preload("res://shaders/jump_bulge.shader")
    lateral_movement.animation_shader = later_movement_animation
    add_child(lateral_movement)
    mechanics.append(lateral_movement)

    var jump: Jump = Jump.new()
    jump.character = self
    add_child(jump) #TODO: this is when the _ready function gets ran and the sahder gets added
    mechanics.append(jump)


func _physics_process(delta: float) -> void:
    for mechanic: Mechanic in mechanics:
        mechanic.process_input(delta)
        mechanic.process_visual_illusion(delta)
        mechanic.process_collision_shape(delta)
