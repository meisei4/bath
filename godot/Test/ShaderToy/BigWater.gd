extends Node2D
class_name BigWater

var background_image: Texture = load("res://Assets/Textures/rocks.jpg")
var caustic_shader: Shader = load("res://Resources/Shaders/water.gdshader")
var caustic_material: ShaderMaterial
var ripple_shader: Shader = load("res://Resources/Shaders/finite_approx_ripple.gdshader")
var ripple_material: ShaderMaterial

var iTime: float = 0.0
var iMouse: Vector3 = Vector3(0.0, 0.0, 0.0)

var RippleBufferA: SubViewport
var RippleBufferA_Image: ColorRect
var RippleBufferB: SubViewport
var RippleBufferB_Image: TextureRect
var RippleMainImage: TextureRect

var WaterBufferA: SubViewport
var WaterBufferA_Image: ColorRect
var WaterMainImage: TextureRect  # final output

func _ready() -> void:
    var main_viewport_size: Vector2 = get_viewport_rect().size  # THIS WILL ALWAYS BE THE WINDOW I THINK
    RippleBufferA = SubViewport.new()
    RippleBufferA.disable_3d = true
    RippleBufferA.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    RippleBufferA.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    RippleBufferA.size = main_viewport_size

    RippleBufferA_Image = ColorRect.new()
    #RippleBufferA_Image.set_anchors_preset(ColorRect.PRESET_FULL_RECT)
    RippleBufferA_Image.size = main_viewport_size
    ripple_material = ShaderMaterial.new()
    ripple_material.shader = self.ripple_shader
    RippleBufferA_Image.material = self.ripple_material

    RippleBufferB = SubViewport.new()
    RippleBufferB.disable_3d = true
    RippleBufferB.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    RippleBufferB.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    RippleBufferB.size = main_viewport_size

    RippleBufferB_Image = TextureRect.new()
    RippleBufferB_Image.size = main_viewport_size
    RippleBufferB_Image.texture = RippleBufferA.get_texture()

    RippleMainImage = TextureRect.new()
    RippleMainImage.size = main_viewport_size
    RippleMainImage.texture = RippleBufferB.get_texture()

    RippleBufferA.add_child(RippleBufferA_Image)
    RippleBufferB.add_child(RippleBufferB_Image)
    add_child(RippleBufferA)
    add_child(RippleBufferB)
    add_child(RippleMainImage)

    ripple_material.set_shader_parameter("iResolution", main_viewport_size)
    ripple_material.set_shader_parameter("iChannel0", RippleMainImage.get_texture())

    WaterBufferA = SubViewport.new()
    WaterBufferA.disable_3d = true
    WaterBufferA.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    WaterBufferA.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    WaterBufferA.size = main_viewport_size

    WaterBufferA_Image = ColorRect.new()
    WaterBufferA_Image.size = main_viewport_size
    caustic_material = ShaderMaterial.new()
    caustic_material.shader = self.caustic_shader
    WaterBufferA_Image.material = self.caustic_material

    WaterMainImage = TextureRect.new()
    WaterMainImage.size = main_viewport_size
    WaterMainImage.texture = WaterBufferA.get_texture()

    WaterBufferA.add_child(WaterBufferA_Image)
    add_child(WaterBufferA)
    add_child(WaterMainImage)

    caustic_material.set_shader_parameter("iChannel3", RippleMainImage.get_texture())
    caustic_material.set_shader_parameter("iChannel1", background_image)

func _process(delta: float) -> void:
    iTime += delta
    var mouse_coords: Vector2 = get_viewport().get_mouse_position()
    var mouse_z: int = 1.0 if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) else 0.0
    iMouse = Vector3(mouse_coords.x, mouse_coords.y, mouse_z)
    #print("Mouse Position: ", mouse_coords, " | Mouse Z (pressed): ", mouse_z)
    ripple_material.set_shader_parameter("iTime", iTime)
    ripple_material.set_shader_parameter("iMouse", iMouse)
    #ripple_material.set_shader_parameter("iChannel0", RippleMainImage.get_texture())
