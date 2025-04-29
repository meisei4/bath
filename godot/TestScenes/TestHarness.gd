extends Node2D
class_name TestHarness


var collision_mask: CollisionMask
var glacier_flow: GlacierFlow
var tilt_mask: PerspectiveTiltMask
var shadows_test: ShadowsTest
var mechanics_test: MechanicsTest


func _ready() -> void:
    #TODO: ORDER MATTERS
    add_collision_mask_compute_shader_scene()
    add_glacier_flow_test_scene()

    add_perspective_tilt_mask_compute_shader_scene()
    add_shadows_test_scene()
    add_jump_mechanic_test_scene()
    add_snow_particles_test_scene()


func add_perspective_tilt_mask_compute_shader_scene() -> void:
    var tilt_scene: PackedScene = preload("res://godot/TestScenes/Shaders/Compute/PerspectiveTiltMask.tscn")
    tilt_mask = tilt_scene.instantiate() as PerspectiveTiltMask
    add_child(tilt_mask)


func add_collision_mask_compute_shader_scene() -> void:
    var collision_scene: PackedScene = preload("res://godot/TestScenes/Shaders/Compute/CollisionMask.tscn")
    collision_mask = collision_scene.instantiate() as CollisionMask
    add_child(collision_mask)


func add_shadows_test_scene() -> void:
    var shadows_scene: PackedScene = preload("res://godot/TestScenes/Shaders/Shadows/ShadowsTest.tscn")
    shadows_test = shadows_scene.instantiate() as ShadowsTest
    shadows_test.tilt_mask = tilt_mask
    add_child(shadows_test)


func add_jump_mechanic_test_scene() -> void:
    var mechanics_scene: PackedScene = preload("res://godot/TestScenes/Mechanics/MechanicsTest.tscn")
    mechanics_test = mechanics_scene.instantiate() as MechanicsTest
    mechanics_test.tilt_mask = tilt_mask  #TODO: HACKED
    add_child(mechanics_test)


func add_glacier_flow_test_scene() -> void:
    var glacier_scene: PackedScene = preload("res://godot/TestScenes/Shaders/Glacier/GlacierFlow.tscn")
    glacier_flow = glacier_scene.instantiate() as GlacierFlow
    glacier_flow.collision_mask = collision_mask #TODO: HACKED
    add_child(glacier_flow)


#TODO: this is old as heck particles, i can write it so much better now that i know shaders more, this actually sucks,
# putting it in for fun though
func add_snow_particles_test_scene() -> void:
    var snow_scene: PackedScene = preload("res://godot/TestScenes/Shaders/Particles/SnowfallParticles.tscn")
    var snowfall: SnowfallParticles = snow_scene.instantiate() as SnowfallParticles
    add_child(snowfall)
