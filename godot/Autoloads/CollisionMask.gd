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
    debug_print_ascii()
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
    var used: int = ComputeShaderLayer._compute_hull_pool_gpu(boundary_tile_lists)
    #var used: int = ComputeShaderLayer._compute_hull_pool_gpu_optimized(boundary_tile_lists)
    #var used: int = ComputeShaderLayer._compute_hull_pool_cpu(boundary_tile_lists)
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

    #GPU OFFLOAD OF HULL COMPUTATIONS
    _init_hull_compute_pipeline()
    #_init_hull_ssbos_optimized()
    #_init_hull_uniform_set_optimized()
    _init_hull_ssbos()
    _init_hull_uniform_set()


func _init_pools() -> void:
    var w: int = int(ComputeShaderLayer.iResolution.x)
    var h: int = int(ComputeShaderLayer.iResolution.y)
    pixel_mask_array_pool.resize(w * h)
    var cols: int = ComputeShaderLayer._calculate_tile_column_count(
        w, ComputeShaderLayer.TILE_SIZE_PIXELS
    )
    var rows: int = ComputeShaderLayer._calculate_tile_row_count(
        h, ComputeShaderLayer.TILE_SIZE_PIXELS
    )
    tile_solidness_array_pool.resize(cols * rows)
    visited_array_pool.resize(cols * rows)

    collision_polygon_hulls_pool.clear()
    collision_polygon_hulls_pool.resize(MAX_COLLISION_SHAPES)
    for i: int in range(MAX_COLLISION_SHAPES):
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


var andrew_shader_file: RDShaderFile = load("res://Resources/Shaders/Compute/hulls.glsl")
#var andrew_shader_file: RDShaderFile = load("res://Resources/Shaders/Compute/hulls_optimized.glsl")
var andrew_shader_spirv: RDShaderSPIRV
var andrew_shader_rid: RID
var andrew_pipeline_rid: RID

var hull_boundary_uniform: RDUniform
var hull_sorted_uniform: RDUniform
var hull_meta_uniform: RDUniform
var hull_result_uniform: RDUniform
var hull_uniform_set_rid: RID

var boundary_ssbo_rid: RID
var sorted_ssbo_rid: RID
var meta_ssbo_rid: RID
var result_ssbo_rid: RID

const MAX_HULL_POINTS: int = 256
const HULL_BOUNDARY_SSBO_BINDING: int = 0
const HULL_SORTED_SSBO_BINDING: int = 1
const HULL_RESULT_SSBO_BINDING: int = 2


func _init_hull_compute_pipeline() -> void:
    andrew_shader_spirv = andrew_shader_file.get_spirv()
    andrew_shader_rid = ComputeShaderLayer.rendering_device.shader_create_from_spirv(
        andrew_shader_spirv
    )
    andrew_pipeline_rid = ComputeShaderLayer.rendering_device.compute_pipeline_create(
        andrew_shader_rid
    )


func _init_hull_ssbos_optimized() -> void:
    var maxRegions = MAX_COLLISION_SHAPES
    var maxPts = MAX_HULL_POINTS

    # — boundary SSBO: all regions × max points × 8 bytes
    var boundary_capacity = maxRegions * maxPts * 8
    var bdata = PackedByteArray()
    bdata.resize(boundary_capacity)
    boundary_ssbo_rid = ComputeShaderLayer.rendering_device.storage_buffer_create(
        boundary_capacity, bdata
    )
    print("→ boundary SSBO capacity:", boundary_capacity)

    # — meta SSBO: one (offset,uint) pair = 8 bytes per region
    var meta_capacity = maxRegions * 8
    var mdata = PackedByteArray()
    mdata.resize(meta_capacity)
    meta_ssbo_rid = ComputeShaderLayer.rendering_device.storage_buffer_create(meta_capacity, mdata)
    print("→ meta     SSBO capacity:", meta_capacity)

    # — result SSBO:
    #    counts: maxRegions×4 bytes
    #    pad   : to next 8‐byte boundary
    #    array : maxRegions×maxPts×8 bytes
    var counts_bytes = maxRegions * 4
    var pad = (8 - (counts_bytes % 8)) % 8
    var array_bytes = maxRegions * maxPts * 8
    var result_capacity = counts_bytes + pad + array_bytes
    var rdata = PackedByteArray()
    rdata.resize(result_capacity)
    result_ssbo_rid = ComputeShaderLayer.rendering_device.storage_buffer_create(
        result_capacity, rdata
    )
    print("→ result   SSBO capacity:", result_capacity)


func _init_hull_ssbos() -> void:
    # boundary & sorted both hold up to MAX_HULL_POINTS vec2's → 8 bytes each
    var boundary_size: int = MAX_HULL_POINTS * 8
    var bdata: PackedByteArray = PackedByteArray()
    bdata.resize(boundary_size)
    print("→ boundary SSBO capacity:", bdata.size())  # should print 2048 for MAX_HULL_POINTS=256
    boundary_ssbo_rid = ComputeShaderLayer.rendering_device.storage_buffer_create(
        boundary_size, bdata
    )

    var sdata: PackedByteArray = PackedByteArray()
    sdata.resize(boundary_size)
    print("→ sorted   SSBO capacity:", sdata.size())
    sorted_ssbo_rid = ComputeShaderLayer.rendering_device.storage_buffer_create(
        boundary_size, sdata
    )

    # result SSBO holds a uint (4 bytes) + pad (4) + MAX_HULL_POINTS×8
    var result_size: int = 4 + 4 + MAX_HULL_POINTS * 8
    var rdata: PackedByteArray = PackedByteArray()
    rdata.resize(result_size)
    print("→ result   SSBO capacity:", rdata.size())  # should print 2056
    result_ssbo_rid = ComputeShaderLayer.rendering_device.storage_buffer_create(result_size, rdata)


