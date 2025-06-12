extends Node
class_name AnimationController

var ANIMATIONS: Dictionary[Mechanic.TYPE, Shader] = {
    Mechanic.TYPE.JUMP: preload(ResourcePaths.JUMP_TRIG_SHADER),
    Mechanic.TYPE.SWIM: preload(ResourcePaths.SWIM_SHADER),
}

var controller_host: CharacterBody2D
var mechanic_controller: MechanicController
var sprite_node: Sprite2D
var _current_mechanic_type: Mechanic.TYPE


func _ready() -> void:
    controller_host = get_parent() as CapsuleDummy
    sprite_node = controller_host.get_node("Sprite2D") as Sprite2D
    mechanic_controller = controller_host.mechanic_controller
    mechanic_controller.state_changed.connect(_on_state_changed)
    _connect_mechanic_animation_signals(mechanic_controller.mechanics)
    _on_state_changed(mechanic_controller.current_state)


func _connect_mechanic_animation_signals(mechanics: Array[Mechanic]) -> void:
    if AnimationManager.perspective_tilt_mask_fragment:
        if !AnimationManager.character_body_to_mask_index.has(controller_host):
            var sprite_texture: Texture2D = sprite_node.texture
            var mask_index: int = (
                AnimationManager
                . perspective_tilt_mask_fragment
                . register_sprite_texture(sprite_texture)
            )
            AnimationManager.character_body_to_mask_index[controller_host] = mask_index

    for mechanic: Mechanic in mechanics:
        match mechanic.type:
            Mechanic.TYPE.JUMP:
                var jump_animation: JumpAnimation = JumpAnimation.new()
                var jump_mechanic: Jump = mechanic as Jump
                jump_mechanic.animate_jump.connect(
                    jump_animation.process_animation.bind(controller_host)
                )
            Mechanic.TYPE.SWIM:
                var swim_animation: SwimAnimation = SwimAnimation.new()
                var swim_mechanic: Swim = mechanic as Swim
                swim_mechanic.animate_swim.connect(
                    swim_animation.process_animation.bind(controller_host)
                )


func update_animation(next_mechanic: Mechanic) -> void:
    var shader: Shader = ANIMATIONS[next_mechanic.type]
    if !shader:
        return
    if sprite_node:
        if sprite_node.material:
            sprite_node.material.shader = shader
        else:
            sprite_node.material = ShaderMaterial.new()


func _on_state_changed(state: MechanicManager.STATE) -> void:
    for mechanic: Mechanic in mechanic_controller.mechanics:
        if mechanic.handles_state(state):
            update_animation(mechanic)
            break
