extends Node2D
#class_name ComputeShaderLayer

var rd: RenderingDevice
var iResolution: Vector2

const WORKGROUP_TILE_PIXELS_X: int = 2
const WORKGROUP_TILE_PIXELS_Y: int = 2

var rust_util: RustUtil


func _ready() -> void:
    rust_util = RustUtil.new()
    _init_rendering_device()


func _init_rendering_device() -> void:
    rd = RenderingServer.get_rendering_device()
    iResolution = Resolution.resolution


func dispatch_compute(
    compute_pipeline_rid: RID, uniform_set_rid: RID, push_constants: PackedByteArray
) -> void:
    var compute_list_int: int = rd.compute_list_begin()
    rd.compute_list_bind_compute_pipeline(compute_list_int, compute_pipeline_rid)
    rd.compute_list_bind_uniform_set(compute_list_int, uniform_set_rid, 0)
    if push_constants.size() > 0:
        rd.compute_list_set_push_constant(compute_list_int, push_constants, push_constants.size())
    var groups_x: int = int(ceil(iResolution.x / float(WORKGROUP_TILE_PIXELS_X)))
    var groups_y: int = int(ceil(iResolution.y / float(WORKGROUP_TILE_PIXELS_Y)))
    var groups_z: int = 1
    rd.compute_list_dispatch(compute_list_int, groups_x, groups_y, groups_z)
    rd.compute_list_end()


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
    var view_rid: RID = rd.texture_create(sprite_texture_format, view)
    var sprite_texture_image: Image = sprite_texture.get_image()
    rd.texture_update(view_rid, 0, sprite_texture_image.get_data())
    var sprite_texture_rd: Texture2DRD = Texture2DRD.new()
    sprite_texture_rd.set_texture_rd_rid(view_rid)
    return sprite_texture_rd
