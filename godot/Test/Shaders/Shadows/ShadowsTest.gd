#extends WorldEnvironment
extends Node2D
class_name ShadowsTest

var UmbralShader: Shader = load("res://Resources/Shaders/Shadows/umbral_zone.gdshader")
var UmbralShaderNode: ColorRect
var UmbralShaderMaterial: ShaderMaterial
var UmbralBackBuffer: BackBufferCopy
const UMBRAL_ZONE_BOUNDS_UV_X: float = 0.5
const UMBRAL_ZONE_BOUNDS_UV_Y: float = 1.0

var DitherShader: Shader = load("res://Resources/Shaders/Shadows/dither_zone.gdshader")
var BayerTexture: Image = Image.load_from_file("res://Assets/Textures/bayer.png")
var DitherShaderNode: ColorRect
var DitherShaderMaterial: ShaderMaterial
var DitherBackBuffer: BackBufferCopy
const DITHER_ZONE_BOUNDS_UV_X: float = 0.5
const DITHER_ZONE_BOUNDS_UV_Y: float = 0.75

var iChannel0: Texture
var iResolution: Vector2

var MainViewport: Viewport
var BaseCanvasLayer: CanvasLayer
var UpperCanvasLayer: CanvasLayer


func _ready() -> void:
    RenderingServer.set_default_clear_color(Color.MIDNIGHT_BLUE)
    MainViewport = get_viewport()
    MainViewport.use_hdr_2d = true
    MainViewport.disable_3d = true
    #TODO: experiment with compositor effects for no reason because the issue was the fucking _process vs _physics_process functions
    #var environment: Environment = Environment.new()
    #environment.set_background(Environment.BGMode.BG_CANVAS)
    #self.environment = environment
    #var tilt_effect: TiltMaskCompositorEffect = TiltMaskCompositorEffect.new()
    #self.compositor = Compositor.new()
    #tilt_effect.set_effect_callback_type(CompositorEffect.EFFECT_CALLBACK_TYPE_POST_OPAQUE)
    #compositor.compositor_effects = [tilt_effect]
    iResolution = Resolution.resolution
    BaseCanvasLayer = CanvasLayer.new()
    BaseCanvasLayer.layer = 1
    add_child(BaseCanvasLayer)

    #TODO: always apply umbral zone first if there are overlaps in the
    # zones, because dither is going to serve as a penumbral gradient perhaps
    setup_ubmral_zone()
    setup_dither_zone()
    add_glacier_flow_test_scene()
    add_jump_mechanic_test_scene()


func setup_ubmral_zone() -> void:
    var umbral_zone_instance: UmbralZone = UmbralZone.new()
    var screen_space_umbral_zone_bounds_x: float = iResolution.x * UMBRAL_ZONE_BOUNDS_UV_X
    var screen_space_umbral_zone_bounds_y: float = iResolution.y * UMBRAL_ZONE_BOUNDS_UV_Y
    var screen_space_umbral_zone_bounds: Vector2 = Vector2(
        screen_space_umbral_zone_bounds_x, screen_space_umbral_zone_bounds_y
    )
    umbral_zone_instance.set_zone_bounds(screen_space_umbral_zone_bounds)
    add_child(umbral_zone_instance)

    UmbralShaderMaterial = ShaderMaterial.new()
    UmbralShaderMaterial.shader = UmbralShader
    UmbralShaderNode = ColorRect.new()
    UmbralShaderNode.size = iResolution
    UmbralShaderNode.material = UmbralShaderMaterial
    UmbralShaderMaterial.set_shader_parameter("iResolution", iResolution)
    UmbralShaderMaterial.set_shader_parameter(
        "umbral_zone_bounds", Vector2(UMBRAL_ZONE_BOUNDS_UV_X, UMBRAL_ZONE_BOUNDS_UV_Y)
    )
    UmbralShaderMaterial.set_shader_parameter(
        "iChannel1", PerspectiveTiltMask.perspective_tilt_mask_texture
    )
    #TODO: in Compatibility Mode/opengl, sampling the MainViewport here doesnt result in a framebuffer error BUTTT,
    # it results in this zone in the top left quadrant of the viewport, where there is right triangle on the bottom half of the quadrant that ends up
    # turning the character body 2D's sprite invisible (or very glitchy sampling when MainViewport.use_hdr_2d = true)
    # seems to be an opengl compatibility bug...
    # investigation: https://github.com/godotengine/godot-docs/issues/2808

    # TODO: this is for testing godot render behavior when not using hint_screen_texture in the viewport sampling shaders
    #UmbralShaderMaterial.set_shader_parameter("iChannel0", MainViewport.get_texture())

    UmbralBackBuffer = BackBufferCopy.new()
    UmbralBackBuffer.copy_mode = BackBufferCopy.COPY_MODE_VIEWPORT
    UmbralBackBuffer.add_child(UmbralShaderNode)
    BaseCanvasLayer.add_child(UmbralBackBuffer)


