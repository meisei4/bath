extends Node2D
class_name IceSheets

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = preload("res://Resources/Shaders/IceSheets/ice_sheets.gdshader")
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport
var MainImage: TextureRect
var AlphaOverrideShader: Shader = preload("res://Resources/Shaders/free_alpha_channel.gdshader")
var MainImageMaterial: ShaderMaterial

var iResolution: Vector2

var iChannel0: Texture
var iTime: float


func _ready() -> void:
    FragmentShaderSignalManager.register_ice_sheets_fragment(self)
    #ComputeShaderSignalManager.register_ice_sheets(self)
    #TODO: i just set the default for canvas items to this in the project settings but seriously its annoying
    self.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
    iResolution = ResolutionManager.resolution
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
    MainImageMaterial = ShaderMaterial.new()
    MainImageMaterial.shader = AlphaOverrideShader
    MainImageMaterial.set_shader_parameter("iChannel0", BufferA.get_texture())

    MainImage.material = MainImageMaterial
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)


func _process(delta: float) -> void:
    iTime += delta
    FragmentShaderSignalManager.iTime_update.emit(iTime)
    #ComputeShaderSignalManager.iTime_update.emit(iTime)
    BufferAShaderMaterial.set_shader_parameter("iTime", iTime)
