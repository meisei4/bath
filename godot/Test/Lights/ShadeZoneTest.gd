extends Node2D
class_name ShadeZoneTest

var ShadeBackBuffer: BackBufferCopy
var ShadeShader: Shader = load("res://Resources/Shaders/Light/shade.gdshader")
var ShadeShaderNode: ColorRect
var ShadeShaderMaterial: ShaderMaterial

var DitherBackBuffer: BackBufferCopy
var DitherShader: Shader = load("res://Resources/Shaders/Light/dither.gdshader")
var DitherShaderNode: ColorRect
var DitherShaderMaterial: ShaderMaterial
var BayerTexture: Image = Image.load_from_file("res://Assets/Textures/bayer.png")
var iChannel0: Texture

var SHADE_ZONE_BOUNDS_UV_X: float = 0.5
var SHADE_ZONE_BOUNDS_UV_Y: float = 1.0

var DITHER_ZONE_BOUNDS_UV_X: float = 0.5
var DITHER_ZONE_BOUNDS_UV_Y: float = 0.5

var BaseCanvasLayer: CanvasLayer
var MainViewport: Viewport
var iResolution: Vector2


func _ready():
    RenderingServer.set_default_clear_color(Color.CADET_BLUE)
    MainViewport = get_viewport()
    MainViewport.use_hdr_2d = true
    MainViewport.disable_3d = true
    iResolution = get_viewport_rect().size

    var shade_zone_instance: ShadeZone = ShadeZone.new()
    var screen_space_shade_zone_bounds_x: float = iResolution.x * SHADE_ZONE_BOUNDS_UV_X
    var screen_space_shade_zone_bounds_y: float = iResolution.y * SHADE_ZONE_BOUNDS_UV_Y
    var screen_space_shade_zone_bounds: Vector2 = Vector2(
        screen_space_shade_zone_bounds_x, screen_space_shade_zone_bounds_y
    )
    shade_zone_instance.set_zone_bounds(screen_space_shade_zone_bounds)
    add_child(shade_zone_instance)

    ShadeShaderMaterial = ShaderMaterial.new()
    ShadeShaderMaterial.shader = ShadeShader
    ShadeShaderNode = ColorRect.new()
    ShadeShaderNode.size = iResolution
    ShadeShaderNode.material = ShadeShaderMaterial
    ShadeShaderMaterial.set_shader_parameter("iResolution", iResolution)
    ShadeShaderMaterial.set_shader_parameter(
        "shade_zone_bounds", Vector2(SHADE_ZONE_BOUNDS_UV_X, SHADE_ZONE_BOUNDS_UV_Y)
    )
    #TODO: in Compatibility Mode/opengl, sampling the MainViewport here doesnt result in a framebuffer error BUTTT,
    # it results in this zone in the top left quadrant of the viewport, where there is right triangle on the bottom half of the quadrant that ends up
    # turning the character body 2D's sprite invisible (or very glitchy sampling when MainViewport.use_hdr_2d = true)
    # seems to be an opengl compatibility bug...
    # investigation: https://github.com/godotengine/godot-docs/issues/2808

    # TODO: this is for testing godot render behavior when not using hint_screen_texture in the viewport sampling shaders
    #ShadeShaderMaterial.set_shader_parameter("iChannel0", MainViewport.get_texture())

    var dither_zone_instance: DitherZone = DitherZone.new()
    var screen_space_dither_zone_bounds_x: float = iResolution.x * SHADE_ZONE_BOUNDS_UV_X
    var screen_space_dither_zone_bounds_y: float = iResolution.y * SHADE_ZONE_BOUNDS_UV_Y
    var screen_space_dither_zone_bounds: Vector2 = Vector2(
        screen_space_shade_zone_bounds_x, screen_space_shade_zone_bounds_y
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

    BaseCanvasLayer = CanvasLayer.new()
    BaseCanvasLayer.layer = 1
    #^^This allows for targetting an entire layer of the main root node, do more with this once introducing
    # z-index for psuedo 3D vertical layers
    ShadeBackBuffer = BackBufferCopy.new()
    ShadeBackBuffer.copy_mode = BackBufferCopy.COPY_MODE_VIEWPORT
    ShadeBackBuffer.add_child(ShadeShaderNode)
    BaseCanvasLayer.add_child(ShadeBackBuffer)
    #TODO: order matters a ton here!!!!! figure out why!!
    #it seems to be targetting the background (WHITE or BLUE, vs the actual shade cover background
    #but in this order, the mechanics sprite2D doesnt get dither applied to it... (3 shaders active though)
    DitherBackBuffer = BackBufferCopy.new()
    DitherBackBuffer.copy_mode = BackBufferCopy.COPY_MODE_VIEWPORT
    DitherBackBuffer.add_child(DitherShaderNode)
    BaseCanvasLayer.add_child(DitherBackBuffer)

    add_child(BaseCanvasLayer)

    ##TODO: put mechanics in the tree dynamically or put it in statically
    #var mechanics_test_scene = preload("res://godot/Test/Mechanics/MechanicsTest.tscn") as PackedScene
    #var mechanics_test: MechanicsTest = mechanics_test_scene.instantiate() as MechanicsTest
    ##TODO: because in the shader we target the main viewport with MainViewport/get_viewport().get_texture()
    ## we must add the mechanics test scene with its sprite to the main root node/i.e. the main viewport
    #add_child(mechanics_test)
