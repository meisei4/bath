extends Node2D
class_name Water

var RippleShaderNode: ColorRect
var RippleShader: Shader = preload(ResourcePaths.FINITE_APPROX_RIPPLE)
var RippleShaderMaterial: ShaderMaterial

var WaterShaderNode: ColorRect
var WaterShader: Shader = preload(ResourcePaths.WATER_SHADER)
var WaterShaderMaterial: ShaderMaterial

var BufferA: SubViewport
var BufferB: SubViewport
var MainImage: TextureRect

var iResolution: Vector2

#TODO: get all this non sense out of here and just pass it in from collision shapes or something later
# need to completely rewrite the water shader to be decoupled from ripple entities
var iceberg_previous_positions: PackedVector2Array = PackedVector2Array()
var iceberg_current_positions: PackedVector2Array = PackedVector2Array()
var iceberg_target_positions: PackedVector2Array = PackedVector2Array()
var iceberg_velocities: PackedVector2Array = PackedVector2Array()

var iceberg_tile_positions: PackedVector2Array = PackedVector2Array()
var cluster_offsets: PackedInt32Array = PackedInt32Array()

var interpolation_timer: float = 0.0

var iChannel0: Texture = preload(ResourcePaths.GRAY_NOISE_SMALL_PNG)
var iChannel1: Texture = preload(ResourcePaths.MOON_WATER_PNG)
var iChannel2: Texture = preload(ResourcePaths.PEBBLES_PNG)
var iChannel3: Texture


func _ready() -> void:
    iResolution = ResolutionManager.resolution
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
