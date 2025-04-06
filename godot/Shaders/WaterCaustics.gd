extends Node2D
class_name WaterCaustics

var RippleShaderNode: ColorRect
var RippleShader: Shader = load("res://Resources/Shaders/Water/finite_approx_ripple.gdshader")
var RippleShaderMaterial: ShaderMaterial

var WaterShaderNode: ColorRect
var WaterShader: Shader = load("res://Resources/Shaders/Water/water_caustics.gdshader")
var WaterShaderMaterial: ShaderMaterial

var NoiseTexture: Image = Image.load_from_file("res://Assets/Textures/gray_noise_small.png")
var BackgroundTexture: Image = Image.load_from_file("res://Assets/Textures/moon_water.png")
var CausticsTexture: Image = Image.load_from_file("res://Assets/Textures/pebbles.png")

var BufferA: SubViewport
var BufferB: SubViewport
var MainImage: TextureRect

var iResolution: Vector2

var iceberg_previous_positions: PackedVector2Array = PackedVector2Array()
var iceberg_current_positions: PackedVector2Array = PackedVector2Array()
var iceberg_target_positions: PackedVector2Array = PackedVector2Array()
var iceberg_velocities: PackedVector2Array = PackedVector2Array()

var iceberg_tile_positions: PackedVector2Array = PackedVector2Array()
var cluster_offsets: PackedInt32Array = PackedInt32Array()

var interpolation_timer: float = 0.0

var iChannel0: Texture
var iChannel1: Texture
var iChannel2: Texture
var iChannel3: Texture


func _ready() -> void:
    initialize_shadertoy_uniforms_and_textures()
    BufferA = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferA.use_hdr_2d = true
    RippleShaderMaterial = ShaderMaterial.new()
    RippleShaderNode = ColorRect.new()
    RippleShaderNode.size = iResolution
    RippleShaderMaterial.shader = RippleShader
    RippleShaderNode.material = RippleShaderMaterial
    RippleShaderMaterial.set_shader_parameter("iResolution", iResolution)
    RippleShaderMaterial.set_shader_parameter("tile_size", GlacierConstants.TILE_SIZE_1D)

    BufferB = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferB.use_hdr_2d = false
    WaterShaderMaterial = ShaderMaterial.new()
    WaterShaderNode = ColorRect.new()
    WaterShaderNode.size = iResolution
    WaterShaderMaterial.shader = WaterShader
    WaterShaderNode.material = WaterShaderMaterial
    WaterShaderMaterial.set_shader_parameter("iResolution", iResolution)
    WaterShaderMaterial.set_shader_parameter("iChannel0", iChannel0)
    WaterShaderMaterial.set_shader_parameter("iChannel1", iChannel1)
    WaterShaderMaterial.set_shader_parameter("iChannel2", iChannel2)

    MainImage = TextureRect.new()
    MainImage.texture = BufferB.get_texture()
    MainImage.flip_v = true
    BufferA.add_child(RippleShaderNode)
    add_child(BufferA)
    BufferB.add_child(WaterShaderNode)
    add_child(BufferB)
    add_child(MainImage)


func initialize_shadertoy_uniforms_and_textures() -> void:
    iResolution = get_viewport_rect().size
    #NoiseTexture.convert(Image.FORMAT_R8)
    #BackgroundTexture.convert(Image.FORMAT_RGBA8)
    #CausticsTexture.convert(Image.FORMAT_R8)
    iChannel0 = ImageTexture.create_from_image(NoiseTexture)
    iChannel1 = ImageTexture.create_from_image(BackgroundTexture)
    iChannel2 = ImageTexture.create_from_image(CausticsTexture)


func _process(delta: float) -> void:
    interpolation_timer = min(
        interpolation_timer + delta, GlacierConstants.SIMULATION_TICK_INTERVAL
    )
    var t: float = interpolation_timer / GlacierConstants.SIMULATION_TICK_INTERVAL
    for i: int in range(iceberg_previous_positions.size()):
        var previous_position: Vector2 = iceberg_previous_positions[i]
        var target_position: Vector2 = iceberg_target_positions[i]
        var current_position: Vector2 = previous_position.lerp(target_position, t)
        iceberg_current_positions.set(i, current_position)
    RippleShaderMaterial.set_shader_parameter("iceberg_positions", iceberg_current_positions)
    RippleShaderMaterial.set_shader_parameter("iceberg_velocities", iceberg_velocities)
    RippleShaderMaterial.set_shader_parameter("iceberg_tile_positions", iceberg_tile_positions)
    RippleShaderMaterial.set_shader_parameter("cluster_offsets", cluster_offsets)

    iChannel3 = BufferA.get_texture() as ViewportTexture
    WaterShaderMaterial.set_shader_parameter("iChannel3", iChannel3)


func update_iceberg_clusters_anchor_position_from_discrete_tile_space_to_continious_interpolated_screen_space(
    cluster_id: int, iceberg_cluster_anchor_in_tile_coordinates: Vector2i
) -> void:
    var iceberg_cluster_anchor_screen_coordinates: Vector2 = (
        iceberg_cluster_anchor_in_tile_coordinates * GlacierConstants.TILE_SIZE_1D
    )

    #TODO: fix all this initialization garbagio, its ugly as hell
    if cluster_id >= iceberg_target_positions.size():
        while iceberg_target_positions.size() <= cluster_id:
            iceberg_previous_positions.append(Vector2.ZERO)
            iceberg_current_positions.append(Vector2.ZERO)
            iceberg_target_positions.append(Vector2.ZERO)
            iceberg_velocities.append(Vector2.ZERO)
    if iceberg_current_positions[cluster_id] == Vector2.ZERO:
        iceberg_previous_positions[cluster_id] = iceberg_cluster_anchor_screen_coordinates
        iceberg_current_positions[cluster_id] = iceberg_cluster_anchor_screen_coordinates
        iceberg_target_positions[cluster_id] = iceberg_cluster_anchor_screen_coordinates
        iceberg_velocities[cluster_id] = Vector2.ZERO
    else:
        iceberg_previous_positions[cluster_id] = iceberg_current_positions[cluster_id]
        iceberg_target_positions[cluster_id] = iceberg_cluster_anchor_screen_coordinates
        var new_velocity: Vector2 = (
            (iceberg_cluster_anchor_screen_coordinates - iceberg_previous_positions[cluster_id])
            / GlacierConstants.SIMULATION_TICK_INTERVAL
        )
        iceberg_velocities[cluster_id] = new_velocity

    interpolation_timer = 0.0
