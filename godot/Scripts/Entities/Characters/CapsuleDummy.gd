extends CharacterBody2D
class_name CapsuleDummy

var mechanics: Array[Mechanic]

var collision_shape: CollisionShape2D


func _ready() -> void:
    var lateral_movement: LateralMovement = LateralMovement.new()
    add_child(lateral_movement)

    var jump: Jump = Jump.new()
    add_child(jump)

    var swim: Swim = Swim.new()
    add_child(swim)

    if !collision_shape:
        for child: Node in self.get_children():
            if child is CollisionShape2D:
                collision_shape = child
                break
    if !mechanics:
        for child: Node in self.get_children():
            if child is Mechanic:
                mechanics.append(child)

    MechanicManager.register_character_body(self)


func _physics_process(delta: float) -> void:
    for mechanic: Mechanic in mechanics:
        mechanic.update_position_delta_pixels(delta)
        mechanic.update_collision(collision_shape)
        self.position += mechanic.delta_pixels

    move_and_slide()

#TODO: are you serious, learn wtf physics process actually does, it can cause sprite draws vs compute shaderdraws single frame lag...
#func _process(delta: float) -> void:
#for mechanic: Mechanic in mechanics:
#mechanic.process_input(delta)
#mechanic.process_visual_illusion(delta)
