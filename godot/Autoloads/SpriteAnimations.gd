extends Node2D

const MAXIMUM_SPRITE_COUNT: int = 64

var rendering_device: RenderingDevice
var compute_shader_id: RID
var compute_pipeline_id: RID
var sprite_data_buffer_id: RID
var sprite_data_uniform_set_id: RID
var warp_mask_texture: Texture2D

# Runtime list of sprite entries, each is a Dictionary { "position": Vector2, "altitude": float }
var sprite_information_list: Array[Dictionary] = []
# Each entry is a Dictionary { "position": Vector2, "altitude": float }
var sprite_state_list: Array[Dictionary] = []

var iResolution: Vector2


func _ready() -> void:
    iResolution = get_viewport_rect().size
    _initialize_compute_resources()


func _initialize_compute_resources() -> void:
    # 1) Create our own RenderingDevice
    rendering_device = RenderingServer.create_local_rendering_device()

    var shader_file := load("res://Resources/Shaders/Compute/sprite_animations.glsl")
    var shader_spirv: RDShaderSPIRV = shader_file.get_spirv()
    compute_shader_id = rendering_device.shader_create_from_spirv(shader_spirv)

    # 5) Bind that buffer into a uniform set (binding 0)
    var storage_buffer_uniform: RDUniform = RDUniform.new()
    storage_buffer_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
    storage_buffer_uniform.binding = 0
    storage_buffer_uniform.add_id(sprite_data_buffer_id)

    sprite_data_uniform_set_id = rendering_device.uniform_set_create(
        [storage_buffer_uniform], compute_shader_id, 0
    )
    compute_pipeline_id = rendering_device.compute_pipeline_create(compute_shader_id)


func _process(_delta):
    _upload_sprite_buffer()
    _dispatch_compute()


func _upload_sprite_buffer():
    # Prepare our data. We use floats in the shader, so we need 32 bit.
    var input: PackedFloat32Array = PackedFloat32Array()
    for info in sprite_information_list:
        input.append_array([info.pos.x, info.pos.y, info.alt, 0.0])
    # pad out to MAXIMUM_SPRITE_COUNT
    input.resize(MAXIMUM_SPRITE_COUNT * 4)
    var input_bytes: PackedByteArray = input.to_byte_array()
    # push to GPU
    sprite_data_buffer_id = rendering_device.storage_buffer_create(input_bytes.size(), input_bytes)


func _dispatch_compute():
    var compute_list_int: int = rendering_device.compute_list_begin()
    rendering_device.compute_list_bind_compute_pipeline(compute_list_int, compute_pipeline_id)
    rendering_device.compute_list_bind_uniform_set(compute_list_int, sprite_data_uniform_set_id, 0)
    # Dispatch 2×1×1 workgroups as your test
    rendering_device.compute_list_dispatch(compute_list_int, 2, 1, 1)
    rendering_device.compute_list_end()
    rendering_device.submit()


func register_sprite() -> int:
    var id = sprite_information_list.size()
    if id >= MAXIMUM_SPRITE_COUNT:
        push_error("Too many sprites registered!")
        return -1
    sprite_information_list.append({"id": id, "pos": Vector2(), "alt": 0.0})
    return id


func update_sprite_state(id: int, world_pos: Vector2, altitude_normal: float) -> void:
    if id < 0 or id >= sprite_information_list.size():
        push_error("Invalid sprite ID")
        return
    sprite_information_list[id].pos = world_pos
    sprite_information_list[id].alt = altitude_normal
