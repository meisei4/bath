extends Node2D
class_name TestHarness

var collision_fragment_scene: PackedScene = preload(ResourcePaths.COLLISION_MASK_FRAGMENT)
var collision_mask_isp_scene: PackedScene = preload(ResourcePaths.RUSTY_COLLISION_MASK)
var ice_sheets_scene: PackedScene = preload(ResourcePaths.ICE_SHEETS_SCENE)
var perspective_tilt_fragment_scene: PackedScene = preload(
    ResourcePaths.PERSPECTIVE_TILT_MASK_FRAGMENT
)
var shadow_mask_scene: PackedScene = preload(ResourcePaths.SHADOW_MASK_SCENE)
var mechanics_scene: PackedScene = preload(ResourcePaths.MECHANICS_TEST)

@export var collision_mask_fragment: CollisionMaskFragment
@export var collision_mask_isp: CollisionMaskIncrementalScanlinePolygonizer
@export var ice_sheets: IceSheets
@export var perspective_tilt_mask_fragment: PerspectiveTiltMaskFragment
@export var shadow_mask: ShadowMask
@export var mechanics: Mechanics


func _ready() -> void:
    add_ice_sheets_scene()
    #add_collision_mask_isp_scene()
    add_collision_mask_fragment_scene()
    add_perspective_tilt_mask_fragment_scene()
    add_shadow_mask_scene()
    add_jump_mechanic_test_scene()
    #bake()


func add_collision_mask_fragment_scene() -> void:
    collision_mask_fragment = collision_fragment_scene.instantiate() as CollisionMaskFragment
    add_child(collision_mask_fragment)
    collision_mask_fragment.owner = self


func add_collision_mask_isp_scene() -> void:
    collision_mask_isp = (
        collision_mask_isp_scene.instantiate() as CollisionMaskIncrementalScanlinePolygonizer
    )
    add_child(collision_mask_isp)
    collision_mask_isp.owner = self


func add_ice_sheets_scene() -> void:
    ice_sheets = ice_sheets_scene.instantiate() as IceSheets
    add_child(ice_sheets)
    ice_sheets.owner = self


func add_perspective_tilt_mask_fragment_scene() -> void:
    perspective_tilt_mask_fragment = (
        perspective_tilt_fragment_scene.instantiate() as PerspectiveTiltMaskFragment
    )
    add_child(perspective_tilt_mask_fragment)
    perspective_tilt_mask_fragment.owner = self


func add_shadow_mask_scene() -> void:
    shadow_mask = shadow_mask_scene.instantiate() as ShadowMask
    add_child(shadow_mask)
    shadow_mask.owner = self


func add_jump_mechanic_test_scene() -> void:
    mechanics = mechanics_scene.instantiate() as Mechanics
    add_child(mechanics)
    mechanics.owner = self
