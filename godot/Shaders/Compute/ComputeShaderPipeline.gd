extends Node2D
class_name ComputeShaderPipeline

var rendering_device: RenderingDevice
var iResolution: Vector2

const WORKGROUP_TILE_PIXELS_X: int = 2
const WORKGROUP_TILE_PIXELS_Y: int = 2

var rust_util: RustUtil

var compute_shader_file: RDShaderFile
var compute_shader_spirv: RDShaderSPIRV
var compute_shader_rid: RID
var compute_pipeline_rid: RID

var uniform_set_rid: RID
var uniform_set: Array[RDUniform]


func _init_compute_shader_pipeline() -> void:
    rust_util = RustUtil.new()
    _init_rendering_device()
    #TODO: none of this will work on openGL/compatibility mode, only vulkan
    # in fact: RenderingDevice is not available [...] when using the Compatibility rendering method.
    # https://docs.godotengine.org/en/stable/classes/class_renderingdevice.html#class-renderingdevice
    compute_shader_spirv = compute_shader_file.get_spirv()
    compute_shader_rid = rendering_device.shader_create_from_spirv(compute_shader_spirv)
    compute_pipeline_rid = rendering_device.compute_pipeline_create(compute_shader_rid)


func _init_rendering_device() -> void:
    rendering_device = RenderingServer.get_rendering_device()
    iResolution = ResolutionManager.resolution


func _init_uniform_set() -> void:
    uniform_set_rid = rendering_device.uniform_set_create(uniform_set, compute_shader_rid, 0)


func dispatch_compute(push_constants: PackedByteArray) -> void:
    var compute_list_int: int = rendering_device.compute_list_begin()
    rendering_device.compute_list_bind_compute_pipeline(compute_list_int, compute_pipeline_rid)
    rendering_device.compute_list_bind_uniform_set(compute_list_int, uniform_set_rid, 0)
    if push_constants.size() > 0:
        rendering_device.compute_list_set_push_constant(
            compute_list_int, push_constants, push_constants.size()
        )
    var groups_x: int = ceili(iResolution.x / float(WORKGROUP_TILE_PIXELS_X))
    var groups_y: int = ceili(iResolution.y / float(WORKGROUP_TILE_PIXELS_Y))
    var groups_z: int = 1
    rendering_device.compute_list_dispatch(compute_list_int, groups_x, groups_y, groups_z)
    rendering_device.compute_list_end()


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
