extends Node2D
class_name ShaderToy

var caustic_shader: Shader = load("res://Resources/Shaders/water.gdshader")
var caustic_material: ShaderMaterial
#var feedback_buffer_shader: Shader = load("res://Resources/Shaders/simple_feedback_buffer.gdshader")
#TODO:^^the above works fine just with up till BufferC being the texture fed into the shader as feedback
#TODO: see https://www.shadertoy.com/view/W3s3W8, it works the same way i think
#var ripple_shader: Shader = load("res://Resources/Shaders/finite_approx_ripple.gdshader")
var ripple_shader: Shader = load("res://Resources/Shaders/all_in_one_ripple.gdshader")
var ripple_material: ShaderMaterial

var background_image: Texture = load("res://Assets/Textures/rocks.jpg")

var iTime: float = 0.0
var iMouse: Vector3 = Vector3(0.0, 0.0, 0.0)

var BufferA: SubViewport
var BufferA_Image: ColorRect
var BufferB: SubViewport
var BufferB_Image: TextureRect
var BufferC: TextureRect
var BufferD: SubViewport
var BufferD_Image: ColorRect
var BufferE: SubViewport
var BufferE_Image: TextureRect
var MainImage: TextureRect


func _ready() -> void:
    var main_viewport_size: Vector2 = get_viewport_rect().size  # THIS WILL ALWAYS BE THE WINDOW I THINK
    BufferA = SubViewport.new()
    BufferA.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    BufferA.size = main_viewport_size

    BufferA_Image = ColorRect.new()
    BufferA_Image.color = Color(0.0, 0.0, 0.0, 0.0)
    BufferA_Image.size = main_viewport_size
    ripple_material = ShaderMaterial.new()
    ripple_material.shader = self.ripple_shader
    BufferA_Image.material = self.ripple_material

    BufferB = SubViewport.new()
    BufferB.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    BufferB.size = main_viewport_size

    BufferB_Image = TextureRect.new()
    BufferB_Image.size = main_viewport_size

    BufferB_Image.texture = BufferA.get_texture()

    BufferC = TextureRect.new()
    BufferC.size = main_viewport_size
    BufferC.texture = BufferB.get_texture()

    BufferA.add_child(BufferA_Image)
    BufferB.add_child(BufferB_Image)
    add_child(BufferA)
    add_child(BufferB)
    add_child(BufferC)

    ripple_material.set_shader_parameter("iChannel0", BufferC.texture)
    ripple_material.set_shader_parameter("iChannel1", background_image)
    ripple_material.set_shader_parameter("iResolution", main_viewport_size)

    #BufferD = SubViewport.new()
    #BufferD.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    #BufferD.size = main_viewport_size
#
    #BufferD_Image = ColorRect.new()
    #BufferD_Image.size = main_viewport_size
    #caustic_material = ShaderMaterial.new()
    #caustic_material.shader = self.caustic_shader
    #BufferD_Image.material = self.caustic_material
#
    #BufferE = SubViewport.new()
    #BufferE.render_target_clear_mode = SubViewport.CLEAR_MODE_ONCE
    #BufferE.size = main_viewport_size
#
    #BufferE_Image = TextureRect.new()
    #BufferE_Image.size = main_viewport_size
    #BufferE_Image.texture = BufferD.get_texture()
#
    #MainImage = TextureRect.new()
    #MainImage.size = main_viewport_size
    #MainImage.texture = BufferE.get_texture()

    #BufferD.add_child(BufferD_Image)
    #BufferE.add_child(BufferE_Image)
    #add_child(BufferD)
    #add_child(BufferE)
    #add_child(MainImage)

    #caustic_material.set_shader_parameter("iChannel1", background_image)
    #caustic_material.set_shader_parameter("iChannel3", MainImage.texture)


#TODO: I have no idea about the frozen shakey stuff in the result, seems like its not able to recognize even the following:
# 1- there needs to be an understood equilibrium of the heightmap
# 2- might be screwing up the viewport stuff up above
func _physics_process(delta: float) -> void:
    iTime += delta
    var mouse_coords: Vector2 = get_viewport().get_mouse_position()
    var mouse_z: int = 1.0 if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) else 0.0
    iMouse = Vector3(mouse_coords.x, mouse_coords.y, mouse_z)
    #print("Mouse Position: ", mouse_coords, " | Mouse Z (pressed): ", mouse_z)
    ripple_material.set_shader_parameter("iTime", iTime)
    ripple_material.set_shader_parameter("iMouse", iMouse)
