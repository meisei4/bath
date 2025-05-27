extends Node2D
class_name CollisionMaskFragment

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = load(
    "res://Resources/Shaders/Collision/collision_mask_fragment.gdshader"
)
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport
var MainImage: TextureRect
var iResolution: Vector2
var iTime: float

const MAX_COLLISION_SHAPES: int = 8
var collision_mask_concave_polygons_pool: Array[CollisionShape2D] = []
var collision_mask_bodies: Array[StaticBody2D] = []

const TILE_SIZE_PIXELS: int = 2


func _ready() -> void:
    ComputeShaderSignalManager.register_collision_mask_fragment(self)
    iResolution = ResolutionManager.resolution
    BufferA = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferA.render_target_clear_mode = SubViewport.CLEAR_MODE_ALWAYS
    BufferA.transparent_bg = true
    BufferA.use_hdr_2d = false
    BufferAShaderMaterial = ShaderMaterial.new()
    BufferAShaderMaterial.shader = BufferAShader
    BufferAShaderMaterial.set_shader_parameter("iResolution", iResolution)
    BufferAShaderNode = ColorRect.new()
    BufferAShaderNode.size = iResolution
    BufferAShaderNode.material = BufferAShaderMaterial
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    MainImage = TextureRect.new()
    MainImage.texture = BufferA.get_texture()
    MainImage.flip_v = true
    add_child(MainImage)
    _init_concave_collision_polygon_pool()
    RenderingServer.frame_post_draw.connect(_on_frame_post_draw)


func _process(_delta: float) -> void:
    BufferAShaderMaterial.set_shader_parameter("iTime", iTime)


func get_collision_mask_texture_fragment() -> Texture:
    return BufferA.get_texture()


func _on_frame_post_draw() -> void:
    var img: Image = BufferA.get_texture().get_image()
    var raw_rgba: PackedByteArray = img.get_data()
    var width: int = int(iResolution.x)
    var height: int = int(iResolution.y)

    var raw_pixel_data: PackedByteArray
    raw_pixel_data.resize(width * height)
    for i: int in raw_pixel_data.size():
        var byte: int = raw_rgba[i * 4]
        raw_pixel_data[i] = byte
    var collision_polygons: Array[PackedVector2Array] = (
        MusicDimensionsManager
        . rust_util
        . compute_concave_collision_polygons(raw_pixel_data, width, height, TILE_SIZE_PIXELS)
    )
    _update_concave_polygons(collision_polygons)
    debug_print_ascii(raw_pixel_data)


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
        var sample_y: int = clamp(row * tile_height + tile_height / 2, 0, height - 1)
        var line_text: String = ""
        for col: int in range(cols):
            var sample_x: int = clamp(col * tile_width + tile_width / 2, 0, width - 1)
            var byte: float = raw_pixel_data[sample_y * width + sample_x]
            line_text += "#" if byte != 0 else "."
        print(" ", line_text)


func _calculate_tile_column_count(image_width: int, tile_size: int) -> int:
    return int((image_width + tile_size - 1) / tile_size)


func _calculate_tile_row_count(image_height: int, tile_size: int) -> int:
    return int((image_height + tile_size - 1) / tile_size)
