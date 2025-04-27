extends CharacterBody2D
class_name CapsuleDummy

var mechanics: Array[Mechanic] = []


func _ready() -> void:
    #TODO: HACKED?
    var sprite_node: Sprite2D = get_node("Sprite2D") as Sprite2D
    var sprite_texture_index: int = PerspectiveTiltMask.register_sprite_texture(sprite_node.texture)
    #TODO: ^^HACKED?

    var lateral_movement: LateralMovement = LateralMovement.new()
    lateral_movement.character = self
    add_child(lateral_movement)
    mechanics.append(lateral_movement)

    var jump: Jump = Jump.new()
    jump.character = self
    #TODO: hacked???... but the only way to make sure the texture was added above
    jump.sprite_texture_index = sprite_texture_index
    add_child(jump)
    mechanics.append(jump)


#TODO: are you serious, learn wtf physics process actually does, it can cause sprite draws vs compute shaderdraws single frame lag...
func _process(delta: float) -> void:
    for mechanic: Mechanic in mechanics:
        mechanic.process_input(delta)
        mechanic.process_visual_illusion(delta)
        mechanic.process_collision_shape(delta)
