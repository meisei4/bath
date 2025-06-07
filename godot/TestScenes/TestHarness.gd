extends Node2D
class_name TestHarness


func _ready() -> void:
    add_ice_sheets_scene()
    #add_collision_mask_scene()
    #add_collision_mask_fragment_scene()
    #add_collision_mask_isp_scene()
    add_collision_mask_rusty_scene()
    #add_perspective_tilt_mask_scene()
    add_perspective_tilt_mask_fragment_scene()
    add_shadows_test_scene()
    add_jump_mechanic_test_scene()


func add_collision_mask_scene() -> void:
    var collision_scene: PackedScene = preload(
        "res://TestScenes/Shaders/Compute/CollisionMask.tscn"
    )
    var collision_mask: CollisionMask = collision_scene.instantiate() as CollisionMask
    add_child(collision_mask)


func add_collision_mask_fragment_scene() -> void:
    var collision_fragment_scene: PackedScene = preload(
        "res://TestScenes/Shaders/Collision/CollisionMaskFragment.tscn"
    )
    var collision_mask_fragment: CollisionMaskFragment = (
        collision_fragment_scene.instantiate() as CollisionMaskFragment
    )
    add_child(collision_mask_fragment)


func add_collision_mask_isp_scene() -> void:
    var collision_mask_isp_scene: PackedScene = preload(
        "res://TestScenes/Shaders/Collision/CollisionMaskScanlinePolygonizer.tscn"
    )
    var collision_mask_isp: CollisionMaskScanlinePolygonizer = (
        collision_mask_isp_scene.instantiate() as CollisionMaskScanlinePolygonizer
    )
    add_child(collision_mask_isp)


func add_collision_mask_rusty_scene() -> void:
    var collision_mask_rusty_scene: PackedScene = preload(
        "res://TestScenes/Shaders/Collision/RustyCollisionMask.tscn"
    )
    var collision_mask_rusty: RustyCollisionMask = (
        collision_mask_rusty_scene.instantiate() as RustyCollisionMask
    )
    add_child(collision_mask_rusty)


func add_ice_sheets_scene() -> void:
    var ice_sheets_scene: PackedScene = preload("res://TestScenes/Shaders/IceSheets/IceSheets.tscn")
    var ice_sheets: IceSheets = ice_sheets_scene.instantiate() as IceSheets
    add_child(ice_sheets)


func add_perspective_tilt_mask_scene() -> void:
    var perspective_tilt_scene: PackedScene = preload(
        "res://TestScenes/Shaders/Compute/PerspectiveTiltMask.tscn"
    )
    var perspective_tilt_mask: PerspectiveTiltMask = (
        perspective_tilt_scene.instantiate() as PerspectiveTiltMask
    )
    add_child(perspective_tilt_mask)


func add_perspective_tilt_mask_fragment_scene() -> void:
    var perspective_tilt_fragment_scene: PackedScene = preload(
        "res://TestScenes/Shaders/MechanicAnimations/PerspectiveTiltMaskFragment.tscn"
    )
    var perspective_tilt_mask_fragment: PerspectiveTiltMaskFragment = (
        perspective_tilt_fragment_scene.instantiate() as PerspectiveTiltMaskFragment
    )
    add_child(perspective_tilt_mask_fragment)


func add_shadows_test_scene() -> void:
    var shadows_scene: PackedScene = preload("res://TestScenes/Shaders/Shadows/Shadows.tscn")
    var shadows_test: Shadows = shadows_scene.instantiate() as Shadows
    add_child(shadows_test)


func add_jump_mechanic_test_scene() -> void:
    var mechanics_scene: PackedScene = preload("res://TestScenes/Mechanics/MechanicsTest.tscn")
    var mechanics_test: MechanicsTest = mechanics_scene.instantiate() as MechanicsTest
    add_child(mechanics_test)
