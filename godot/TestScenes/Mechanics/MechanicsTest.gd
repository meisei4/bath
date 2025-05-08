extends Node2D
class_name MechanicsTest

var capsule_dummy: CapsuleDummy


func _ready() -> void:
    var capsule_scene: PackedScene = (
        ResourceLoader.load("res://godot/TestScenes/Entities/Characters/CapsuleDummy.tscn")
        as PackedScene
    )
    capsule_dummy = capsule_scene.instantiate() as CapsuleDummy
    capsule_dummy.z_index = 1
    add_child(capsule_dummy)

    var viewport_size: Vector2 = ResolutionManager.resolution
    var sprite_node: Sprite2D = capsule_dummy.get_node("Sprite2D") as Sprite2D
    var sprite_size: Vector2 = sprite_node.texture.get_size()
    capsule_dummy.position = Vector2(viewport_size.x * 0.5, viewport_size.y - sprite_size.y * 0.5)
    MechanicManager.register_controller(capsule_dummy)

    var music_resource: AudioStream = load(AudioConsts.HELLION_MP3)
    AudioManager.play_music(music_resource, 0.0)