func _init_hull_uniform_set_optimized() -> void:
    hull_boundary_uniform.binding = 0
    hull_boundary_uniform.add_id(boundary_ssbo_rid)
    hull_meta_uniform = RDUniform.new()
    hull_meta_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
    hull_meta_uniform.binding = 1
    hull_meta_uniform.add_id(meta_ssbo_rid)

    # skip binding 2 (or reuse if you want)
    hull_result_uniform.binding = 2
    hull_result_uniform.add_id(result_ssbo_rid)

    hull_uniform_set_rid = ComputeShaderLayer.rendering_device.uniform_set_create(
        [hull_boundary_uniform, hull_meta_uniform, hull_result_uniform], andrew_shader_rid, 0
    )


func _init_hull_uniform_set() -> void:
    hull_boundary_uniform = RDUniform.new()
    hull_boundary_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
    hull_boundary_uniform.binding = HULL_BOUNDARY_SSBO_BINDING
    hull_boundary_uniform.add_id(boundary_ssbo_rid)

    hull_sorted_uniform = RDUniform.new()
    hull_sorted_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
    hull_sorted_uniform.binding = HULL_SORTED_SSBO_BINDING
    hull_sorted_uniform.add_id(sorted_ssbo_rid)

    hull_result_uniform = RDUniform.new()
    hull_result_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
    hull_result_uniform.binding = HULL_RESULT_SSBO_BINDING
    hull_result_uniform.add_id(result_ssbo_rid)

    hull_uniform_set_rid = ComputeShaderLayer.rendering_device.uniform_set_create(
        [hull_boundary_uniform, hull_sorted_uniform, hull_result_uniform], andrew_shader_rid, 0
    )


func _compute_hull_gpu(center_point_list: PackedVector2Array) -> PackedVector2Array:
    var number_of_points: int = center_point_list.size()
    print("  ▶︎ GPU hull start: N=", number_of_points)

    if number_of_points < ComputeShaderLayer.MIN_VERTICIES_FOR_ANDREW:
        return center_point_list.duplicate()

    var boundary_bytes: PackedByteArray = center_point_list.to_byte_array()
    print("    - uploading ", boundary_bytes.size(), " bytes to boundary SSBO")
    #assert(boundary_bytes.size() <= MAX_HULL_POINTS * 8)
    print(
        "    - uploading ",
        boundary_bytes.size(),
        " bytes to a SSBO of capacity ",
        MAX_HULL_POINTS * 8
    )
    ComputeShaderLayer.rendering_device.buffer_update(
        boundary_ssbo_rid, 0, boundary_bytes.size(), boundary_bytes
    )

    var push_constants_hull: PackedByteArray = PackedByteArray()
    push_constants_hull.resize(16)
    push_constants_hull.encode_u32(0, number_of_points)
    push_constants_hull.encode_u32(4, ComputeShaderLayer.MIN_VERTICIES_FOR_ANDREW)
    push_constants_hull.encode_u32(8, MAX_HULL_POINTS)
    print(
        "    - push_constants: [N=",
        push_constants_hull.decode_u32(0),
        ", minV=",
        push_constants_hull.decode_u32(4),
        ", maxV=",
        push_constants_hull.decode_u32(8),
        "]"
    )

    # dispatch
    print("    - dispatching Andrew-hull compute…")

    var compute_list_int: int = ComputeShaderLayer.rendering_device.compute_list_begin()
    ComputeShaderLayer.rendering_device.compute_list_bind_compute_pipeline(
        compute_list_int, andrew_pipeline_rid
    )
    ComputeShaderLayer.rendering_device.compute_list_bind_uniform_set(
        compute_list_int, hull_uniform_set_rid, 0
    )
    if push_constants_hull.size() > 0:
        ComputeShaderLayer.rendering_device.compute_list_set_push_constant(
            compute_list_int, push_constants_hull, push_constants_hull.size()
        )
    #TODO: WORK GROUPS ARE ALL 1 1 1!!??? idk whats happening, it works but not for big concave shapes still
    ComputeShaderLayer.rendering_device.compute_list_dispatch(compute_list_int, 1, 1, 1)
    ComputeShaderLayer.rendering_device.compute_list_end()
    #ComputeShaderLayer.dispatch_compute(andrew_pipeline_rid, hull_uniform_set_rid, push_constants)
    var count_bytes: PackedByteArray = ComputeShaderLayer.rendering_device.buffer_get_data(
        result_ssbo_rid, 0, 4
    )

    var hull_count: int = count_bytes.decode_u32(0)
    print("    - got hullCount=", hull_count)

    hull_count = min(hull_count, MAX_HULL_POINTS)

    var data_bytes: PackedByteArray = ComputeShaderLayer.rendering_device.buffer_get_data(
        result_ssbo_rid, 8, hull_count * 8
    )
    print(
        "    - downloading ",
        hull_count,
        " points (",
        hull_count * 8,
        " bytes) starting at offset 8"
    )
    var hull: PackedVector2Array = PackedVector2Array()
    hull.resize(hull_count)
    var off: int = 0
    for i: int in range(hull_count):
        var x: float = data_bytes.decode_float(off)
        var y: float = data_bytes.decode_float(off + 4)
        hull[i] = Vector2(x, y)
        if i < 5:
            print("       pt[", i, "] = (", x, ",", y, ")")
        off += 8
    return hull
