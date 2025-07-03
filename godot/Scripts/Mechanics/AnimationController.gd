extends Node
class_name AnimationController

@export var sprite: Sprite2D
@export var animation_scenes: Array[PackedScene] = [
    preload(ResourcePaths.JUMP_ANIMATION),
    preload(ResourcePaths.DIVE_ANIMATION),
    preload(ResourcePaths.SPIN_ANIMATION),
]

@export var mechanics: Array[Node]
@export var jump: Jump
@export var dive: Dive
@export var spin: Spin


func _ready() -> void:
    for mechanic: Node in mechanics:
        if mechanic is Jump:
            jump = mechanic
        if mechanic is Dive:
            dive = mechanic
        if mechanic is Spin:
            spin = mechanic

    for animation_scene: PackedScene in animation_scenes:
        var animation: Node = animation_scene.instantiate()
        animation.sprite = sprite
        if animation is JumpAnimation:
            jump.animate_mechanic.connect(animation.process_animation_data)
        if animation is DiveAnimation:
            dive.animate_mechanic.connect(animation.process_animation_data)
        if animation is SpinAnimation:
            spin.animate_mechanic.connect(animation.process_animation_data)

        add_child(animation)
