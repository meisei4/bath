extends Node2D
class_name CollisionMaskIncrementalScanlinePolygonizer

const MAX_POLYGONS: int = 24

var isp_texture: ISPTexture
var iResolution: Vector2

var collision_mask_bodies: Array[StaticBody2D]
var collision_mask_concave_polygons_pool: Array[CollisionShape2D]
var collision_polygons: Array[PackedVector2Array]
var projected_polygons: Array[PackedVector2Array]
var scanline_count_per_polygon: PackedInt32Array

var previous_quantized_vertical_pixel_coord: int = 0


func _ready() -> void:
    iResolution = ResolutionManager.resolution
    _init_concave_collision_polygon_pool()
    _init_polygon_state_arrays()
    CollisionMaskTargetsManager.ice_sheets_entered_scene.connect(_on_ice_sheets_entered)
    if CollisionMaskTargetsManager.ice_sheets:
        _on_ice_sheets_entered(CollisionMaskTargetsManager.ice_sheets)


func _on_ice_sheets_entered(ice_sheets: IceSheets) -> void:
    if isp_texture:
        return

    isp_texture = ISPTexture.new()
    isp_texture.TargetFrameBuffer = ice_sheets.BufferA
    add_child(isp_texture)
    #TODO: in order to allow for the ISPTexture to have its own every frame update cycle
    # so that we dont need to call
    # isp_texture.update_scanline_alpha_bucket_bit_masks() everytime here, figure out event order with this thing
    RenderingServer.frame_post_draw.connect(_on_frame_post_draw)


func _init_polygon_state_arrays() -> void:
    scanline_count_per_polygon.resize(MAX_POLYGONS)
    collision_polygons.resize(MAX_POLYGONS)


func _init_concave_collision_polygon_pool() -> void:
    for i: int in range(MAX_POLYGONS):
        var static_body: StaticBody2D = StaticBody2D.new()
        add_child(static_body)
        collision_mask_bodies.append(static_body)
        var shape_node: CollisionShape2D = CollisionShape2D.new()
        shape_node.disabled = true
        var concave: ConcavePolygonShape2D = ConcavePolygonShape2D.new()
        shape_node.shape = concave
        static_body.add_child(shape_node)
        collision_mask_concave_polygons_pool.append(shape_node)


func _on_frame_post_draw() -> void:
    var iTime: float = CollisionMaskTargetsManager.iTime
    isp_texture.update_scanline_alpha_bucket_bit_masks()
    var scanline_alpha_buckets_top_row: PackedVector2Array
    scanline_alpha_buckets_top_row = isp_texture.fill_scanline_alpha_buckets_top_row()
    var result: Dictionary = (
        RustUtilSingleton
        . rust_util
        . process_scanline(
            iTime,
            iResolution,
            collision_polygons,
            scanline_alpha_buckets_top_row,
            previous_quantized_vertical_pixel_coord,
            scanline_count_per_polygon,
        )
    )
    previous_quantized_vertical_pixel_coord = result["previous_quantized_vertical_pixel_coord"]
    scanline_count_per_polygon = result["scanline_count_per_polygon"]
    collision_polygons = result["collision_polygons"]
    projected_polygons = result["projected_polygons"]
    _update_collision_polygons()


func _update_collision_polygons() -> void:
    for i: int in range(MAX_POLYGONS):
        var segments: PackedVector2Array = projected_polygons[i]
        var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
        var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
        concave.segments = segments
        shape_node.disabled = segments.is_empty()
