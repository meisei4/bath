extends Node2D
class_name ShaderToy

#TODO: this can be any shader that serves to sample the ITSELF in shadertoy
var BufferAShader: Shader = load("res://Resources/Shaders/Water/finite_approx_ripple.gdshader")
var BufferAShaderMaterial: ShaderMaterial

#TODO: this can be any shader that serves to sample the BufferA in shadertoy
var BufferBShader: Shader = load("res://Resources/Shaders/Water/water_caustics.gdshader")
var BufferBShaderMaterial: ShaderMaterial

var NoiseTexture: Image = Image.load_from_file("res://Assets/Textures/gray_noise_small.png")
var BackgroundTexture: Image = Image.load_from_file("res://Assets/Textures/rocks.jpg")
var CausticsTexture: Image = Image.load_from_file("res://Assets/Textures/pebbles.png")

var BufferA: SubViewport
var BufferB: SubViewport
var MainImage: TextureRect

var iResolution: Vector2
var iMouse: Vector3
var iChannel0: Texture
var iChannel1: Texture
var iChannel2: Texture
var iChannel3: Texture

#TODO: mess around with making this node apply to all shadertoy set ups, where you pass
#in a Dictionary of
#{BufferA:
#       [iChannel0 = buffer_a_texture0, ..., iChannelN = buffer_a_textureN],
# BufferB:
#       [iChannel0 = buffer_b_texture0, ..., iChannelN = buffer_b_textureN],
# ...
# BufferD:
#       [iChannel0 = buffer_d_texture0, ..., iChannelN = buffer_d_textureN],
#}

#func init(BufferAShader: Shader, BufferBShader: Shader) -> void:
#self.BufferAShader = BufferAShader
#self.BufferBShader = BufferBShader


func _ready() -> void:
    initialize_shadertoy_uniforms_and_textures()
    BufferA = create_buffer_viewport(iResolution)
    BufferAShaderMaterial = initialize_shadertoy_style_shader(
        BufferA, Image.FORMAT_RGBAH, BufferAShader, iResolution, []  # buffer_viewport  # buffer_texture_format #TODO: THIS HAS TO BE TRUE FOR THE RIPPLE SHADER TO PROVIDE PROPER PRECISION FOR SAMPLING F16!!!  # shader  # resolution  # channels
    )

    BufferB = create_buffer_viewport(iResolution)
    BufferBShaderMaterial = initialize_shadertoy_style_shader(
        BufferB, Image.FORMAT_RGBA8, BufferBShader, iResolution, [iChannel0, iChannel1, iChannel2]  # buffer_viewport  # buffer_texture_format #TODO: without this the noise texture goes insane, feel like it should be able to be controlled by the ImageTexture channel format...  # shader  # resolution  # channels
    )

    MainImage = TextureRect.new()
    MainImage.texture = BufferB.get_texture()
    MainImage.flip_v = true  # flip because shadertoy has fragment coordinates at origin bottom left (godot is top left)
    add_child(MainImage)


func initialize_shadertoy_uniforms_and_textures() -> void:
    iResolution = get_viewport_rect().size
    iMouse = get_iMouse_uniform()
    NoiseTexture.convert(Image.FORMAT_R8)
    BackgroundTexture.convert(Image.FORMAT_RGBA8)
    CausticsTexture.convert(Image.FORMAT_R8)
    iChannel0 = ImageTexture.create_from_image(NoiseTexture)
    iChannel1 = ImageTexture.create_from_image(BackgroundTexture)
    iChannel2 = ImageTexture.create_from_image(CausticsTexture)


func initialize_shadertoy_style_shader(
    buffer: SubViewport,
    buffer_texture_format: Image.Format,
    shader: Shader,
    resolution: Vector2,
    channels: Array = []
) -> ShaderMaterial:
    match buffer_texture_format:
        Image.FORMAT_RGBAH:
            buffer.use_hdr_2d = true
        Image.FORMAT_RGBA8:
            buffer.use_hdr_2d = false
        _:
            buffer.use_hdr_2d = false  # this is the default anyways, but just for brevity

    var shader_material: ShaderMaterial = ShaderMaterial.new()
    shader_material.shader = shader
    var shader_node: ColorRect = ColorRect.new()
    shader_node.size = resolution
    shader_node.material = shader_material
    shader_material.set_shader_parameter("iResolution", resolution)
    #TODO: i dont like this for loop, it should be able to opick up the iChannels better somehow
    for i: int in range(channels.size()):
        shader_material.set_shader_parameter("iChannel%d" % i, channels[i])

    buffer.add_child(shader_node)
    add_child(buffer)
    return shader_material


func create_buffer_viewport(resolution: Vector2) -> SubViewport:
    var subviewport: SubViewport = SubViewport.new()
    subviewport.size = resolution
    subviewport.disable_3d = true
    RenderingServer.set_default_clear_color(Color.BLACK)
    subviewport.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    subviewport.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    return subviewport


func _process(_delta: float) -> void:
    iMouse = get_iMouse_uniform()
    iChannel3 = BufferA.get_texture() as ViewportTexture
    BufferAShaderMaterial.set_shader_parameter("iMouse", iMouse)
    BufferBShaderMaterial.set_shader_parameter("iChannel3", iChannel3)


func get_iMouse_uniform() -> Vector3:
    var mouse_coords: Vector2 = get_viewport().get_mouse_position()
    var mouse_z: float = 1.0 if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) else 0.0
    iMouse = Vector3(mouse_coords.x, mouse_coords.y, mouse_z)
    return iMouse
