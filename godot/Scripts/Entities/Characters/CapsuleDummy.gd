extends CharacterBody2D
class_name CapsuleDummy

var mechanics: Dictionary[Mechanic.TYPE, Mechanic]
var collision_shape: CollisionShape2D


func _ready() -> void:
    MechanicsManager.register_character_body(self)
    var lateral_movement: LateralMovement = LateralMovement.new()
    add_child(lateral_movement)
    mechanics.set(Mechanic.TYPE.LATERAL_MOVEMENT, lateral_movement)

    var jump: Jump = Jump.new()
    add_child(jump)
    mechanics.set(Mechanic.TYPE.JUMP, jump)

    var swim: Swim = Swim.new()
    add_child(swim)
    mechanics.set(Mechanic.TYPE.SWIM, swim)

    if !collision_shape:
        for child: Node in self.get_children():
            if child is CollisionShape2D:
                collision_shape = child
                break


func _physics_process(delta: float) -> void:
    for mechanic: Mechanic in mechanics.values():
        mechanic.process_input(delta)
        match mechanic.type:
            Mechanic.TYPE.JUMP:
                self.position.y = self.position.y - mechanic.forward_movement_pixel_units
                if mechanic._is_grounded():
                    self.collision_shape.disabled = false  #TODO: lmao double negatives
                else:
                    self.collision_shape.disabled = true
            Mechanic.TYPE.LATERAL_MOVEMENT:
                self.position.x += mechanic.delta_pixels

    move_and_slide()

#TODO: are you serious, learn wtf physics process actually does, it can cause sprite draws vs compute shaderdraws single frame lag...
#func _process(delta: float) -> void:
#for mechanic: Mechanic in mechanics:
#mechanic.process_input(delta)
#mechanic.process_visual_illusion(delta)
