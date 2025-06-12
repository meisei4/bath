extends Node
#class_name AnimationManager

var character_body_to_mask_index: Dictionary[CharacterBody2D, int]

var ANIMATIONS: Dictionary[Mechanic.TYPE, Shader] = {
    Mechanic.TYPE.JUMP: preload(ResourcePaths.JUMP_TRIG_SHADER),
    Mechanic.TYPE.SWIM: preload(ResourcePaths.SWIM_SHADER),
}

var umbral_shadow: ShaderMaterial
var perspective_tilt_mask_fragment: PerspectiveTiltMaskFragment


func _ready() -> void:
    MechanicManager.character_body_registered.connect(_on_character_body_registered)
    MechanicManager.state_changed.connect(_on_state_changed)
    set_process(false)


func register_perspective_tilt_mask_fragment(
    _perspective_tilt_mask_fragment: PerspectiveTiltMaskFragment
) -> void:
    perspective_tilt_mask_fragment = _perspective_tilt_mask_fragment


func update_perspective_tilt_mask(
    sprite_texture: Texture2D,
    character_body: CharacterBody2D,
    sprite_position: Vector2,
    sprite_scale: Vector2,
    altitude_normal: float,
    ascending: bool
) -> void:
    perspective_tilt_mask_fragment.set_sprite_data(
        sprite_texture,
        character_body_to_mask_index[character_body],
        sprite_position,
        sprite_scale,
        altitude_normal,
        1.0 if ascending else 0.0
    )


func register_umbral_shadow(_umbral_shadow: ShaderMaterial) -> void:
    self.umbral_shadow = _umbral_shadow
    if perspective_tilt_mask_fragment:
        umbral_shadow.set_shader_parameter(
            "iChannel1", perspective_tilt_mask_fragment.get_perspective_tilt_mask_texture_fragment()
        )


func _on_character_body_registered(_character_body: CharacterBody2D) -> void:
    if !perspective_tilt_mask_fragment:
        return
    if character_body_to_mask_index.has(_character_body):
        print("youre trying to add a body that already exists?")
        return

    var sprite_node: Sprite2D = _character_body.get_node("Sprite2D") as Sprite2D
    var sprite_texture: Texture2D = sprite_node.texture
    var mask_index: int = perspective_tilt_mask_fragment.register_sprite_texture(sprite_texture)
    character_body_to_mask_index[_character_body] = mask_index
    for mechanic: Mechanic in _character_body.mechanics:
        match mechanic.type:
            Mechanic.TYPE.JUMP:
                var jump_mechanic: Jump = mechanic as Jump
                var jump_animation: JumpAnimation = JumpAnimation.new()
                jump_mechanic.animate_jump.connect(
                    jump_animation.process_animation.bind(_character_body)
                )
                break
            Mechanic.TYPE.SWIM:
                var swim_mechanic: Swim = mechanic as Swim
                var swim_animation: SwimAnimation = SwimAnimation.new()
                swim_mechanic.animate_swim.connect(
                    swim_animation.process_animation.bind(_character_body)
                )
                break


func update_animation(character_body: CharacterBody2D, next_mechanic: Mechanic) -> void:
    var shader: Shader = ANIMATIONS[next_mechanic.type]
    var sprite: Sprite2D = character_body.get_node("Sprite2D") as Sprite2D
    if sprite:
        if sprite.material:
            sprite.material.shader = shader
        else:
            sprite.material = ShaderMaterial.new()


func _on_state_changed(state: MechanicManager.STATE) -> void:
    var character_body: CharacterBody2D = MechanicManager.controller.character
    for mechanic: Mechanic in character_body.mechanics:
        if mechanic.handles_state(state):
            update_animation(character_body, mechanic)
            break
