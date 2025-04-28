extends Node2D
class_name CollisionMask

var compute_shader_file: RDShaderFile = load(
    "res://Resources/Shaders/Compute/glacier_collision_mask.glsl"
)
var compute_shader_spirv: RDShaderSPIRV
var compute_shader_rid: RID
var compute_pipeline_rid: RID

const MAX_COLLISION_SHAPES: int = 10
var collision_mask_polygons_pool: Array[CollisionPolygon2D] = []

var collision_mask_uniform_set_rid: RID

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


func generate_collision_polygons() -> void:
    var width: int = int(ComputeShaderLayer.iResolution.x)
    var height: int = int(ComputeShaderLayer.iResolution.y)
    var raw_pixel_data: PackedByteArray = ComputeShaderLayer.rd.texture_get_data(
        collision_mask_texture_view_rid, 0
    )
    var collision_polygons: Array[PackedVector2Array] = (
        ComputeShaderLayer
        . rust_util
        . compute_collision_polygons(raw_pixel_data, width, height, TILE_SIZE_PIXELS)
    )
    _update_polygons(collision_polygons)
    #TODO: pixel raw data isnt being updated....
    debug_print_ascii(raw_pixel_data)


func _update_polygons(collision_polygons: Array[PackedVector2Array]) -> void:
    for i: int in range(MAX_COLLISION_SHAPES):
        var collision_polygon: CollisionPolygon2D = collision_mask_polygons_pool[i]
        if i < collision_polygons.size():
            collision_polygon.disabled = false
            collision_polygon.polygon = collision_polygons[i]
        else:
            collision_polygon.disabled = true
            collision_polygon.polygon = []


func _ready() -> void:
    _init_collision_polygon_pool()
    _init_compute_shader_pipeline()
    _init_collision_mask_texture()
    _init_collision_mask_uniform_set()
    RenderingServer.frame_pre_draw.connect(_dispatch_compute)
    RenderingServer.frame_post_draw.connect(generate_collision_polygons)


func _init_collision_polygon_pool() -> void:
    for i: int in range(MAX_COLLISION_SHAPES):
        var static_body: StaticBody2D = StaticBody2D.new()
        add_child(static_body)
        var collision_polygon: CollisionPolygon2D = CollisionPolygon2D.new()
        collision_polygon.disabled = true
        static_body.add_child(collision_polygon)
        collision_mask_polygons_pool.append(collision_polygon)


func _init_compute_shader_pipeline() -> void:
    compute_shader_spirv = compute_shader_file.get_spirv()
    compute_shader_rid = ComputeShaderLayer.rd.shader_create_from_spirv(compute_shader_spirv)
    compute_pipeline_rid = ComputeShaderLayer.rd.compute_pipeline_create(compute_shader_rid)


func _init_collision_mask_texture() -> void:
    collision_mask_texture_format = RDTextureFormat.new()
    collision_mask_texture_format.texture_type = RenderingDevice.TEXTURE_TYPE_2D

    collision_mask_texture_format.format = RenderingDevice.DATA_FORMAT_R8_UINT
    #TODO: image format support bug???
    #collision_mask_texture_format.format = RenderingDevice.DATA_FORMAT_R32_UINT
    #collision_mask_texture_format.format = RenderingDevice.DATA_FORMAT_R8G8B8A8_UNORM

    collision_mask_texture_format.width = int(ComputeShaderLayer.iResolution.x)
    collision_mask_texture_format.height = int(ComputeShaderLayer.iResolution.y)
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
    collision_mask_texture_view_rid = ComputeShaderLayer.rd.texture_create(
        collision_mask_texture_format, collision_mask_view
    )
    collision_mask_texture = Texture2DRD.new()
    collision_mask_texture.set_texture_rd_rid(collision_mask_texture_view_rid)

    collision_mask_uniform = RDUniform.new()
    collision_mask_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_IMAGE
    collision_mask_uniform.binding = COLLISION_MASK_SSBO_UNIFORM_BINDING
    collision_mask_uniform.add_id(collision_mask_texture_view_rid)


func _init_collision_mask_uniform_set() -> void:
    #TODO: this is gross in CollisionMask AND PerspectiveTiltMask
    collision_mask_uniform_set_rid = ComputeShaderLayer.rd.uniform_set_create(
        [collision_mask_uniform], compute_shader_rid, 0
    )


func _dispatch_compute() -> void:
    push_constants = PackedByteArray()
    push_constants.resize(PUSH_CONSTANTS_BYTE_BLOCK_SIZE)
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_0, ComputeShaderLayer.iResolution.x)
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_4, ComputeShaderLayer.iResolution.y)
    #TODO: this iTime global singleton value gets hack updated in the GlacierFlow test scene for now
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_8, iTime)
    push_constants.encode_u32(PUSH_CONSTANTS_BYTE_ALIGNMENT_12, 0)
    ComputeShaderLayer.dispatch_compute(
        compute_pipeline_rid, collision_mask_uniform_set_rid, push_constants
    )


func debug_print_ascii(
    raw_pixel_data: PackedByteArray, tile_width: int = 4, tile_height: int = 8
) -> void:
    var width: int = int(ComputeShaderLayer.iResolution.x)
    var height: int = int(ComputeShaderLayer.iResolution.y)
    var nonzero_pixel_count: int = 0
    for index: int in range(raw_pixel_data.size()):
        if raw_pixel_data[index] == 1:
            nonzero_pixel_count += 1
    print(" non-zero mask pixels:", nonzero_pixel_count)

    var tile_column_count: int = _calculate_tile_column_count(width, tile_width)
    var tile_row_count: int = _calculate_tile_row_count(height, tile_height)
    print(
        "ASCII square:",
        tile_column_count,
        "×",
        tile_row_count,
        "(tile_w=",
        tile_width,
        ", tile_h=",
        tile_height,
        ")"
    )

    for row_index: int in range(tile_row_count):
        var sample_y_position: int = clamp(row_index * tile_height + tile_height / 2, 0, height - 1)
        var source_y: int = height - 1 - sample_y_position
        var line_text: String = ""
        for column_index: int in range(tile_column_count):
            var sample_x_position: int = clamp(
                column_index * tile_width + tile_width / 2, 0, width - 1
            )
            line_text += ("#" if raw_pixel_data[source_y * width + sample_x_position] == 1 else ".")
        print(" ", line_text)


func _calculate_tile_column_count(image_width: int, tile_size: int) -> int:
    return int((image_width + tile_size - 1) / tile_size)


func _calculate_tile_row_count(image_height: int, tile_size: int) -> int:
    return int((image_height + tile_size - 1) / tile_size)
