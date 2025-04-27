extends Node2D

var iResolution: Vector2
var rendering_device: RenderingDevice
var compute_shader_file: RDShaderFile = load(
    "res://Resources/Shaders/Compute/perspective_tilt_mask.glsl"
)
var compute_shader_spirv: RDShaderSPIRV
var compute_shader_rid: RID
var compute_pipeline_rid: RID

const SPRITE_DATA_SSBO_UNIFORM_BINDING: int = 0
var sprite_data_ssbo_uniform: RDUniform


class SpriteDataSSBOStruct:  # 32 bytes total (std430 aligned)
    var center_px: Vector2  # 8 bytes
    var half_size_px: Vector2  # 8 bytes
    var altitude_normal: float  # 4 bytes
    var ascending: float  # 4 bytes
    var _pad: Vector2  # 8 bytes (keeps each row 16-byte aligned)


const MAXIMUM_SPRITE_COUNT: int = 16
const SPRITE_DATA_STRUCT_SIZE_BYTES: int = 32  # vec2 + vec2 + float + float + vec2_padding
const SPRITE_DATA_SSBO_TOTAL_BYTES: int = MAXIMUM_SPRITE_COUNT * SPRITE_DATA_STRUCT_SIZE_BYTES

var cpu_side_sprite_data_ssbo_cache: Array[SpriteDataSSBOStruct]
var gpu_side_sprite_data_ssbo_rid: RID
var gpu_side_sprite_data_ssbo_uniform_set_rid: RID

const SPRITE_TEXTURES_BINDING: int = 1
var sprite_textures_uniform: RDUniform
var sprite_textures_rids: Array[RID]
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

# What are “work-groups” and “tiles”?
# •   A work-group is the smallest set of threads a GPU lets you
#     synchronise and share on-chip memory with.  Vulkan (and GLSL) call the
#     size of that set local_size.  In the shader we fixed it to
#         layout(local_size_x = 2, local_size_y = 2)  →  2 × 2 × 1 = 4 threads

# •   Because every thread in our kernel processes exactly one output
#     pixel, one work-group therefore covers a   2 × 2   block of pixels.
#     We call that block a tile.

# •   At dispatch time we have to tell the driver *how many* of those tiles
#     we need to blanket the whole render target.  For a screen that is
#     iResolution.x × iResolution.y pixels large the math is simply
#         groups_x = ceil(iResolution.x / WORKGROUP_TILE_PIXELS_X)
#         groups_y = ceil(iResolution.y / WORKGROUP_TILE_PIXELS_Y)

#     so every pixel (even the ragged right/bottom edges) gets exactly one
#     shader invocation.
#
# •   The three numbers we finally hand to
#         compute_list_dispatch(…, groups_x, groups_y, groups_z)
#     are therefore:
#          (#tiles horizontally,  #tiles vertically,  #tiles in depth-Z)
#     and the GPU launches   groups_x × groups_y × groups_z   work-groups,
#     each containing     local_size_x × local_size_y × local_size_z
#     threads.

const WORKGROUP_TILE_PIXELS_X: int = 2  # one work-group covers 2×2 pixels horizontally
const WORKGROUP_TILE_PIXELS_Y: int = 2  # one work-group covers 2×2 pixels vertically


#TODO: MOVE ALL PUBLIC API ENTRY POINTS SOMEWHERE AND BLACK BOX ALL THE COMPUTE PIPELINE STUFF
func register_sprite_texture(sprite_texture: Texture2D) -> int:
    if sprite_textures_rids.size() >= MAXIMUM_SPRITE_COUNT:
        push_error("Too many sprites registered!")
        return -1
    var texture_rd: Texture2DRD = _sprite_texture2d_to_rd(sprite_texture)
    sprite_textures_rids.append(texture_rd.get_texture_rd_rid())
    var sprite_data_ssbo: SpriteDataSSBOStruct = SpriteDataSSBOStruct.new()
    sprite_data_ssbo.center_px = Vector2.ZERO
    sprite_data_ssbo.half_size_px = Vector2.ONE * 8.0
    sprite_data_ssbo.altitude_normal = 0.0
    sprite_data_ssbo.ascending = 0.0
    cpu_side_sprite_data_ssbo_cache.append(sprite_data_ssbo)
    _update_gpu_side_sprite_data_ssbo_uniform_set()  #THIS ONLY EVER GETS CALLED WHEN A NEW SPRITE IS ADDED
    return sprite_textures_rids.size() - 1


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
    _update_gpu_side_sprite_data_ssbo()


