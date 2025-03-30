extends Node2D
class_name ShaderToy

var WaterShaderNode: ColorRect
var WaterShader: Shader = load("res://Resources/Shaders/water.gdshader")
var WaterShaderMaterial: ShaderMaterial

var RippleShaderNode: ColorRect
var RippleShader: Shader = load("res://Resources/Shaders/finite_approx_ripple.gdshader")
var RippleShaderMaterial: ShaderMaterial

var NoiseTexture: ImageTexture
var NoiseImage: Image = Image.load_from_file("res://Assets/Textures/gray_noise_small.png")
var BackgroundTexture: ImageTexture
var BackgroundImage: Image = Image.load_from_file("res://Assets/Textures/rocks.jpg")
var CausticsTexture: ImageTexture
var CausticsImage: Image = Image.load_from_file("res://Assets/Textures/pebbles.png")

var iResolution: Vector2
var iMouse: Vector3
var BufferA: SubViewport
var BufferB: SubViewport
var FinalImage: TextureRect


func _ready() -> void:
    NoiseImage.convert(Image.FORMAT_R8)
    NoiseTexture = ImageTexture.create_from_image(NoiseImage)
    BackgroundImage.convert(Image.FORMAT_RGBA8)
    BackgroundTexture = ImageTexture.create_from_image(BackgroundImage)
    CausticsImage.convert(Image.FORMAT_R8)
    CausticsTexture = ImageTexture.create_from_image(CausticsImage)

    iResolution = get_viewport_rect().size
    BufferA = create_viewport(iResolution)
    #TODO: this was the fix! it allows for the texture format for the subviewport sampling to go from R10G10B10A2_UNORM (10 bit precision unsigned normalized) to 16 bit FLOATS!
    BufferA.use_hdr_2d = true
    RippleShaderMaterial = ShaderMaterial.new()
    RippleShaderNode = ColorRect.new()
    RippleShaderNode.size = iResolution
    RippleShaderMaterial.shader = RippleShader
    RippleShaderNode.material = RippleShaderMaterial
    RippleShaderMaterial.set_shader_parameter("iResolution", iResolution)
    BufferA.add_child(RippleShaderNode)
    add_child(BufferA)

    BufferB = create_viewport(iResolution)
    BufferB.use_hdr_2d = false  #TODO: without this the noise texture goes insane, feel like it should be able to be controlled by the ImageTexture channel format...
    FinalImage = TextureRect.new()
    FinalImage.texture = BufferB.get_texture()
    FinalImage.flip_v = true  # flip because shadertoy has fragment coordinates at origin bottom left (godot is top left)
    WaterShaderMaterial = ShaderMaterial.new()
    WaterShaderNode = ColorRect.new()
    WaterShaderNode.size = iResolution
    WaterShaderMaterial.shader = WaterShader
    WaterShaderNode.material = WaterShaderMaterial
    WaterShaderMaterial.set_shader_parameter("iResolution", iResolution)
    WaterShaderMaterial.set_shader_parameter("iChannel0", NoiseTexture)
    WaterShaderMaterial.set_shader_parameter("iChannel1", BackgroundTexture)
    WaterShaderMaterial.set_shader_parameter("iChannel2", CausticsTexture)
    BufferB.add_child(WaterShaderNode)
    add_child(BufferB)
    add_child(FinalImage)


func create_viewport(size: Vector2) -> SubViewport:
    var subviewport: SubViewport = SubViewport.new()
    subviewport.size = size
    subviewport.disable_3d = true
    RenderingServer.set_default_clear_color(Color.BLACK)
    subviewport.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    subviewport.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    return subviewport


func _process(delta: float) -> void:
    var mouse_xy: Vector2 = get_viewport().get_mouse_position()
    var mouse_z: float = 1.0 if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) else 0.0
    iMouse = Vector3(mouse_xy.x, mouse_xy.y, mouse_z)
    RippleShaderMaterial.set_shader_parameter("iMouse", iMouse)
    WaterShaderMaterial.set_shader_parameter("iChannel3", BufferA.get_texture())
