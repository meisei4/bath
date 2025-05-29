extends ComputeShaderPipeline
class_name PerspectiveTiltMask

const SPRITE_DATA_SSBO_UNIFORM_BINDING: int = 0
var sprite_data_ssbo_uniform: RDUniform
var sprite_data_ssbo_bytes: PackedByteArray


class SpriteDataSSBOStruct:  # 32 bytes total (std430 aligned)
    var center_px: Vector2  # 8 bytes
    var half_size_px: Vector2  # 8 bytes
    var altitude_normal: float  # 4 bytes
    var ascending: float  # 4 bytes
    var _pad: Vector2  # 8 bytes (keeps each row 16-byte aligned)


const MAXIMUM_SPRITE_COUNT: int = 12
const SPRITE_DATA_STRUCT_SIZE_BYTES: int = 32  # vec2 + vec2 + float + float + vec2_padding
const SPRITE_DATA_SSBO_TOTAL_BYTES: int = MAXIMUM_SPRITE_COUNT * SPRITE_DATA_STRUCT_SIZE_BYTES

var cpu_side_sprite_data_ssbo_cache: Array[SpriteDataSSBOStruct] = []
var sprite_data_ssbo_rid: RID

const SPRITE_TEXTURES_BINDING: int = 1
var sprite_textures_uniform: RDUniform
var sprite_textures_rids: Array[RID] = []
var memory_padding_sprite_textures_rid: RID  # to fill up the unused sprite texture blocks
var resuable_sampler_state: RDSamplerState
var resuable_sampler_state_rid: RID

const PERSPECTIVE_TILT_MASK_UNIFORM_BINDING: int = 2
var perspective_tilt_mask_uniform: RDUniform
var perspective_tilt_mask_texture_format: RDTextureFormat
var perspective_tilt_mask_view: RDTextureView
var perspective_tilt_mask_texture_view_rid: RID
var perspective_tilt_mask_texture: Texture2DRD

var push_constants: PackedByteArray
const PUSH_CONSTANTS_BYTE_BLOCK_SIZE: int = 16
const PUSH_CONSTANTS_BYTE_ALIGNMENT_0: int = 0
const PUSH_CONSTANTS_BYTE_ALIGNMENT_4: int = 4
const PUSH_CONSTANTS_BYTE_ALIGNMENT_8: int = 8
const PUSH_CONSTANTS_BYTE_ALIGNMENT_12: int = 12


func _ready() -> void:
    #ComputeShaderSignalManager.register_perspective_tilt_mask(self)
    _init_shader()
    _init_compute_shader_pipeline()
    _init_sprite_data_ssbo_uniform()
    _init_sprite_textures_uniform()
    _init_perspective_tilt_mask_uniform()
    _init_uniform_set()
    #RenderingServer.frame_pre_draw.connect(_dispatch_compute)


#TODO: MOVE ALL PUBLIC API ENTRY POINTS SOMEWHERE AND BLACK BOX ALL THE COMPUTE PIPELINE STUFF
func register_sprite_texture(sprite_texture: Texture2D) -> int:
    if sprite_textures_rids.size() >= MAXIMUM_SPRITE_COUNT:
        push_error("Too many sprites registered!")
        return -1
    var texture_rd: Texture2DRD = super._sprite_texture2d_to_rd(sprite_texture)
    sprite_textures_rids.append(texture_rd.get_texture_rd_rid())
    var sprite_data_ssbo: SpriteDataSSBOStruct = SpriteDataSSBOStruct.new()
    sprite_data_ssbo.center_px = Vector2.ZERO
    sprite_data_ssbo.half_size_px = Vector2.ONE * 8.0
    sprite_data_ssbo.altitude_normal = 0.0
    sprite_data_ssbo.ascending = 0.0
    cpu_side_sprite_data_ssbo_cache.append(sprite_data_ssbo)
    _update_sprite_textures_uniform()
    var index: int = sprite_textures_rids.size() - 1
    if sprite_textures_rids.size() == 1:
        # first sprite; now it’s safe to connect
        #TODO: just a unique way to make sure the tilt mask is computed before anything else is drawn to the screen...
        RenderingServer.frame_pre_draw.connect(_dispatch_compute)
    return index


func update_cpu_side_sprite_data_ssbo_cache(
    sprite_texture_index: int,
    center_px: Vector2,
    half_size_px: Vector2,
    altitude_normal: float,
    ascending: float
) -> void:
    if sprite_texture_index < 0 or sprite_texture_index >= cpu_side_sprite_data_ssbo_cache.size():
        push_error("Invalid sprite ID")
        return
    cpu_side_sprite_data_ssbo_cache[sprite_texture_index].center_px = center_px
    cpu_side_sprite_data_ssbo_cache[sprite_texture_index].half_size_px = half_size_px
    cpu_side_sprite_data_ssbo_cache[sprite_texture_index].altitude_normal = altitude_normal
    cpu_side_sprite_data_ssbo_cache[sprite_texture_index].ascending = ascending
    _update_sprite_data_ssbo()


func _init_shader() -> void:
    compute_shader_file = load("res://Resources/Shaders/Compute/perspective_tilt_mask.glsl")


func _init_sprite_data_ssbo_uniform() -> void:
    sprite_data_ssbo_bytes = PackedByteArray()
    sprite_data_ssbo_bytes.resize(SPRITE_DATA_SSBO_TOTAL_BYTES)
    sprite_data_ssbo_rid = rendering_device.storage_buffer_create(
        SPRITE_DATA_SSBO_TOTAL_BYTES, PackedByteArray()
    )
    sprite_data_ssbo_uniform = RDUniform.new()
    sprite_data_ssbo_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
    sprite_data_ssbo_uniform.binding = SPRITE_DATA_SSBO_UNIFORM_BINDING
    sprite_data_ssbo_uniform.add_id(sprite_data_ssbo_rid)
    uniform_set.append(sprite_data_ssbo_uniform)
    # ► seed the GPU buffer once with the zero-filled byte-array
    rendering_device.buffer_update(
        sprite_data_ssbo_rid, 0, SPRITE_DATA_SSBO_TOTAL_BYTES, sprite_data_ssbo_bytes
    )


