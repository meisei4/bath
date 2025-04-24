extends Node2D

const MAXIMUM_SPRITE_COUNT: int = 2
const LOCAL_SIZE_X: int = 8  # must match compute shader
const LOCAL_SIZE_Y: int = 8

#  Purely for documentation / auto-completion. The fields are *not* used
#  directly in the SSBO pack loop; we still write floats manually so layout
#  is 100 % deterministic (std430).
const SPRITE_DATA_STRUCT_SIZE_BYTES: int = 32  # vec2 + vec2 + float + float + padding
class SpriteData:  # 32 bytes total (std430 aligned)
    var center_px: Vector2  # 8 bytes
    var half_size_px: Vector2  # 8 bytes
    var altitude_normal: float  # 4 bytes
    var ascending: float  # 4 bytes
    var _pad: Vector2  # 8 bytes (keeps each row 16-byte aligned)

# SSBO total allocation: 32 B × 2 = 64 B  -> tiny
const SSBO_TOTAL_BYTES: int = MAXIMUM_SPRITE_COUNT * SPRITE_DATA_STRUCT_SIZE_BYTES

var rendering_device: RenderingDevice
var compute_shader_file: RDShaderFile = load("res://Resources/Shaders/Compute/sprite_animations.glsl")
var compute_shader_spirv: RDShaderSPIRV
var compute_shader_rid: RID
var compute_pipeline_rid: RID
var sprite_data_buffer_rid: RID
var sprite_data_buffer_uniform_set_rid: RID
var perspective_tilt_mask_texture_view_rid: RID

var ssbo_uniform: RDUniform
const SSBO_UNIFORM_BINDING: int = 0
var perspective_tilt_mask_texture_format: RDTextureFormat
var perspective_tilt_mask_view: RDTextureView
var perspective_tilt_mask_texture: Texture2DRD  # exposed as Godot Texture for post-FX
var perspective_tilt_mask_uniform: RDUniform
const PERSPECTIVE_TILT_MASK_UNIFORM_BINDING: int = 1

var push_constants: PackedByteArray
const PUSH_CONSTANTS_BYTE_ALIGNMENT_0: int = 0;
const PUSH_CONSTANTS_BYTE_ALIGNMENT_4: int = 4;
const PUSH_CONSTANTS_BYTE_ALIGNMENT_8: int = 8;
const PUSH_CONSTANTS_BYTE_ALIGNMENT_12: int = 12;

#  We keep the same Dictionary style you started with to minimise refactors,
#  but each entry now stores *all* fields needed by the compute shader.
#
#    { "center_px": Vector2,
#      "half_size_px": Vector2,
#      "altitude_normal": float,
#      "ascending": uint       # 0 or 1 }
#
var sprite_data_buffer_array: Array[Dictionary] = []

var iResolution: Vector2

var sprite_id : int

func _ready() -> void:
    iResolution = get_viewport_rect().size
    _initialize_compute_resources()
    sprite_id = _register_sprite()


func _initialize_compute_resources() -> void:
    rendering_device = RenderingServer.get_rendering_device()
    compute_shader_spirv = compute_shader_file.get_spirv()
    compute_shader_rid = rendering_device.shader_create_from_spirv(compute_shader_spirv)
    sprite_data_buffer_rid = rendering_device.storage_buffer_create(
        SSBO_TOTAL_BYTES, PackedByteArray()
    )
    ssbo_uniform = RDUniform.new()
    ssbo_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
    ssbo_uniform.binding = SSBO_UNIFORM_BINDING
    ssbo_uniform.add_id(sprite_data_buffer_rid)

    perspective_tilt_mask_texture_format = RDTextureFormat.new()
    perspective_tilt_mask_texture_format.texture_type = RenderingDevice.TEXTURE_TYPE_2D
    perspective_tilt_mask_texture_format.format = RenderingDevice.DATA_FORMAT_R32_SFLOAT
    perspective_tilt_mask_texture_format.width = iResolution.x as int
    perspective_tilt_mask_texture_format.height = iResolution.y as int
    perspective_tilt_mask_texture_format.depth = 1
    perspective_tilt_mask_texture_format.array_layers = 1
    perspective_tilt_mask_texture_format.mipmaps = 1
    perspective_tilt_mask_texture_format.usage_bits = (
        RenderingDevice.TEXTURE_USAGE_STORAGE_BIT
        | RenderingDevice.TEXTURE_USAGE_CAN_UPDATE_BIT
        | RenderingDevice.TEXTURE_USAGE_SAMPLING_BIT
    )
    perspective_tilt_mask_view = RDTextureView.new()
    perspective_tilt_mask_texture_view_rid = rendering_device.texture_create(
        perspective_tilt_mask_texture_format, perspective_tilt_mask_view
    )
    perspective_tilt_mask_texture = Texture2DRD.new()
    perspective_tilt_mask_texture.set_texture_rd_rid(perspective_tilt_mask_texture_view_rid)
    perspective_tilt_mask_uniform = RDUniform.new()
    perspective_tilt_mask_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_IMAGE
    perspective_tilt_mask_uniform.binding = PERSPECTIVE_TILT_MASK_UNIFORM_BINDING
    perspective_tilt_mask_uniform.add_id(perspective_tilt_mask_texture_view_rid)

    sprite_data_buffer_uniform_set_rid = rendering_device.uniform_set_create(
        [ssbo_uniform, perspective_tilt_mask_uniform], compute_shader_rid, 0
    )
    compute_pipeline_rid = rendering_device.compute_pipeline_create(compute_shader_rid)

