extends Node2D
#class_name GpuOffloadCollisionMask

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


func _init_andrew_hull_compute_pipeline() -> void:
    andrew_shader_spirv = andrew_shader_file.get_spirv()
    andrew_shader_rid = ComputeShaderLayer.rendering_device.shader_create_from_spirv(
        andrew_shader_spirv
    )
    andrew_pipeline_rid = ComputeShaderLayer.rendering_device.compute_pipeline_create(
        andrew_shader_rid
    )


func _init_andrew_hull_ssbos() -> void:
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


func _init_andrew_hull_uniform_set() -> void:
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


func _compute_andrew_hull_gpu(center_point_list: PackedVector2Array) -> PackedVector2Array:
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
