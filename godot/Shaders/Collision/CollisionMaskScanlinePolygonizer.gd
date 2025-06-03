extends Node2D
class_name CollisionMaskScanlinePolygonizer

const MAX_POLYGONS: int = 8

var isp_texture: ISPTexture
var collision_mask_concave_polygons_pool: Array[CollisionShape2D]
var collision_mask_bodies: Array[StaticBody2D]
var polygon_active_global: PackedInt32Array
var polygon_active_local: PackedInt32Array



func _ready() -> void:
    _init_isp_texture()
    _init_concave_collision_polygon_pool()


func _process(delta: float) -> void:
    var iTime: float = FragmentShaderSignalManager.ice_sheets.iTime
    #TODO: look at res://Resources/Shaders/IceSheets/noise.gdshaderinc magic ass numbers...
    ##define NOISE_SCROLL_VELOCITY       vec2(0.0, 0.05)
    var NOISE_SCROLL_VELOCITY_Y: float = 0.05
    ##define GLOBAL_COORD_SCALAR         180.0
    var GLOBAL_COORD_SCALAR: float = 180.0
    ##define STRETCH_SCALAR_Y            2.0
    var STRETCH_SCALAR_Y: float = 2.0
    ##define UNIFORM_STRETCH_CORRECTION_Y            blah
    var UNIFORM_STRETCH_CORRECTION_Y: float = sqrt(2.0)
    #var UNIFORM_STRETCH_CORRECTION_Y: float = 1.0
    var scroll_pixels: float = (
        iTime
        * NOISE_SCROLL_VELOCITY_Y
        * GLOBAL_COORD_SCALAR
        * STRETCH_SCALAR_Y
        * UNIFORM_STRETCH_CORRECTION_Y
    )
    var cpu_rows_built: int = polygon_active_local[0]  # any active polygon works

    if Engine.get_frames_drawn() % 20 == 0:
        print("GPU offset=", scroll_pixels,
              "  |  polygon-0 height=", cpu_rows_built)
    #if Engine.get_frames_drawn() % 30 == 0:
        #print("GPU y-offset px ≈", scroll_pixels)


    # TODO: this call to get_image is still super costly especially since we are only wanting
    # the top pixel line as a texture from it.
    # FIGURE OUT HOW TO GET REGION FROM TEXTURE??
    var full_screen_image: Image = (
        FragmentShaderSignalManager.ice_sheets.BufferA.get_texture().get_image()
    )
    isp_texture.update_from_full_screen_image(full_screen_image)
    _update_polygons_with_edge_buckets(isp_texture.get_edge_buckets_in_scanline())
    _advance_polygons_by_scanline_height()


func _init_isp_texture() -> void:
    isp_texture = ISPTexture.new()
    add_child(isp_texture)


func _init_concave_collision_polygon_pool() -> void:
    polygon_active_global.resize(MAX_POLYGONS)
    polygon_active_global.fill(0)
    polygon_active_local.resize(MAX_POLYGONS)         # <-- NEW
    polygon_active_local.fill(0)                      # <-- NEW
    for i: int in range(MAX_POLYGONS):
        var static_body: StaticBody2D = StaticBody2D.new()
        add_child(static_body)
        var shape_node: CollisionShape2D = CollisionShape2D.new()
        shape_node.disabled = true
        var concave: ConcavePolygonShape2D = ConcavePolygonShape2D.new()
        shape_node.shape = concave
        static_body.add_child(shape_node)
        collision_mask_bodies.append(static_body)
        collision_mask_concave_polygons_pool.append(shape_node)


func _update_polygons_with_edge_buckets(edge_buckets: PackedVector2Array) -> void:
    var num_buckets: int = edge_buckets.size() / 2
    for bucket_index: int in range(num_buckets):
        var bucket_start: Vector2 = edge_buckets[bucket_index * 2]
        var bucket_end: Vector2   = edge_buckets[bucket_index * 2 + 1]
        var bucket_x_start: float = bucket_start.x
        var bucket_x_end:   float = bucket_end.x
        var matched: bool = false
        for i: int in range(MAX_POLYGONS):
            if polygon_active_global[i] == 1:
                var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
                var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
                var segments: PackedVector2Array = concave.segments
                var seg_size: int = segments.size()
                if seg_size >= 2:
                    var last_left:  float = segments[seg_size - 2].x
                    var last_right: float = segments[seg_size - 1].x
                    if bucket_x_start <= last_right and bucket_x_end >= last_left:
                        var y_local: int = polygon_active_local[i]
                        segments.push_back(Vector2(bucket_x_start, y_local))
                        segments.push_back(Vector2(bucket_x_end,   y_local))
                        concave.segments = segments
                        polygon_active_local[i] += 1
                        matched = true
                        break

        if not matched:
            for i: int in range(MAX_POLYGONS):
                if polygon_active_global[i] == 0:
                    polygon_active_global[i] = 1
                    polygon_active_local[i]    = 1
                    var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
                    var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
                    var new_segments: PackedVector2Array = PackedVector2Array()
                    new_segments.push_back(Vector2(bucket_x_start, 0))
                    new_segments.push_back(Vector2(bucket_x_end,   0))
                    concave.segments = new_segments
                    shape_node.disabled = false
                    break


func _advance_polygons_by_scanline_height() -> void:
    # 1 px per scan-line keeps the mesh welded to the ice sheet
    for i: int in range(MAX_POLYGONS):
        if polygon_active_global[i] == 1:
            var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
            shape_node.position += Vector2(0, isp_texture.TEXTURE_HEIGHT)

            # recycle when the whole shape has scrolled off-screen
            if shape_node.position.y > ResolutionManager.resolution.y:
                _clear_polygon(i)

func _clear_polygon(index: int) -> void:
    polygon_active_global[index] = 0
    polygon_active_local[index]  = 0            # ← NEW: reset height counter
    var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[index]
    var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
    concave.segments = PackedVector2Array()
    shape_node.position = Vector2.ZERO          # keep it parked at origin
    shape_node.disabled = true