func setup_dither_zone() -> void:
    var dither_zone_instance: DitherZone = DitherZone.new()
    var screen_space_dither_zone_bounds_x: float = iResolution.x * UMBRAL_ZONE_BOUNDS_UV_X
    var screen_space_dither_zone_bounds_y: float = iResolution.y * UMBRAL_ZONE_BOUNDS_UV_Y
    var screen_space_dither_zone_bounds: Vector2 = Vector2(
        screen_space_dither_zone_bounds_x, screen_space_dither_zone_bounds_y
    )
    dither_zone_instance.set_zone_bounds(screen_space_dither_zone_bounds)
    add_child(dither_zone_instance)

    DitherShaderMaterial = ShaderMaterial.new()
    DitherShaderMaterial.shader = DitherShader
    DitherShaderNode = ColorRect.new()
    DitherShaderNode.size = iResolution
    DitherShaderNode.material = DitherShaderMaterial
    DitherShaderMaterial.set_shader_parameter("iResolution", iResolution)
    iChannel0 = ImageTexture.create_from_image(BayerTexture)
    DitherShaderMaterial.set_shader_parameter("iChannel0", iChannel0)
    DitherShaderMaterial.set_shader_parameter(
        "dither_zone_bounds", Vector2(DITHER_ZONE_BOUNDS_UV_X, DITHER_ZONE_BOUNDS_UV_Y)
    )
    DitherBackBuffer = BackBufferCopy.new()
    DitherBackBuffer.copy_mode = BackBufferCopy.COPY_MODE_VIEWPORT
    DitherBackBuffer.add_child(DitherShaderNode)
    BaseCanvasLayer.add_child(DitherBackBuffer)


func add_jump_mechanic_test_scene() -> void:
    #TODO: put mechanics in the tree dynamically or statically
    var mechanics_test_scene: PackedScene = (
        preload("res://godot/Test/Mechanics/MechanicsTest.tscn") as PackedScene
    )
    var mechanics_test: MechanicsTest = mechanics_test_scene.instantiate() as MechanicsTest
    #TODO: because in the shader we target the main viewport with MainViewport/get_viewport().get_texture()
    # we must add the mechanics test scene with its sprite to the main root node/i.e. the main viewport
    add_child(mechanics_test)


func add_glacier_flow_test_scene() -> void:
    var glacier_flow_test_scene: PackedScene = (
        preload("res://godot/Test/Shaders/Glacier/GlacierFlow.tscn") as PackedScene
    )
    var glacier_flow_test: GlacierFlow = glacier_flow_test_scene.instantiate() as GlacierFlow
    #TODO: because in the shader we target the main viewport with MainViewport/get_viewport().get_texture()
    # we must add the mechanics test scene with its sprite to the main root node/i.e. the main viewport
    add_child(glacier_flow_test)
