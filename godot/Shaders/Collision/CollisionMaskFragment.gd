extends Node2D
class_name CollisionMaskFragment

var iResolution: Vector2

const MAX_COLLISION_SHAPES: int = 8
var collision_mask_concave_polygons_pool: Array[CollisionShape2D] = []
var collision_mask_bodies: Array[StaticBody2D] = []

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
    debug_print_ascii(mask_data)


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


func _update_concave_polygons(collision_polygons: Array[PackedVector2Array]) -> void:
    for i: int in range(MAX_COLLISION_SHAPES):
        var collision_shape: CollisionShape2D = collision_mask_concave_polygons_pool[i]
        if i < collision_polygons.size():
            collision_shape.disabled = false
            var collision_polygon: PackedVector2Array = collision_polygons[i]
            var segments: PackedVector2Array = PackedVector2Array()
            for j: int in range(collision_polygon.size()):
                var a: Vector2 = collision_polygon[j]
                var b: Vector2 = collision_polygon[(j + 1) % collision_polygon.size()]
                segments.push_back(a)
                segments.push_back(b)
            collision_shape.shape.segments = segments
        else:
            collision_shape.disabled = true


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
