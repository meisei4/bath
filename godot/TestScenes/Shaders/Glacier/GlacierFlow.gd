extends Node2D
class_name GlacierFlow

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = load("res://Resources/Shaders/Glacier/glacier_main.gdshader")
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport
var MainImage: TextureRect
var iResolution: Vector2

var collision_mask: CollisionMask


func _ready() -> void:
    #TODO: i just set the default for canvas items to this in the project settings but seriously its annoying
    self.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
    #iResolution = get_viewport_rect().size
    iResolution = Resolution.resolution
    BufferA = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferAShaderMaterial = ShaderMaterial.new()
    BufferAShaderNode = ColorRect.new()
    BufferAShaderNode.size = iResolution
    BufferAShaderMaterial.shader = BufferAShader
    BufferAShaderNode.material = BufferAShaderMaterial
    BufferAShaderMaterial.set_shader_parameter("iResolution", iResolution)

    MainImage = TextureRect.new()
    MainImage.texture = BufferA.get_texture()
    MainImage.flip_v = true
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    if collision_mask == null:
        collision_mask = CollisionMask.new()
        add_child(collision_mask)


func _process(delta: float) -> void:
    #TODO: this is bad maybe?
    collision_mask.iTime += delta
    #also update the fragment shader material…
    BufferAShaderMaterial.set_shader_parameter("iTime", collision_mask.iTime)
