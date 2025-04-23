extends Node2D

# ─────────────────────────────────────────────────────────────
#                    CONSTANT DEFINITIONS
# ─────────────────────────────────────────────────────────────
const MAXIMUM_SPRITE_COUNT: int = 64
const SPRITE_STRUCT_SIZE_BYTES: int = 32  # vec2 + vec2 + float + float + padding
const LOCAL_SIZE_X: int = 8  # must match compute shader
const LOCAL_SIZE_Y: int = 8

# SSBO total allocation: 32 B × 64 = 2 KiB  → tiny
const SSBO_TOTAL_BYTES: int = MAXIMUM_SPRITE_COUNT * SPRITE_STRUCT_SIZE_BYTES

# ─────────────────────────────────────────────────────────────
#                       GPU OBJECT HANDLES
# ─────────────────────────────────────────────────────────────
var rendering_device: RenderingDevice
var compute_shader_id: RID
var compute_pipeline_id: RID
var sprite_data_buffer_id: RID
var sprite_data_uniform_set_id: RID
var warp_mask_texture: Texture2D  # exposed as Godot Texture for post-FX

# ─────────────────────────────────────────────────────────────
#                   RUNTIME SPRITE STATE TABLE
# ─────────────────────────────────────────────────────────────
#
#  We keep the same Dictionary style you started with to minimise refactors,
#  but each entry now stores *all* fields needed by the compute shader.
#
#    { "center": Vector2,
#      "half_size": Vector2,
#      "altitude": float,
#      "ascending": float       # 0 or 1 }
#
var sprite_information_list: Array[Dictionary] = []


# ─────────────────────────────────────────────────────────────
#                      STRUCT  REFERENCE
# ─────────────────────────────────────────────────────────────
#
#  Purely for documentation / auto-completion. The fields are *not* used
#  directly in the SSBO pack loop; we still write floats manually so layout
#  is 100 % deterministic (std430).
#
class SpriteData:  # 32 bytes total (std430 aligned)
    var center_px: Vector2  # 8 bytes
    var half_size_px: Vector2  # 8 bytes
    var altitude: float  # 4 bytes
    var ascending: float  # 4 bytes
    var _pad: Vector2  # 8 bytes (keeps each row 16-byte aligned)


# ─────────────────────────────────────────────────────────────
#                              SET-UP
# ─────────────────────────────────────────────────────────────
var iResolution: Vector2

var shader_file: RDShaderFile = load("res://Resources/Shaders/Compute/sprite_animations.glsl")


func _ready() -> void:
    iResolution = get_viewport_rect().size
    _initialize_compute_resources()


func _initialize_compute_resources() -> void:
    # 1) local RenderingDevice (isolated queue)
    rendering_device = RenderingServer.create_local_rendering_device()
    # 2) compile / register compute shader
    var shader_spirv: RDShaderSPIRV = shader_file.get_spirv()
    compute_shader_id = rendering_device.shader_create_from_spirv(shader_spirv)

    # 3) allocate the SSBO up-front (filled with zeroes for now)
    sprite_data_buffer_id = rendering_device.storage_buffer_create(
        SSBO_TOTAL_BYTES, PackedByteArray()
    )

    # 4) create the R16F escape-mask texture the compute shader will write
    #
    #    We must build:
    #      • RDTextureFormat  – describes the GPU image itself
    #      • RDTextureView    – how shaders will see that image
    #    Then pass both to   rendering_device.texture_create().
    #
    var fmt: RDTextureFormat = RDTextureFormat.new()
    fmt.texture_type = RenderingDevice.TEXTURE_TYPE_2D
    fmt.format = RenderingDevice.DATA_FORMAT_R32_SFLOAT
    fmt.width = iResolution.x as int
    fmt.height = iResolution.y as int
    fmt.depth = 1
    fmt.array_layers = 1
    fmt.mipmaps = 1
    fmt.usage_bits = (
        RenderingDevice.TEXTURE_USAGE_CAN_UPDATE_BIT | RenderingDevice.TEXTURE_USAGE_STORAGE_BIT
    )

    var view: RDTextureView = RDTextureView.new()

    var mask_rid: RID = rendering_device.texture_create(fmt, view)

    var ssbo_uniform := RDUniform.new()
    ssbo_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
    ssbo_uniform.binding = 0
    ssbo_uniform.add_id(sprite_data_buffer_id)

    var image_uniform: RDUniform = RDUniform.new()
    image_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_IMAGE
    image_uniform.binding = 1
    image_uniform.add_id(mask_rid)

    sprite_data_uniform_set_id = rendering_device.uniform_set_create(
        [ssbo_uniform, image_uniform], compute_shader_id, 0
    )
    compute_pipeline_id = rendering_device.compute_pipeline_create(compute_shader_id)


