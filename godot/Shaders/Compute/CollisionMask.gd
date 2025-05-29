extends ComputeShaderPipeline
class_name CollisionMask

const MAX_COLLISION_SHAPES: int = 8
var collision_mask_convex_polygons_pool: Array[CollisionPolygon2D] = []
var collision_mask_concave_polygons_pool: Array[CollisionShape2D] = []
var collision_mask_bodies: Array[StaticBody2D] = []

const COLLISION_MASK_SSBO_UNIFORM_BINDING: int = 0
var collision_mask_uniform: RDUniform
var collision_mask_texture_format: RDTextureFormat
var collision_mask_view: RDTextureView
var collision_mask_texture_view_rid: RID
var collision_mask_texture: Texture2DRD

var push_constants: PackedByteArray
const PUSH_CONSTANTS_BYTE_BLOCK_SIZE: int = 16
const PUSH_CONSTANTS_BYTE_ALIGNMENT_0: int = 0
const PUSH_CONSTANTS_BYTE_ALIGNMENT_4: int = 4
const PUSH_CONSTANTS_BYTE_ALIGNMENT_8: int = 8
const PUSH_CONSTANTS_BYTE_ALIGNMENT_12: int = 12

var iTime: float

const TILE_SIZE_PIXELS: int = 2


func _ready() -> void:
    #ComputeShaderSignalManager.register_collision_mask(self)
    _init_shader()
    _init_compute_shader_pipeline()
    _init_convex_collision_polygon_pool()
    _init_concave_collision_polygon_pool()
    _init_collision_mask_uniform()
    _init_uniform_set()
    RenderingServer.frame_pre_draw.connect(_dispatch_compute)
    RenderingServer.frame_post_draw.connect(generate_collision_polygons)


func _init_shader() -> void:
    compute_shader_file = preload("res://Resources/Shaders/Compute/ice_sheet_collision_mask.glsl")


func generate_collision_polygons() -> void:
    var width: int = int(iResolution.x)
    var height: int = int(iResolution.y)
    var raw_pixel_data: PackedByteArray = rendering_device.texture_get_data(
        collision_mask_texture_view_rid, 0
    )
    #var collision_polygons: Array[PackedVector2Array] = rust_util.compute_convex_collision_polygons(
    #raw_pixel_data, width, height, TILE_SIZE_PIXELS
    #)
    #_update_convex_polygons(collision_polygons)
    var collision_polygons: Array[PackedVector2Array] = (
        rust_util
        . compute_concave_collision_polygons(raw_pixel_data, width, height, TILE_SIZE_PIXELS)
    )
    _update_concave_polygons(collision_polygons)
    #TODO:so the ascii works only for a while and then it just prints out the same spot in the window idk why
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


#TODO: something is causing certain collision polygons to sometimes linger/not update correctly with the fragment texture
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


func _init_convex_collision_polygon_pool() -> void:
    for i: int in range(MAX_COLLISION_SHAPES):
        var static_body: StaticBody2D = StaticBody2D.new()
        add_child(static_body)
        var collision_polygon: CollisionPolygon2D = CollisionPolygon2D.new()
        collision_polygon.disabled = true
        static_body.add_child(collision_polygon)
        collision_mask_bodies.append(static_body)
        collision_mask_convex_polygons_pool.append(collision_polygon)


func _update_convex_polygons(collision_polygons: Array[PackedVector2Array]) -> void:
    for i: int in range(MAX_COLLISION_SHAPES):
        var collision_polygon: CollisionPolygon2D = collision_mask_convex_polygons_pool[i]
        if i < collision_polygons.size():
            collision_polygon.disabled = false
            collision_polygon.polygon = collision_polygons[i]
        else:
            collision_polygon.disabled = true
            collision_polygon.polygon = []


func _init_collision_mask_uniform() -> void:
    collision_mask_texture_format = RDTextureFormat.new()
    collision_mask_texture_format.texture_type = RenderingDevice.TEXTURE_TYPE_2D
    #TODO: godot does not support using unsigned 8 bit ints.. so we have to use unorm float
    # see https://github.com/godotengine/godot/blob/6c9765d87e142e786f0190783f41a0250a835c99/servers/rendering/renderer_rd/storage_rd/texture_storage.cpp#L2281C1-L2664C1
    #collision_mask_texture_format.format = RenderingDevice.DATA_FORMAT_R8_UINT
    collision_mask_texture_format.format = RenderingDevice.DATA_FORMAT_R8_UNORM
    collision_mask_texture_format.width = int(iResolution.x)
    collision_mask_texture_format.height = int(iResolution.y)
    collision_mask_texture_format.depth = 1
    collision_mask_texture_format.array_layers = 1
    collision_mask_texture_format.mipmaps = 1
    collision_mask_texture_format.usage_bits = (
        RenderingDevice.TEXTURE_USAGE_STORAGE_BIT
        | RenderingDevice.TEXTURE_USAGE_CAN_UPDATE_BIT
        | RenderingDevice.TEXTURE_USAGE_SAMPLING_BIT
        | RenderingDevice.TEXTURE_USAGE_CAN_COPY_FROM_BIT
    )
    collision_mask_view = RDTextureView.new()
    collision_mask_texture_view_rid = rendering_device.texture_create(
        collision_mask_texture_format, collision_mask_view
    )
    collision_mask_texture = Texture2DRD.new()
    collision_mask_texture.set_texture_rd_rid(collision_mask_texture_view_rid)
    collision_mask_uniform = RDUniform.new()
    collision_mask_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_IMAGE
    collision_mask_uniform.binding = COLLISION_MASK_SSBO_UNIFORM_BINDING
    collision_mask_uniform.add_id(collision_mask_texture_view_rid)
    uniform_set.append(collision_mask_uniform)


func _dispatch_compute() -> void:
    push_constants = PackedByteArray()
    push_constants.resize(PUSH_CONSTANTS_BYTE_BLOCK_SIZE)
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_0, iResolution.x)
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_4, iResolution.y)
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_8, iTime)
    push_constants.encode_u32(PUSH_CONSTANTS_BYTE_ALIGNMENT_12, 0)
    super.dispatch_compute(push_constants)


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
            # treat any non-zero byte (i.e. 255) as “on”:
            line_text += "#" if byte != 0 else "."
        print(" ", line_text)


func _calculate_tile_column_count(image_width: int, tile_size: int) -> int:
    return (image_width + tile_size - 1) / tile_size


func _calculate_tile_row_count(image_height: int, tile_size: int) -> int:
    return (image_height + tile_size - 1) / tile_size