func debug() -> void:
    var unserialized_sprite_data_ssbo_bytes: PackedByteArray = rendering_device.buffer_get_data(
        gpu_side_sprite_data_ssbo_rid, 0, 32
    )
    var x: float = unserialized_sprite_data_ssbo_bytes.decode_float(0)
    var y: float = unserialized_sprite_data_ssbo_bytes.decode_float(4)
    var hn: float = unserialized_sprite_data_ssbo_bytes.decode_float(16)  # altitude_normal
    var asc: float = unserialized_sprite_data_ssbo_bytes.decode_float(20)  # ascending (0|1)
    print("GPU row 0 centre=", Vector2(x, y), " altitude_normal=", hn, " ascending=", asc)


func _ready() -> void:
    _init_rendering_device()
    _init_compute_shader_pipeline()
    _init_ssbo()
    _init_sprite_textures_and_sampler()
    _init_perspective_tilt_mask_texture()
    #TODO: just a unique way to make sure the tilt mask is computed before anything else is drawn to the screen...
    RenderingServer.frame_pre_draw.connect(_dispatch_compute)


func _init_rendering_device() -> void:
    iResolution = Resolution.resolution
    rendering_device = RenderingServer.get_rendering_device()


func _init_compute_shader_pipeline() -> void:
    #TODO: none of this will work on openGL/compatibility mode, only vulkan
    # in fact: RenderingDevice is not available [...] when using the Compatibility rendering method.
    # https://docs.godotengine.org/en/stable/classes/class_renderingdevice.html#class-renderingdevice
    compute_shader_spirv = compute_shader_file.get_spirv()
    compute_shader_rid = rendering_device.shader_create_from_spirv(compute_shader_spirv)
    compute_pipeline_rid = rendering_device.compute_pipeline_create(compute_shader_rid)


func _init_ssbo() -> void:
    gpu_side_sprite_data_ssbo_rid = rendering_device.storage_buffer_create(
        SPRITE_DATA_SSBO_TOTAL_BYTES, PackedByteArray()
    )
    sprite_data_ssbo_uniform = RDUniform.new()
    sprite_data_ssbo_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
    sprite_data_ssbo_uniform.binding = SPRITE_DATA_SSBO_UNIFORM_BINDING
    sprite_data_ssbo_uniform.add_id(gpu_side_sprite_data_ssbo_rid)


func _init_sprite_textures_and_sampler() -> void:
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


func _init_perspective_tilt_mask_texture() -> void:
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


#TODO: the difference between this function and _update_gpu_side_sprite_data_ssbo is such a terminology headache...
func _update_gpu_side_sprite_data_ssbo_uniform_set() -> void:
    sprite_textures_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_SAMPLER_WITH_TEXTURE
    sprite_textures_uniform.binding = SPRITE_TEXTURES_BINDING
    #TODO: this might be redundant with the padding thats attempted to be added in _update_gpu_side_sprite_data_ssbo
    for i: int in range(MAXIMUM_SPRITE_COUNT):
        var sprite_textures_rid: RID = (
            sprite_textures_rids[i]
            if i < sprite_textures_rids.size()
            else memory_padding_sprite_textures_rid
        )
        sprite_textures_uniform.add_id(resuable_sampler_state_rid)
        sprite_textures_uniform.add_id(sprite_textures_rid)
    gpu_side_sprite_data_ssbo_uniform_set_rid = (rendering_device.uniform_set_create(
        [sprite_data_ssbo_uniform, sprite_textures_uniform, perspective_tilt_mask_uniform],
        compute_shader_rid,
        0
    ))


