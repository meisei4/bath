extends Node2D
#class_name CollisionMask

var compute_shader_file: RDShaderFile = load(
    "res://Resources/Shaders/Compute/glacier_collision_mask.glsl"
)
var compute_shader_spirv: RDShaderSPIRV
var compute_shader_rid: RID
var compute_pipeline_rid: RID

const MAX_COLLISION_SHAPES: int = 12
var collision_mask_polygons_pool: Array[CollisionPolygon2D] = []
var collision_polygon_hulls_pool: Array[PackedVector2Array] = []
var pixel_mask_array_pool: PackedByteArray = PackedByteArray()
var tile_solidness_array_pool: PackedByteArray = PackedByteArray()
var visited_array_pool: PackedByteArray = PackedByteArray()

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


func generate_collision_polygons() -> void:
    #debug_print_ascii()
    var width: int = int(ComputeShaderLayer.iResolution.x)
    var height: int = int(ComputeShaderLayer.iResolution.y)
    var tile_column_count: int = ComputeShaderLayer._calculate_tile_column_count(
        width, ComputeShaderLayer.TILE_SIZE_PIXELS
    )
    var tile_row_count: int = ComputeShaderLayer._calculate_tile_row_count(
        height, ComputeShaderLayer.TILE_SIZE_PIXELS
    )

    var raw_pixel_data: PackedByteArray = ComputeShaderLayer.rendering_device.texture_get_data(
        collision_mask_texture_view_rid, 0
    )
    #_update_pixel_mask_array_pool_rgba8_or_r32ui(raw_pixel_data, width, height)
    ComputeShaderLayer._update_pixel_mask_array_pool_r8ui(raw_pixel_data, width, height)
    ComputeShaderLayer._update_tile_solidness_array(
        width, height, tile_column_count, tile_row_count, ComputeShaderLayer.TILE_SIZE_PIXELS
    )
    var connected_regions: Array[PackedVector2Array] = (
        ComputeShaderLayer
        . _find_all_connected_regions_in_tile_array_packed(tile_column_count, tile_row_count)
    )
    var boundary_tile_lists: Array[PackedVector2Array] = (
        ComputeShaderLayer
        . _find_boundary_tiles_for_each_region_packed(
            connected_regions, tile_solidness_array_pool, tile_column_count, tile_row_count
        )
    )
    var used: int = ComputeShaderLayer._compute_hull_pool(boundary_tile_lists, width, height)
    #var used: int = _compute_collision_polygons_marching_squares(tile_column_count, tile_row_count, TILE_SIZE_PIXELS)
    ComputeShaderLayer._update_polygons_from_hulls(used)


func debug_print_ascii(tile_width: int = 4, tile_height: int = 8) -> void:
    var width: int = int(ComputeShaderLayer.iResolution.x)
    var height: int = int(ComputeShaderLayer.iResolution.y)
    var nonzero_pixel_count: int = 0
    for index: int in range(pixel_mask_array_pool.size()):
        if pixel_mask_array_pool[index] == 1:
            nonzero_pixel_count += 1
    print(" non-zero mask pixels:", nonzero_pixel_count)

    var tile_column_count: int = ComputeShaderLayer._calculate_tile_column_count(width, tile_width)
    var tile_row_count: int = ComputeShaderLayer._calculate_tile_row_count(height, tile_height)
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
            line_text += (
                "#" if pixel_mask_array_pool[source_y * width + sample_x_position] == 1 else "."
            )
        print(" ", line_text)


func _ready() -> void:
    _init_collision_polygons()
    _init_compute_shader_pipeline()
    _init_collision_mask_texture()
    #TODO: this is gross in CollisionMask AND PerspectiveTiltMask
    collision_mask_uniform_set_rid = ComputeShaderLayer.rendering_device.uniform_set_create(
        [collision_mask_uniform], compute_shader_rid, 0
    )
    _init_pools()
    RenderingServer.frame_pre_draw.connect(_dispatch_compute)
    RenderingServer.frame_post_draw.connect(generate_collision_polygons)


func _init_pools() -> void:
    var w = int(ComputeShaderLayer.iResolution.x)
    var h = int(ComputeShaderLayer.iResolution.y)
    pixel_mask_array_pool.resize(w * h)
    var cols = ComputeShaderLayer._calculate_tile_column_count(
        w, ComputeShaderLayer.TILE_SIZE_PIXELS
    )
    var rows = ComputeShaderLayer._calculate_tile_row_count(h, ComputeShaderLayer.TILE_SIZE_PIXELS)
    tile_solidness_array_pool.resize(cols * rows)
    visited_array_pool.resize(cols * rows)

    collision_polygon_hulls_pool.clear()
    collision_polygon_hulls_pool.resize(MAX_COLLISION_SHAPES)
    for i in range(MAX_COLLISION_SHAPES):
        var hull_buf: PackedVector2Array
        hull_buf.resize(0)
        collision_polygon_hulls_pool[i] = hull_buf


func _init_collision_polygons() -> void:
    for i: int in range(MAX_COLLISION_SHAPES):
        var static_body: StaticBody2D = StaticBody2D.new()
        add_child(static_body)
        var collision_polygon: CollisionPolygon2D = CollisionPolygon2D.new()
        collision_polygon.disabled = true
        static_body.add_child(collision_polygon)
        collision_mask_polygons_pool.append(collision_polygon)


func _init_compute_shader_pipeline() -> void:
    compute_shader_spirv = compute_shader_file.get_spirv()
    compute_shader_rid = ComputeShaderLayer.rendering_device.shader_create_from_spirv(
        compute_shader_spirv
    )
    compute_pipeline_rid = ComputeShaderLayer.rendering_device.compute_pipeline_create(
        compute_shader_rid
    )


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
    collision_mask_texture_view_rid = ComputeShaderLayer.rendering_device.texture_create(
        collision_mask_texture_format, collision_mask_view
    )
    collision_mask_texture = Texture2DRD.new()
    collision_mask_texture.set_texture_rd_rid(collision_mask_texture_view_rid)

    collision_mask_uniform = RDUniform.new()
    collision_mask_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_IMAGE
    collision_mask_uniform.binding = COLLISION_MASK_SSBO_UNIFORM_BINDING
    collision_mask_uniform.add_id(collision_mask_texture_view_rid)


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
