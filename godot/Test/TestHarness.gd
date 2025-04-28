extends Node2D
class_name TestHarness

var glacier_flow: GlacierFlow
var collision_mask: CollisionMask
var tilt_mask: PerspectiveTiltMask
var shadows_test: ShadowsTest
var mechanics_test: MechanicsTest


func _ready() -> void:
    add_glacier_flow_test_scene()
    add_collision_mask_compute_shader_scene()

    #TODO: perspective tilt needs to be instantiated before shadows, bad design dependency, fix it later
    add_perspective_tilt_mask_compute_shader_scene()
    #below all depend on perspective tilt mask..
    add_shadows_test_scene()
    add_jump_mechanic_test_scene()
    add_snow_particles_test_scene()


func _process(delta: float) -> void:
    collision_mask.iTime += delta
    #also update the fragment shader material…
    glacier_flow.BufferAShaderMaterial.set_shader_parameter("iTime", collision_mask.iTime)


func add_perspective_tilt_mask_compute_shader_scene() -> void:
    var tilt_scene: PackedScene = preload("res://godot/Shaders/Compute/PerspectiveTiltMask.tscn")
    tilt_mask = tilt_scene.instantiate() as PerspectiveTiltMask
    add_child(tilt_mask)


func add_collision_mask_compute_shader_scene() -> void:
    var collision_scene: PackedScene = preload("res://godot/Shaders/Compute/CollisionMask.tscn")
    collision_mask = collision_scene.instantiate() as CollisionMask
    add_child(collision_mask)


func add_shadows_test_scene() -> void:
    var shadows_scene: PackedScene = preload("res://godot/Test/Shaders/Shadows/ShadowsTest.tscn")
    shadows_test = shadows_scene.instantiate() as ShadowsTest
    #TODO: this is silly because now it adds order dependency in the node tree, but ill figure it out later
    add_child(shadows_test)
    shadows_test.UmbralShaderMaterial.set_shader_parameter(
        "iChannel1", tilt_mask.perspective_tilt_mask_texture
    )


func add_jump_mechanic_test_scene() -> void:
    var mechanics_scene: PackedScene = preload("res://godot/Test/Mechanics/MechanicsTest.tscn")
    mechanics_test = mechanics_scene.instantiate() as MechanicsTest
    mechanics_test.tilt_mask = tilt_mask  #TODO: HACKED ew ew ew
    add_child(mechanics_test)


func add_glacier_flow_test_scene() -> void:
    var glacier_scene: PackedScene = preload("res://godot/Test/Shaders/Glacier/GlacierFlow.tscn")
    glacier_flow = glacier_scene.instantiate() as GlacierFlow
    add_child(glacier_flow)


#TODO: this is old as heck particles, i can write it so much better now that i know shaders more, this actually sucks,
# putting it in for fun though
func add_snow_particles_test_scene() -> void:
    var snow_scene: PackedScene = preload("res://godot/Shaders/Particles/SnowfallParticles.tscn")
    var snowfall: SnowfallParticles = snow_scene.instantiate() as SnowfallParticles
    add_child(snowfall)
