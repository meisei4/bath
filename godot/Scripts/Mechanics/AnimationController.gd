extends Node
class_name AnimationController

var sprite: Sprite2D
var animation_scenes: Array[PackedScene] = [
    preload(ResourcePaths.JUMP_ANIMATION),
    preload(ResourcePaths.DIVE_ANIMATION),
    preload(ResourcePaths.FLIP_ANIMATION),
]

var mechanics: Array[Node]
var jump: Jump
var dive: Dive
var flip: Flip


func _ready() -> void:
    for mechanic: Node in mechanics:
        if mechanic is Jump:
            jump = mechanic
        if mechanic is Dive:
            dive = mechanic
        if mechanic is Flip:
            flip = mechanic

    for animation_scene: PackedScene in animation_scenes:
        var animation: Node = animation_scene.instantiate()
        animation.sprite = sprite
        if animation is JumpAnimation:
            jump.animate_mechanic.connect(animation.process_animation_data)
        if animation is DiveAnimation:
            dive.animate_mechanic.connect(animation.process_animation_data)
        if animation is FlipAnimation:
            flip.animate_mechanic.connect(animation.process_animation_data)

        add_child(animation)
