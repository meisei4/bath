extends Node2D
class_name CollisionMaskIncrementalScanlinePolygonizer

const MAX_POLYGONS: int = 8

var isp_texture: ISPTexture
var iResolution: Vector2

var collision_mask_bodies: Array[StaticBody2D]
var collision_mask_concave_polygons_pool: Array[CollisionShape2D]
var collision_polygons: Array[PackedVector2Array]
var scanline_count_per_polygon: PackedInt32Array

var previous_scroll_accum: float = 0
var prev_iTime: float
var speed_px_per_sec: float
var total_rows_scrolled: int


func _ready() -> void:
    var prev_iTime = MaskManager.iTime
    iResolution = ResolutionManager.resolution
    _init_concave_collision_polygon_pool()
    _init_polygon_state_arrays()
    MaskManager.ice_sheets_entered_scene.connect(_on_ice_sheets_entered)
    if MaskManager.ice_sheets:
        _on_ice_sheets_entered(MaskManager.ice_sheets)


func _on_ice_sheets_entered(ice_sheets: IceSheetsRenderer) -> void:
    if isp_texture:
        return

    isp_texture = ISPTexture.new()
    isp_texture.TargetFrameBuffer = ice_sheets.BufferA
    add_child(isp_texture)
    isp_texture.owner = self
    RenderingServer.frame_post_draw.connect(_on_frame_post_draw)


func _init_polygon_state_arrays() -> void:
    scanline_count_per_polygon.resize(MAX_POLYGONS)
    collision_polygons.resize(MAX_POLYGONS)


func _init_concave_collision_polygon_pool() -> void:
    for i: int in range(MAX_POLYGONS):
        var static_body: StaticBody2D = StaticBody2D.new()
        add_child(static_body)
        static_body.owner = self
        collision_mask_bodies.append(static_body)
        var shape_node: CollisionShape2D = CollisionShape2D.new()
        shape_node.disabled = true
        var concave: ConcavePolygonShape2D = ConcavePolygonShape2D.new()
        shape_node.shape = concave
        static_body.add_child(shape_node)
        shape_node.owner = static_body
        collision_mask_concave_polygons_pool.append(shape_node)


func _on_frame_post_draw() -> void:
    var iTime: float = MaskManager.iTime
    isp_texture.update_scanline_alpha_bucket_bit_masks()
    var scanline_alpha_buckets_top_row: PackedVector2Array
    scanline_alpha_buckets_top_row = isp_texture.fill_scanline_alpha_buckets_top_row()
    var noise_vel = MaskManager.ice_sheets.BufferAShaderMaterial.get_shader_parameter(
        "noiseScrollVelocity"
    )
    var depth = MaskManager.ice_sheets.BufferAShaderMaterial.get_shader_parameter("parallaxDepth")
    var global_coordinate_scale = MaskManager.ice_sheets.BufferAShaderMaterial.get_shader_parameter(
        "globalCoordinateScale"
    )
    var uniform_stretch_correction = (
        MaskManager
        . ice_sheets
        . BufferAShaderMaterial
        . get_shader_parameter("uniformStretchCorrection")
    )
    var stretch_scalar_y = MaskManager.ice_sheets.BufferAShaderMaterial.get_shader_parameter(
        "stretchScalarY"
    )
    var parallax_near_scale = MaskManager.ice_sheets.BufferAShaderMaterial.get_shader_parameter(
        "parallaxNearScale"
    )
    var result: Dictionary = RustUtil.process_scanline_closest_1(
        prev_iTime,
        iTime,
        iResolution.y,
        noise_vel,
        depth,
        global_coordinate_scale,
        uniform_stretch_correction,
        stretch_scalar_y,
        parallax_near_scale,
        scanline_alpha_buckets_top_row,
        collision_polygons,
        previous_scroll_accum,
        scanline_count_per_polygon,
        total_rows_scrolled
    )
    scanline_count_per_polygon = result["scanline_count_per_polygon"]
    collision_polygons = result["collision_polygons"]
    total_rows_scrolled = result["total_rows_scrolled"]
    previous_scroll_accum = result["scroll_accum"]
    prev_iTime = result["prev_time"]
    _update_collision_polygons()


func _update_collision_polygons() -> void:
    for i: int in range(MAX_POLYGONS):
        var segments: PackedVector2Array = collision_polygons[i]
        var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
        var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
        concave.segments = segments
        shape_node.disabled = segments.is_empty()
