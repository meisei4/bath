extends Node2D
class_name CollisionMaskIncrementalScanlinePolygonizer

const MAX_POLYGONS: int = 4

var isp_texture: ISPTexture
var iResolution: Vector2

var collision_mask_bodies: Array[StaticBody2D]
var collision_mask_concave_polygons_pool: Array[CollisionShape2D]
var collision_polygons: Array[PackedVector2Array]
var scanline_count_per_polygon: PackedInt32Array

var previous_scroll_accum: float = 0
var prev_iTime: float
var speed_px_per_sec: float


func _ready() -> void:
    var prev_iTime = MaskManager.iTime
    iResolution = ResolutionManager.resolution
    _init_concave_collision_polygon_pool()
    _init_polygon_state_arrays()
    MaskManager.ice_sheets_entered_scene.connect(_on_ice_sheets_entered)
    if MaskManager.ice_sheets:
        _on_ice_sheets_entered(MaskManager.ice_sheets)
        speed_px_per_sec = shader_scroll_speed_px_per_sec(
            MaskManager.ice_sheets.BufferAShaderMaterial, iResolution.y
        )


func _on_ice_sheets_entered(ice_sheets: IceSheets) -> void:
    if isp_texture:
        return

    isp_texture = ISPTexture.new()
    isp_texture.TargetFrameBuffer = ice_sheets.BufferA
    add_child(isp_texture)
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


func shader_scroll_speed_px_per_sec(mat: ShaderMaterial, screen_h: float) -> float:
    var vel_y = mat.get_shader_parameter("noiseScrollVelocity").y
    var stretch_y = mat.get_shader_parameter("stretchScalarY")
    var gscale = mat.get_shader_parameter("globalCoordinateScale")
    var near_scale = mat.get_shader_parameter("parallaxNearScale")
    # WRONG: assumed px/s came from noise‐space × stretch × scale × near_scale
    # return vel_y * stretch_y * gscale * near_scale
    # CORRECT: convert noise‐units/s → screen‐pixels/s by dividing out the per‐row projection factor at Y = –1
    var depth = mat.get_shader_parameter("parallaxDepth")
    var a = 0.5 * log((depth + 1.0) / (depth - 1.0))
    var b = 1.5 * (depth * log((depth + 1.0) / (depth - 1.0)) - 2.0)
    var scaleY_top = a + b * -1.0
    return vel_y * screen_h / (2.0 * scaleY_top)


func _on_frame_post_draw() -> void:
    var iTime: float = MaskManager.iTime
    var delta_time: float = iTime - prev_iTime
    prev_iTime = iTime
    isp_texture.update_scanline_alpha_bucket_bit_masks()
    var scanline_alpha_buckets_top_row: PackedVector2Array
    scanline_alpha_buckets_top_row = isp_texture.fill_scanline_alpha_buckets_top_row()
    var vel_y = (
        MaskManager.ice_sheets.BufferAShaderMaterial.get_shader_parameter("noiseScrollVelocity").y
    )
    var depth = MaskManager.ice_sheets.BufferAShaderMaterial.get_shader_parameter("parallaxDepth")
    var result: Dictionary = (
        RustUtilSingleton
        . rust_util
        . process_scanline(
            delta_time,
            iResolution.y,
            vel_y,
            depth,
            scanline_alpha_buckets_top_row,
            collision_polygons,
            previous_scroll_accum,
            scanline_count_per_polygon,
        )
    )
    previous_scroll_accum = result["scroll_accum"]
    scanline_count_per_polygon = result["scanline_count_per_polygon"]
    collision_polygons = result["collision_polygons"]
    _update_collision_polygons()


func _update_collision_polygons() -> void:
    for i: int in range(MAX_POLYGONS):
        var segments: PackedVector2Array = collision_polygons[i]
        var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
        var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
        concave.segments = segments
        shape_node.disabled = segments.is_empty()
