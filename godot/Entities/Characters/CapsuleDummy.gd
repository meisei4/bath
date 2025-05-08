extends CharacterBody2D
class_name CapsuleDummy

#var mechanics: Array[Mechanic] = []
var mechanics: Dictionary[Mechanic.TYPE, Mechanic]


func _ready() -> void:
    ComputeShaderSignalManager.register_character_body(self)
    var lateral_movement: LateralMovement = LateralMovement.new()
    lateral_movement.character_body = self
    add_child(lateral_movement)
    mechanics.set(Mechanic.TYPE.LATERAL_MOVEMENT, lateral_movement)

    var jump: Jump = Jump.new()
    jump.character_body = self
    add_child(jump)
    mechanics.set(Mechanic.TYPE.JUMP, jump)

    var swim: Swim = Swim.new()
    swim.character_body = self
    add_child(swim)
    mechanics.set(Mechanic.TYPE.SWIM, swim)


func _physics_process(_delta: float) -> void:
    move_and_slide()

#TODO: are you serious, learn wtf physics process actually does, it can cause sprite draws vs compute shaderdraws single frame lag...
#func _process(delta: float) -> void:
#for mechanic: Mechanic in mechanics:
#mechanic.process_input(delta)
#mechanic.process_visual_illusion(delta)
#mechanic.process_collision_shape(delta)
