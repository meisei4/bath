extends Node2D
class_name ShaderToy

var background_image: Texture = load("res://Assets/Textures/rocks.jpg")
var caustic_shader: Shader = load("res://Resources/Shaders/water.gdshader")
var caustic_material: ShaderMaterial
var ripple_shader: Shader = load("res://Resources/Shaders/finite_approx_ripple.gdshader")
var ripple_material: ShaderMaterial

var iTime: float = 0.0
var iMouse: Vector3 = Vector3(0.0, 0.0, 0.0)


var RippleBufferA: SubViewport
var RippleBufferB: SubViewport
var RippleMainImage: TextureRect

var ActiveBuffer: SubViewport
var inactive_buffer: SubViewport
var RippleShader: ColorRect

var WaterBuffer: SubViewport
var WaterMainImage: TextureRect
var WaterShader: ColorRect

func _ready() -> void:
    var main_viewport_size: Vector2 = get_viewport_rect().size  # Window size

    ## Initialize Ripple (Ping-Pong) Buffers and Shader
    RippleBufferA = create_viewport(main_viewport_size)
    RippleBufferB = create_viewport(main_viewport_size)
    ActiveBuffer = RippleBufferA
    inactive_buffer = RippleBufferB

    RippleShader = ColorRect.new()
    RippleShader.size = main_viewport_size
    ripple_material = ShaderMaterial.new()
    ripple_material.shader = ripple_shader
    ripple_material.set_shader_parameter("iResolution", main_viewport_size)
    RippleShader.material = ripple_material

    # Add the ripple shader to the active buffer
    ActiveBuffer.add_child(RippleShader)

    # Set up the TextureRect to capture the ripple output
    RippleMainImage = TextureRect.new()
    RippleMainImage.size = main_viewport_size

    ## Initialize Water (Caustic) Shader and Buffer
    WaterBuffer = create_viewport(main_viewport_size)
    WaterShader = ColorRect.new()
    WaterShader.size = main_viewport_size
    caustic_material = ShaderMaterial.new()
    caustic_material.shader = caustic_shader
    WaterShader.material = caustic_material

    # Add the water shader node to its buffer
    WaterBuffer.add_child(WaterShader)

    # Set up the TextureRect to capture the water shader output
    WaterMainImage = TextureRect.new()
    WaterMainImage.size = main_viewport_size

    ## Add Nodes to the Scene
    add_child(ActiveBuffer)
    add_child(inactive_buffer)
    add_child(RippleMainImage)
    add_child(WaterBuffer)
    add_child(WaterMainImage)

    ## Wait for the first frame to ensure render targets are updated
    #await RenderingServer.frame_post_draw
    ## Initialize Shader Texture Parameters after textures are valid
    RippleMainImage.texture = ActiveBuffer.get_texture()
    ripple_material.set_shader_parameter("iChannel0", RippleMainImage.texture)

    WaterMainImage.texture = WaterBuffer.get_texture()
    caustic_material.set_shader_parameter("iChannel3", RippleMainImage.texture)
    caustic_material.set_shader_parameter("iChannel1", background_image)



func create_viewport(size: Vector2) -> SubViewport:
    var vp = SubViewport.new()
    vp.size = size
    vp.disable_3d = true
    vp.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    return vp

func _process(delta: float) -> void:
    iTime += delta
    var mouse_coords: Vector2 = get_viewport().get_mouse_position()
    var mouse_z: int = 1.0 if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) else 0.0
    iMouse = Vector3(mouse_coords.x, mouse_coords.y, mouse_z)
    #print("Mouse Position: ", mouse_coords, " | Mouse Z (pressed): ", mouse_z)
    ripple_material.set_shader_parameter("iTime", iTime)
    ripple_material.set_shader_parameter("iMouse", iMouse)

    RippleMainImage.texture = ActiveBuffer.get_texture()
    ripple_material.set_shader_parameter("iChannel0", RippleMainImage.texture)
    caustic_material.set_shader_parameter("iChannel3", RippleMainImage.texture)
    ActiveBuffer.remove_child(RippleShader)
    inactive_buffer.add_child(RippleShader)

    var temp = ActiveBuffer
    ActiveBuffer = inactive_buffer
    inactive_buffer = temp
