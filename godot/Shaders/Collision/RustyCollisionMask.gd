extends Node2D
class_name RustyCollisionMask

const MAX_POLYGONS: int = 24

var isp_texture: ISPTexture
var iResolution: Vector2

var collision_mask_bodies: Array[StaticBody2D]
var collision_mask_concave_polygons_pool: Array[CollisionShape2D]

var previous_quantized_vertical_pixel_coord: int = 0
var polygon_active_global: PackedInt32Array
var polygon_active_local: PackedInt32Array
var polygon_positions_y: PackedFloat32Array
var polygon_segments: Array[PackedVector2Array]
var polygon_1d_x_coords: Array[PackedFloat32Array]
var polygon_1d_y_coords: Array[PackedFloat32Array]


func _ready() -> void:
    iResolution = ResolutionManager.resolution
    _init_isp_texture()
    _init_concave_collision_polygon_pool()
    _init_polygon_state_arrays()
    RenderingServer.frame_post_draw.connect(_on_frame_post_draw)


func _init_isp_texture() -> void:
    isp_texture = ISPTexture.new()
    add_child(isp_texture)


func _init_polygon_state_arrays() -> void:
    polygon_active_global.resize(MAX_POLYGONS)
    polygon_active_local.resize(MAX_POLYGONS)
    polygon_positions_y.resize(MAX_POLYGONS)
    polygon_segments.resize(MAX_POLYGONS)
    polygon_1d_x_coords.resize(MAX_POLYGONS)
    polygon_1d_y_coords.resize(MAX_POLYGONS)


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
    var i_time: float = FragmentShaderSignalManager.ice_sheets.iTime
    var scanline_image: Image = (
        FragmentShaderSignalManager.ice_sheets.Scanline.get_texture().get_image()
    )
    isp_texture.update_scanline_mask_from_scanline_image(scanline_image)
    var alpha_buckets: PackedVector2Array = isp_texture.get_alpha_buckets_in_scanline()
    var result: Dictionary = RustUtilSingleton.rust_util.process_scanline(
        i_time,
        alpha_buckets,
        previous_quantized_vertical_pixel_coord,
        polygon_active_global,
        polygon_active_local,
        polygon_positions_y,
        polygon_segments,
        polygon_1d_x_coords,
        polygon_1d_y_coords,
        iResolution
    )

    var current_quantized_vertical_pixel_coord: int = result["current_quantized_vertical_pixel_coord"]
    polygon_active_global = result["polygon_active_global"]
    polygon_active_local = result["polygon_active_local"]
    polygon_positions_y = result["polygon_positions_y"]
    polygon_segments = result["polygon_segments"]
    polygon_1d_x_coords = result["polygon_1d_x_coords"]
    polygon_1d_y_coords = result["polygon_1d_y_coords"]

    previous_quantized_vertical_pixel_coord = current_quantized_vertical_pixel_coord
    _update_collision_polygons()

    if Engine.get_frames_drawn() % 60 == 0:
        var seg0 := polygon_segments[0]
        print(
            "dbg poly[0] segs:",
            seg0.size(),
            " y:",
            polygon_positions_y[0],
            " active:",
            polygon_active_global[0]
        )


func _update_collision_polygons() -> void:
    for i: int in range(MAX_POLYGONS):
        var segments: PackedVector2Array = polygon_segments[i]
        var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
        var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
        concave.segments = segments
        shape_node.position.y = polygon_positions_y[i]
        shape_node.disabled = segments.is_empty() or polygon_active_global[i] == 0