# ─────────────────────────────────────────────────────────────
#                        PER-FRAME LOOP
# ─────────────────────────────────────────────────────────────
func _process(_delta: float) -> void:
    pass
    #_upload_sprite_buffer()
    #_dispatch_compute()


# ─────────────────────────────────────────────────────────────
#                 1 )  UPLOAD SSBO  (buffer_update)
# ─────────────────────────────────────────────────────────────
func _upload_sprite_buffer() -> void:
    var floats := PackedFloat32Array()  # contiguous float stream

    # pack each sprite row (std430 layout; keep field order!)
    for info in sprite_information_list:
        floats.append_array(
            [
                info.center.x,
                info.center.y,
                info.half_size.x,
                info.half_size.y,
                info.altitude,
                info.ascending,
                0.0,
                0.0  # explicit padding to 32 bytes
            ]
        )

    # pad unused rows so the buffer size stays constant
    var missing := MAXIMUM_SPRITE_COUNT - sprite_information_list.size()
    if missing > 0:
        floats.resize(floats.size() + missing * 8)  # 8 floats per struct

    # copy into GPU memory *without* reallocating the buffer
    var bytes: PackedByteArray = floats.to_byte_array()
    rendering_device.buffer_update(sprite_data_buffer_id, 0, bytes.size(), bytes)


# ─────────────────────────────────────────────────────────────
#                 2 )  DISPATCH COMPUTE
# ─────────────────────────────────────────────────────────────
func _dispatch_compute() -> void:
    var groups_x := int(ceil(iResolution.x / float(LOCAL_SIZE_X)))
    var groups_y := int(ceil(iResolution.y / float(LOCAL_SIZE_Y)))

    var cl := rendering_device.compute_list_begin()
    rendering_device.compute_list_bind_compute_pipeline(cl, compute_pipeline_id)
    rendering_device.compute_list_bind_uniform_set(cl, sprite_data_uniform_set_id, 0)
    rendering_device.compute_list_dispatch(cl, groups_x, groups_y, 1)
    rendering_device.compute_list_end()
    rendering_device.submit()
    # optional but safe: ensure the write is visible to later screen-space passes
    rendering_device.sync()  # blocks CPU until GPU finished this queue


# ─────────────────────────────────────────────────────────────
#                    3 )  CPU HELPERS
# ─────────────────────────────────────────────────────────────
func register_sprite() -> int:
    var id := sprite_information_list.size()
    if id >= MAXIMUM_SPRITE_COUNT:
        push_error("Too many sprites registered!")
        return -1
    sprite_information_list.append(
        {"center": Vector2.ZERO, "half_size": Vector2.ONE * 8.0, "altitude": 0.0, "ascending": 0.0}  # default 16×16 sprite
    )
    return id


func update_sprite_state(
    id: int, center_px: Vector2, size_px: Vector2, altitude_normal: float, is_ascending: bool
) -> void:
    if id < 0 or id >= sprite_information_list.size():
        push_error("Invalid sprite ID")
        return

    sprite_information_list[id].center = center_px
    sprite_information_list[id].half_size = size_px * 0.5
    sprite_information_list[id].altitude = altitude_normal
    sprite_information_list[id].ascending = 1.0 if is_ascending else 0.0