func _update_gpu_side_sprite_data_ssbo() -> void:
    var serialized_sprite_data_ssbo: PackedFloat32Array = PackedFloat32Array()
    for sprite_data_ssbo: SpriteDataSSBOStruct in cpu_side_sprite_data_ssbo_cache:
        serialized_sprite_data_ssbo.append_array(
            [
                sprite_data_ssbo.center_px.x,
                sprite_data_ssbo.center_px.y,
                sprite_data_ssbo.half_size_px.x,
                sprite_data_ssbo.half_size_px.y,
                sprite_data_ssbo.altitude_normal,
                sprite_data_ssbo.ascending,
                0.0,  # padding total +4 float
                0.0  # padding total +4 float = 8 floats padding
            ]
        )
    #TODO: because we create a completely new serialized ssbo copy in this function everytime, we have to pad it
    # the other way would be to have a serialized ssbo copy that gets padded in the _init_ssbo function and then whenever we
    # want to pass new bytes to the gpu, we just update the first N entries of how many sprites exist
    # The only reason for this is because i like to keep the Struct for readability, in reality we could just
    # only ever maintain the SSBO as a structured PackedFloat32Array but thats confusing for me
    var remaining_padding: int = MAXIMUM_SPRITE_COUNT - cpu_side_sprite_data_ssbo_cache.size()
    if remaining_padding > 0:
        serialized_sprite_data_ssbo.resize(
            serialized_sprite_data_ssbo.size() + remaining_padding * 8
        )
    var serialized_sprite_data_ssbo_bytes: PackedByteArray = (
        serialized_sprite_data_ssbo.to_byte_array()
    )
    rendering_device.buffer_update(
        gpu_side_sprite_data_ssbo_rid,
        0,
        serialized_sprite_data_ssbo_bytes.size(),
        serialized_sprite_data_ssbo_bytes
    )


func _dispatch_compute() -> void:
    var compute_list_int: int = rendering_device.compute_list_begin()
    rendering_device.compute_list_bind_compute_pipeline(compute_list_int, compute_pipeline_rid)
    #TODO: this next line fails if you dont register a sprite ever... bad bad bad
    rendering_device.compute_list_bind_uniform_set(
        compute_list_int, gpu_side_sprite_data_ssbo_uniform_set_rid, 0
    )
    push_constants = PackedByteArray()
    push_constants.resize(PUSH_CONSTANTS_BYTE_BLOCK_SIZE)
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_0, iResolution.x)  # float at bytes 0–3
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_4, iResolution.y)  # float at bytes 4–7
    push_constants.encode_u32(
        PUSH_CONSTANTS_BYTE_ALIGNMENT_8, cpu_side_sprite_data_ssbo_cache.size()
    )
    push_constants.encode_u32(PUSH_CONSTANTS_BYTE_ALIGNMENT_12, 0)  # uint at bytes 12–15
    rendering_device.compute_list_set_push_constant(
        compute_list_int, push_constants, push_constants.size()
    )
    var groups_x: int = int(ceil(iResolution.x / float(WORKGROUP_TILE_PIXELS_X)))
    var groups_y: int = int(ceil(iResolution.y / float(WORKGROUP_TILE_PIXELS_Y)))
    var groups_z: int = 1
    rendering_device.compute_list_dispatch(compute_list_int, groups_x, groups_y, groups_z)
    rendering_device.compute_list_end()
    #TODO: this is not allowed when targetting main/global rendering device...
    # but cpu side textures can not share RID'S with a local rendering device...
    #TODO: option for later is to look at adding a heavy weight full cpu texture copying
    #to a local rendering device to allow for more control but risking bloat and stuff
    #rendering_device.submit()
    #rendering_device.sync()  # blocks CPU until GPU finished this queue


#TODO: stupid util function...
func _sprite_texture2d_to_rd(sprite_texture: Texture2D) -> Texture2DRD:
    var sprite_texture_format: RDTextureFormat = RDTextureFormat.new()
    sprite_texture_format.texture_type = RenderingDevice.TEXTURE_TYPE_2D
    sprite_texture_format.format = RenderingDevice.DATA_FORMAT_R8G8B8A8_UNORM
    sprite_texture_format.width = sprite_texture.get_width()
    sprite_texture_format.height = sprite_texture.get_height()
    sprite_texture_format.mipmaps = 1
    sprite_texture_format.usage_bits = (
        RenderingDevice.TEXTURE_USAGE_SAMPLING_BIT | RenderingDevice.TEXTURE_USAGE_CAN_UPDATE_BIT
    )
    var view: RDTextureView = RDTextureView.new()
    var view_rid: RID = rendering_device.texture_create(sprite_texture_format, view)
    var sprite_texture_image: Image = sprite_texture.get_image()
    rendering_device.texture_update(view_rid, 0, sprite_texture_image.get_data())
    var sprite_texture_rd: Texture2DRD = Texture2DRD.new()
    sprite_texture_rd.set_texture_rd_rid(view_rid)
    return sprite_texture_rd
