extends Node2D
class_name GlacierFlow

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = load("res://Resources/Shaders/Glacier/glacier_main.gdshader")
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport
var MainImage: TextureRect
var iResolution: Vector2

var iChannel0: Texture
var iTime: float


func _ready() -> void:
    ComputeShaderSignalManager.register_glacier_flow(self)
    #TODO: i just set the default for canvas items to this in the project settings but seriously its annoying
    self.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
    iResolution = Resolution.resolution
    BufferA = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferAShaderMaterial = ShaderMaterial.new()
    BufferAShaderNode = ColorRect.new()
    BufferAShaderNode.size = iResolution
    BufferAShaderMaterial.shader = BufferAShader
    BufferAShaderNode.material = BufferAShaderMaterial
    BufferAShaderMaterial.set_shader_parameter("iResolution", iResolution)
    #TODO: this is really bad... I need to completely revamp the water shader to work with the glacier water correctly...
    #var water_projected: WaterProjected = WaterProjected.new()
    #add_child(water_projected)
    #BufferAShaderMaterial.set_shader_parameter("iChannel0", water_projected.get_water_texture())
    MainImage = TextureRect.new()
    MainImage.texture = BufferA.get_texture()
    MainImage.flip_v = true
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)


func _process(delta: float) -> void:
    iTime += delta
    ComputeShaderSignalManager.iTime_update.emit(iTime)
    BufferAShaderMaterial.set_shader_parameter("iTime", iTime)