func _register_sprite() -> int:
    var size: int = sprite_data_buffer_array.size()
    if size >= MAXIMUM_SPRITE_COUNT:
        push_error("Too many sprites registered!")
        return -1
    sprite_data_buffer_array.append(
        {
            "center_px": Vector2.ZERO,
            "half_size_px": Vector2.ONE * 8.0,
            "altitude_normal": 0.0,
            "ascending": 0.0
        }  # default 16×16 sprite????????
    )
    return size


func update_sprite_state(
    i: int, center_px: Vector2, half_size_px: Vector2, altitude_normal: float, ascending: float
) -> void:
    if i < 0 or i >= sprite_data_buffer_array.size():
        push_error("Invalid sprite ID")
        return
    sprite_data_buffer_array[i].center_px = center_px
    sprite_data_buffer_array[i].half_size_px = half_size_px
    sprite_data_buffer_array[i].altitude_normal = altitude_normal
    sprite_data_buffer_array[i].ascending = ascending


func _process(delta: float) -> void:
    pass
    #upload_sprite_data_buffer()
    #debug()
    #dispatch_compute()


func debug() -> void:
    var gpu_bytes: PackedByteArray = rendering_device.buffer_get_data(sprite_data_buffer_rid, 0, 32)
    var x  = gpu_bytes.decode_float(0)
    var y  = gpu_bytes.decode_float(4)
    var hn = gpu_bytes.decode_float(16)   # altitude_normal
    var asc = gpu_bytes.decode_float(20)   # ascending (0|1)
    print("GPU row 0 → centre=", Vector2(x,y),
        " altitude_normal=", hn, " ascending=", asc)

func upload_sprite_data_buffer() -> void:
    var floats: PackedFloat32Array = PackedFloat32Array()  # contiguous float stream
    #Dictionary[Key, SpriteData????]
    for sprite_data_buffer: Dictionary in sprite_data_buffer_array:
        floats.append_array(
            [
                sprite_data_buffer.center_px.x,
                sprite_data_buffer.center_px.y,
                sprite_data_buffer.half_size_px.x,
                sprite_data_buffer.half_size_px.y,
                sprite_data_buffer.altitude_normal,
                sprite_data_buffer.ascending,
                0.0, # padding total +4 float
                0.0  # padding total +4 float = 8 floats padding
                # with padding total size is 32 bytes
            ]
        )
    # pad unused rows so the buffer size stays constant
    var remaining_padding: int = MAXIMUM_SPRITE_COUNT - sprite_data_buffer_array.size()
    if remaining_padding > 0:
        floats.resize(floats.size() + remaining_padding * 8)  # 8 floats per struct
    var bytes: PackedByteArray = floats.to_byte_array()
    rendering_device.buffer_update(sprite_data_buffer_rid, 0, bytes.size(), bytes)


func dispatch_compute() -> void:
    var compute_list_int: int = rendering_device.compute_list_begin()
    rendering_device.compute_list_bind_compute_pipeline(compute_list_int, compute_pipeline_rid)
    rendering_device.compute_list_bind_uniform_set(
        compute_list_int, sprite_data_buffer_uniform_set_rid, 0
    )
    push_constants = PackedByteArray()
    push_constants.resize(16)
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_0, iResolution.x)  # float at bytes 0–3
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_4, iResolution.y)  # float at bytes 4–7
    push_constants.encode_u32(PUSH_CONSTANTS_BYTE_ALIGNMENT_8, sprite_data_buffer_array.size()) # uint at bytes 8–11
    push_constants.encode_u32(PUSH_CONSTANTS_BYTE_ALIGNMENT_12, 0) # uint at bytes 12–15
    rendering_device.compute_list_set_push_constant(
        compute_list_int, push_constants, push_constants.size()
    )
    var groups_x: int = int(ceil(iResolution.x / float(LOCAL_SIZE_X)))
    var groups_y: int = int(ceil(iResolution.y / float(LOCAL_SIZE_Y)))
    var groups_z: int = 1
    rendering_device.compute_list_dispatch(compute_list_int, groups_x, groups_y, groups_z)
    rendering_device.compute_list_end()
    #rendering_device.submit()
    # optional but safe: ensure the write is visible to later screen-space passes
    #rendering_device.sync()  # blocks CPU until GPU finished this queue
