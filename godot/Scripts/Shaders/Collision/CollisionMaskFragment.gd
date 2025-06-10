extends Node2D
#TODO: this is no longer a shader wrapper... should probably move it at some point...
class_name CollisionMaskFragment

var iResolution: Vector2

const MAX_COLLISION_SHAPES: int = 8
var collision_mask_concave_polygons_pool: Array[CollisionShape2D] = []
var collision_mask_bodies: Array[StaticBody2D] = []

var polygon_shape_cache: Dictionary[String, Vector2] = {}

const TILE_SIZE_PIXELS: int = 4


func _ready() -> void:
    iResolution = ResolutionManager.resolution
    _init_concave_collision_polygon_pool()
    RenderingServer.frame_post_draw.connect(_on_frame_post_draw)


func _on_frame_post_draw() -> void:
    #TODO: figure out how to prevent this call every fucking frame jesus: probably not possible
    var img: Image = FragmentShaderSignalManager.ice_sheets.BufferA.get_texture().get_image()
    img.flip_y()
    img.convert(Image.FORMAT_RGBA8)  # Fast conversion to RGBA8?? seems dangerous
    var raw_rgba: PackedByteArray = img.get_data()
    var w: int = int(iResolution.x)
    var h: int = int(iResolution.y)
    var mask_data: PackedByteArray
    mask_data.resize(w * h)
    for i: int in range(w * h):
        mask_data[i] = raw_rgba[4 * i + 3]

    var collision_polygons: Array[PackedVector2Array] = (
        RustUtilSingleton
        . rust_util
        . compute_concave_collision_polygons(mask_data, w, h, TILE_SIZE_PIXELS)
    )
    _update_concave_polygons(collision_polygons)
    #debug_print_ascii(mask_data)


func _init_concave_collision_polygon_pool() -> void:
    for i: int in range(MAX_COLLISION_SHAPES):
        var static_body: StaticBody2D = StaticBody2D.new()
        add_child(static_body)
        var shape_node: CollisionShape2D = CollisionShape2D.new()
        shape_node.disabled = true
        var concave: ConcavePolygonShape2D = ConcavePolygonShape2D.new()
        shape_node.shape = concave
        static_body.add_child(shape_node)
        collision_mask_bodies.append(static_body)
        collision_mask_concave_polygons_pool.append(shape_node)


var debug_dots: Array[ColorRect] = []


func _update_concave_polygons(collision_polygons: Array[PackedVector2Array]) -> void:
    print("DEBUG: _update_concave_polygons: prev cache size =", polygon_shape_cache.size())
    var new_polygon_cache: Dictionary[String, Vector2] = {}
    var previous_centroids: Array[Vector2] = []
    var previous_centroid_matched: Array[bool] = []
    for key: String in polygon_shape_cache.keys():
        previous_centroids.append(polygon_shape_cache[key] as Vector2)
        previous_centroid_matched.append(false)

    for i: int in range(MAX_COLLISION_SHAPES):
        var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
        if i >= collision_polygons.size():
            shape_node.disabled = true
            continue

        shape_node.disabled = false
        var poly: PackedVector2Array = collision_polygons[i]
        var centroid: Vector2 = Vector2.ZERO
        var touching_top: bool = false
        var touching_bottom: bool = false
        for pt: Vector2 in poly:
            centroid += pt
            if pt.y <= 0.0:
                touching_top = true
            if pt.y >= iResolution.y:
                touching_bottom = true
        centroid /= poly.size()
        var fully_inside: bool = not touching_top and not touching_bottom
        var best_match_idx: int = -1
        var best_dist: float = INF
        var MATCH_THRESHOLD: float = TILE_SIZE_PIXELS * 4.0
        for j: int in range(previous_centroids.size()):
            if previous_centroid_matched[j]:
                continue
            var dist: float = centroid.distance_to(previous_centroids[j])
            if dist < best_dist and dist < MATCH_THRESHOLD:
                best_dist = dist
                best_match_idx = j

        if best_match_idx != -1:
            previous_centroid_matched[best_match_idx] = true
            var displacement: Vector2 = centroid - previous_centroids[best_match_idx]
            var label: String = ""

            if touching_bottom:
                label = "TOUCHING BOTTOM → REMOVE"
            elif touching_top:
                label = "TOUCHING TOP"
            elif fully_inside:
                label = "FULLY INSIDE"
            else:
                label = "PARTIAL"

            print(
                "  [idx=",
                i,
                "] MATCHED prev_idx=",
                best_match_idx,
                " dy=",
                displacement.y,
                " label=",
                label,
                " dist=",
                best_dist
            )

            if touching_bottom:
                continue
        else:
            print(
                "  NEW polygon [idx=",
                i,
                "] vcount=",
                poly.size(),
                " centroid=(",
                centroid.x,
                ",",
                centroid.y,
                ")"
            )

        if not touching_bottom:
            new_polygon_cache[str(i)] = centroid

        var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
        var segments: PackedVector2Array = PackedVector2Array()
        for j: int in range(poly.size()):
            segments.push_back(poly[j])
            segments.push_back(poly[(j + 1) % poly.size()])

        concave.segments = segments

    polygon_shape_cache = new_polygon_cache
    while debug_dots.size() < new_polygon_cache.size():
        var dot: ColorRect = ColorRect.new()
        dot.color = Color(1, 0, 0)
        dot.size = Vector2(4, 4)
        dot.z_index = 4
        add_child(dot)
        debug_dots.append(dot)

    while debug_dots.size() > new_polygon_cache.size():
        debug_dots.pop_back().queue_free()

    var d: int = 0
    for c in new_polygon_cache.values():
        debug_dots[d].position = c - debug_dots[d].size * 0.5
        d += 1

    print("DEBUG: _update_concave_polygons: new cache size =", polygon_shape_cache.size())


func debug_print_ascii(
    raw_pixel_data: PackedByteArray, tile_width: int = 8, tile_height: int = 16
) -> void:
    var width: int = int(iResolution.x)
    var height: int = int(iResolution.y)
    var cols: int = _calculate_tile_column_count(width, tile_width)
    var rows: int = _calculate_tile_row_count(height, tile_height)
    for row: int in range(rows):
        var sample_y: int = clampi(row * tile_height + tile_height / 2, 0, height - 1)
        var line_text: String = ""
        for col: int in range(cols):
            var sample_x: int = clampi(col * tile_width + tile_width / 2, 0, width - 1)
            var byte: float = raw_pixel_data[sample_y * width + sample_x]
            line_text += "#" if byte != 0 else "."
        print(" ", line_text)


func _calculate_tile_column_count(image_width: int, tile_size: int) -> int:
    return (image_width + tile_size - 1) / tile_size


func _calculate_tile_row_count(image_height: int, tile_size: int) -> int:
    return (image_height + tile_size - 1) / tile_size
