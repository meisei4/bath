extends Node2D
class_name TestHarness


func _ready() -> void:
    add_collision_mask_isp_scene()
    add_ice_sheets_scene()
    add_collision_mask_fragment_scene()
    add_perspective_tilt_mask_fragment_scene()
    add_shadows_test_scene()
    add_jump_mechanic_test_scene()


func add_collision_mask_fragment_scene() -> void:
    var collision_fragment_scene: PackedScene = preload(ResourcePaths.COLLISION_MASK_FRAGMENT)
    var collision_mask_fragment: CollisionMaskFragment = (
        collision_fragment_scene.instantiate() as CollisionMaskFragment
    )
    add_child(collision_mask_fragment)


func add_collision_mask_isp_scene() -> void:
    var collision_mask_isp_scene: PackedScene = preload(ResourcePaths.RUSTY_COLLISION_MASK)
    var collision_mask_isp: CollisionMaskIncrementalScanlinePolygonizer = (
        collision_mask_isp_scene.instantiate() as CollisionMaskIncrementalScanlinePolygonizer
    )
    add_child(collision_mask_isp)


func add_ice_sheets_scene() -> void:
    var ice_sheets_scene: PackedScene = preload(ResourcePaths.ICE_SHEETS_SCENE)
    var ice_sheets: IceSheets = ice_sheets_scene.instantiate() as IceSheets
    add_child(ice_sheets)


func add_perspective_tilt_mask_fragment_scene() -> void:
    var perspective_tilt_fragment_scene: PackedScene = preload(
        ResourcePaths.PERSPECTIVE_TILT_MASK_FRAGMENT
    )
    var perspective_tilt_mask_fragment: PerspectiveTiltMaskFragment = (
        perspective_tilt_fragment_scene.instantiate() as PerspectiveTiltMaskFragment
    )
    add_child(perspective_tilt_mask_fragment)


func add_shadows_test_scene() -> void:
    var shadows_scene: PackedScene = preload(ResourcePaths.SHADOWS_SCENE)
    var shadows_test: Shadows = shadows_scene.instantiate() as Shadows
    add_child(shadows_test)


func add_jump_mechanic_test_scene() -> void:
    var mechanics_scene: PackedScene = preload(ResourcePaths.MECHANICS_TEST)
    var mechanics_test: Mechanics = mechanics_scene.instantiate() as Mechanics
    add_child(mechanics_test)
