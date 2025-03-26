extends Node2D
class_name DEAD

const RESOLUTION: Vector2i = Vector2i(512, 768)

var current_height: Image
var previous_height: Image
var heightmap_texture: ImageTexture

var time_accum: float = 0.0
var mouse_state: Vector3 = Vector3.ZERO

var display_rect: ColorRect

@export var background_texture: Texture
@export var noise_texture: Texture
@export var pebbles_texture: Texture

const BASE_SAMPLE_STEP: float = 0.005
const SPEED_FACTOR: float = 1.0
const RIPPLE_SCALE: float = 0.25
const PROPAGATION_INTENSITY: float = 1.0
const IMPULSE_WAVE_WIDTH: float = 0.025
const BASE_IMPULSE_STRENGTH: float = -0.15
const BASE_PROPAGATION: float = 1.0
const BASE_DAMPENING: float = 0.99

const EFFECTIVE_SAMPLE_STEP: float = BASE_SAMPLE_STEP * SPEED_FACTOR
const EFFECTIVE_RIPPLE_SCALE: float = RIPPLE_SCALE / sqrt(SPEED_FACTOR)
const IMPULSE_INNER_RADIUS: float = 0.025 * EFFECTIVE_RIPPLE_SCALE
const IMPULSE_OUTER_RADIUS: float = IMPULSE_INNER_RADIUS + IMPULSE_WAVE_WIDTH * EFFECTIVE_RIPPLE_SCALE
const EFFECTIVE_PROPAGATION: float = BASE_PROPAGATION + 0.15 * PROPAGATION_INTENSITY
const EFFECTIVE_DAMPENING: float = BASE_DAMPENING - 0.15 * PROPAGATION_INTENSITY

func _ready() -> void:
    background_texture = load("res://Assets/Textures/rocks.jpg")
    noise_texture = load("res://Assets/Textures/gray_noise_small.png")
    pebbles_texture = load("res://Assets/Textures/pebbles.png")
    current_height = Image.create(RESOLUTION.x, RESOLUTION.y, false, Image.FORMAT_RGBAF)
    previous_height = Image.create(RESOLUTION.x, RESOLUTION.y, false, Image.FORMAT_RGBAF)
    current_height.fill(Color(0.0, 0.0, 0.0, 0.0))
    previous_height.fill(Color(0.0, 0.0, 0.0, 0.0))

    heightmap_texture = ImageTexture.create_from_image(current_height)

    var shader_mat: ShaderMaterial = ShaderMaterial.new()
    shader_mat.shader = preload("res://Resources/Shaders/testripple.gdshader")

    display_rect = ColorRect.new()
    display_rect.size = get_viewport().size
    display_rect.material = shader_mat

    shader_mat.set_shader_parameter("iResolution", RESOLUTION)

    shader_mat.set_shader_parameter("iChannel2", background_texture)
    shader_mat.set_shader_parameter("iChannel3", noise_texture)
    shader_mat.set_shader_parameter("iChannel4", pebbles_texture)

    add_child(display_rect)

func _process(delta: float) -> void:
    time_accum += delta

    var mouse_pos: Vector2 = get_viewport().get_mouse_position()
    var is_pressed: bool = Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
    mouse_state = Vector3(mouse_pos.x, mouse_pos.y, 1.0 if is_pressed else 0.0)

    simulate_ripple()

    heightmap_texture.set_image(current_height)

    var shader_mat: ShaderMaterial = display_rect.material as ShaderMaterial
    shader_mat.set_shader_parameter("iTime", time_accum)
    shader_mat.set_shader_parameter("iMouse", mouse_state)
    shader_mat.set_shader_parameter("iChannel0", heightmap_texture)

func simulate_ripple() -> void:
    var next_height: Image = Image.create(RESOLUTION.x, RESOLUTION.y, false, Image.FORMAT_RGBAF)
    var mouse_uv: Vector2 = Vector2i(mouse_state.x, mouse_state.y) / RESOLUTION

    for_each_simulation_pixel(func(x: int, y: int) -> void:
        var uv: Vector2 = Vector2i(x, y) / RESOLUTION

        var left: float = current_height.get_pixel(x - 1, y).r
        var right: float = current_height.get_pixel(x + 1, y).r
        var top: float = current_height.get_pixel(x, y + 1).r
        var bottom: float = current_height.get_pixel(x, y - 1).r

        var average_neighbor_height: float = (left + right + top + bottom) / 4.0

        var current_value: Color = current_height.get_pixel(x, y)
        var previous_value: Color = previous_height.get_pixel(x, y)

        var previous_height_value: float = current_value.r
        var previous_previous_height_value: float = previous_value.g

        var propagated: float = previous_height_value + EFFECTIVE_PROPAGATION * (average_neighbor_height - previous_previous_height_value)
        propagated *= EFFECTIVE_DAMPENING

        var impulse: float = 0.0
        if mouse_state.z > 0.0:
            var distance_from_mouse: float = (uv - mouse_uv).length()
            impulse = BASE_IMPULSE_STRENGTH * smoothstep(IMPULSE_OUTER_RADIUS, IMPULSE_INNER_RADIUS, distance_from_mouse)

        var final_height: float = propagated + impulse
        next_height.set_pixel(x, y, Color(final_height, previous_height_value, mouse_uv.x, mouse_uv.y))
    )

    previous_height = current_height
    current_height = next_height

func for_each_simulation_pixel(action: Callable) -> void:
    var width: int = RESOLUTION.x
    var height: int = RESOLUTION.y

    for y_index: int in range(1, height - 1):
        for x_index: int in range(1, width - 1):
            action.call(x_index, y_index)
