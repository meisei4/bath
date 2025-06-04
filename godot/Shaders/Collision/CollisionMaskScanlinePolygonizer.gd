extends Node2D
class_name CollisionMaskScanlinePolygonizer

const MAX_POLYGONS: int = 8

var isp_texture: ISPTexture
var collision_mask_concave_polygons_pool: Array[CollisionShape2D]
var collision_mask_bodies: Array[StaticBody2D]
var polygon_active_global: PackedInt32Array
var polygon_active_local: PackedInt32Array

var previous_frame_count: int = 0
var previous_rows_scrolled: int = 0


func _ready() -> void:
    _init_isp_texture()
    _init_concave_collision_polygon_pool()
    RenderingServer.frame_post_draw.connect(_on_frame_post_draw)


func _on_frame_post_draw() -> void:
    var iFrameCount: int = FragmentShaderSignalManager.ice_sheets.iFrameCount
    if iFrameCount == previous_frame_count:
        return
    print("FrameCount=", iFrameCount)
    var iTime: float = FragmentShaderSignalManager.ice_sheets.iTime
    ##define NOISE_SCROLL_VELOCITY       vec2(0.0, 0.05)
    var NOISE_SCROLL_VELOCITY: Vector2 = Vector2(0.0, 0.05)
    ##define GLOBAL_COORD_SCALAR         180.0
    var GLOBAL_COORD_SCALAR: float = 180.0
    ##define STRETCH_SCALAR_Y            2.0
    var STRETCH_SCALAR_Y: float = 2.0
    var UNIFORM_STRETCH_CORRECTION_SCALAR: float = sqrt(2.0)
    ##define UNIFORM_STRETCH_CORRECTION_SCALAR     sqrt(2.0)
    previous_frame_count = iFrameCount
    var scanline_image: Image = (
        FragmentShaderSignalManager.ice_sheets.Scanline.get_texture().get_image()
    )
    isp_texture.update_scanline_mask_from_scanline_image(scanline_image)
    var buckets: PackedVector2Array = isp_texture.get_edge_buckets_in_scanline()
    var arbitrary_virtual_frame_rate: float = 1.0 / 60.0
    var continuous_full: float = (
        float(iFrameCount)
        * arbitrary_virtual_frame_rate
        * NOISE_SCROLL_VELOCITY.y
        * GLOBAL_COORD_SCALAR
        * STRETCH_SCALAR_Y
        * UNIFORM_STRETCH_CORRECTION_SCALAR
    )
    var discrete_full: int = floori(continuous_full)
    var new_rows_this_frame: int = discrete_full - previous_rows_scrolled
    previous_rows_scrolled = discrete_full
    for i in range(new_rows_this_frame):
        _update_polygons_with_edge_buckets(buckets)
        _advance_polygons_by_scanline_height()


func _init_isp_texture() -> void:
    isp_texture = ISPTexture.new()
    add_child(isp_texture)


func _init_concave_collision_polygon_pool() -> void:
    polygon_active_global.resize(MAX_POLYGONS)
    polygon_active_global.fill(0)
    polygon_active_local.resize(MAX_POLYGONS)
    polygon_active_local.fill(0)
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
        var bucket_end: Vector2 = edge_buckets[bucket_index * 2 + 1]
        var bucket_x_start: float = bucket_start.x
        var bucket_x_end: float = bucket_end.x
        var matched: bool = false
        for i: int in range(MAX_POLYGONS):
            if polygon_active_global[i] == 1:
                var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
                var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
                var segments: PackedVector2Array = concave.segments
                var seg_size: int = segments.size()
                if seg_size >= 2:
                    var top_left: float = segments[0].x
                    var top_right: float = segments[1].x
                    if bucket_x_start <= top_right and bucket_x_end >= top_left:
                        var localY: int = -int(shape_node.position.y)
                        segments.insert(0, Vector2(bucket_x_end, localY))
                        segments.insert(0, Vector2(bucket_x_start, localY))
                        concave.segments = segments
                        polygon_active_local[i] += 1
                        matched = true
                        break

        if not matched:
            for i: int in range(MAX_POLYGONS):
                if polygon_active_global[i] == 0:
                    polygon_active_global[i] = 1
                    polygon_active_local[i] = 1
                    var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
                    var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
                    var new_segments: PackedVector2Array = PackedVector2Array()
                    new_segments.push_back(Vector2(bucket_x_start, 0))
                    new_segments.push_back(Vector2(bucket_x_end, 0))
                    concave.segments = new_segments
                    shape_node.disabled = false
                    break


func _advance_polygons_by_scanline_height() -> void:
    for i: int in range(MAX_POLYGONS):
        if polygon_active_global[i] == 1:
            var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
            var old_y: float = shape_node.position.y
            shape_node.position.y += isp_texture.TEXTURE_HEIGHT
            var new_y: float = shape_node.position.y
            print("  [ADVANCE] Polygon slot=", i, " moved from Y=", old_y, " to Y=", new_y)
            if shape_node.position.y > ResolutionManager.resolution.y:
                _clear_polygon(i)


func _clear_polygon(index: int) -> void:
    polygon_active_global[index] = 0
    polygon_active_local[index] = 0
    var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[index]
    var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
    concave.segments = PackedVector2Array()
    shape_node.position = Vector2.ZERO
    shape_node.disabled = true
    print("  [CLEAR]   Polygon slot=", index, " reset to origin and disabled")