func _init_sprite_textures_uniform() -> void:
    sprite_textures_uniform = RDUniform.new()
    sprite_textures_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_SAMPLER_WITH_TEXTURE
    sprite_textures_uniform.binding = SPRITE_TEXTURES_BINDING
    var img: Image = Image.create(1, 1, false, Image.FORMAT_RGBA8)
    img.fill(Color(0, 0, 0, 0))
    var padding_fmt: RDTextureFormat = RDTextureFormat.new()
    padding_fmt.texture_type = RenderingDevice.TEXTURE_TYPE_2D
    padding_fmt.format = RenderingDevice.DATA_FORMAT_R8G8B8A8_UNORM
    padding_fmt.width = 1
    padding_fmt.height = 1
    padding_fmt.mipmaps = 1
    padding_fmt.usage_bits = (
        RenderingDevice.TEXTURE_USAGE_SAMPLING_BIT | RenderingDevice.TEXTURE_USAGE_CAN_UPDATE_BIT
    )
    var padding_view: RDTextureView = RDTextureView.new()
    memory_padding_sprite_textures_rid = rendering_device.texture_create(padding_fmt, padding_view)
    rendering_device.texture_update(memory_padding_sprite_textures_rid, 0, img.get_data())
    resuable_sampler_state = RDSamplerState.new()
    resuable_sampler_state_rid = rendering_device.sampler_create(resuable_sampler_state)
    for i: int in range(MAXIMUM_SPRITE_COUNT):
        var sprite_textures_rid: RID = (
            sprite_textures_rids[i]
            if i < sprite_textures_rids.size()
            else memory_padding_sprite_textures_rid
        )
        sprite_textures_uniform.add_id(resuable_sampler_state_rid)
        sprite_textures_uniform.add_id(sprite_textures_rid)
    uniform_set.append(sprite_textures_uniform)


func _init_perspective_tilt_mask_uniform() -> void:
    perspective_tilt_mask_texture_format = RDTextureFormat.new()
    perspective_tilt_mask_texture_format.texture_type = RenderingDevice.TEXTURE_TYPE_2D
    perspective_tilt_mask_texture_format.format = RenderingDevice.DATA_FORMAT_R8_UNORM
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
    uniform_set.append(perspective_tilt_mask_uniform)


func _update_sprite_textures_uniform() -> void:
    sprite_textures_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_SAMPLER_WITH_TEXTURE
    sprite_textures_uniform.binding = SPRITE_TEXTURES_BINDING
    sprite_textures_uniform.clear_ids()
    for i: int in range(MAXIMUM_SPRITE_COUNT):
        var sprite_textures_rid: RID = (
            sprite_textures_rids[i]
            if i < sprite_textures_rids.size()
            else memory_padding_sprite_textures_rid
        )
        sprite_textures_uniform.add_id(resuable_sampler_state_rid)
        sprite_textures_uniform.add_id(sprite_textures_rid)
    uniform_set_rid = rendering_device.uniform_set_create(uniform_set, compute_shader_rid, 0)


func _update_sprite_data_ssbo() -> void:
    for i: int in range(cpu_side_sprite_data_ssbo_cache.size()):
        var sprite_data_ssbo: SpriteDataSSBOStruct = cpu_side_sprite_data_ssbo_cache[i]
        var byte_offset: int = i * SPRITE_DATA_STRUCT_SIZE_BYTES
        # write32 assumes little‐endian, same as std430
        sprite_data_ssbo_bytes.encode_float(byte_offset + 0, sprite_data_ssbo.center_px.x)
        sprite_data_ssbo_bytes.encode_float(byte_offset + 4, sprite_data_ssbo.center_px.y)
        sprite_data_ssbo_bytes.encode_float(byte_offset + 8, sprite_data_ssbo.half_size_px.x)
        sprite_data_ssbo_bytes.encode_float(byte_offset + 12, sprite_data_ssbo.half_size_px.y)
        sprite_data_ssbo_bytes.encode_float(byte_offset + 16, sprite_data_ssbo.altitude_normal)
        sprite_data_ssbo_bytes.encode_float(byte_offset + 20, sprite_data_ssbo.ascending)
        # explicit padding (optional—will already be zero)
        sprite_data_ssbo_bytes.encode_float(byte_offset + 24, 0.0)
        sprite_data_ssbo_bytes.encode_float(byte_offset + 28, 0.0)

    rendering_device.buffer_update(
        sprite_data_ssbo_rid, 0, SPRITE_DATA_SSBO_TOTAL_BYTES, sprite_data_ssbo_bytes
    )


func _dispatch_compute() -> void:
    push_constants = PackedByteArray()
    push_constants.resize(PUSH_CONSTANTS_BYTE_BLOCK_SIZE)
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_0, iResolution.x)  # float at bytes 0–3
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_4, iResolution.y)  # float at bytes 4–7
    push_constants.encode_u32(
        PUSH_CONSTANTS_BYTE_ALIGNMENT_8, cpu_side_sprite_data_ssbo_cache.size()
    )
    push_constants.encode_u32(PUSH_CONSTANTS_BYTE_ALIGNMENT_12, 0)  # uint at bytes 12–15
    super.dispatch_compute(push_constants)
