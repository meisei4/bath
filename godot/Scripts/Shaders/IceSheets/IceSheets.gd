extends Node2D
class_name IceSheets

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = preload(ResourcePaths.ICE_SHEETS_SHADER)
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport

var ScanlineShaderNode: ColorRect
var ScanlineShader: Shader = preload(ResourcePaths.SCANLINE_SHADER)
var ScanlineShaderMaterial: ShaderMaterial
var Scanline: SubViewport

var MainImage: TextureRect
var AlphaOverrideShader: Shader = preload(ResourcePaths.FREE_ALPHA_CHANNEL)
var MainImageMaterial: ShaderMaterial

var iResolution: Vector2

var iChannel0: Texture
var iTime: float


func _ready() -> void:
    FragmentShaderSignalManager.register_ice_sheets_fragment(self)
    self.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
    iResolution = ResolutionManager.resolution
    BufferA = ShaderToyUtil.create_buffer_viewport(iResolution)

    BufferAShaderMaterial = ShaderMaterial.new()
    BufferAShaderNode = ColorRect.new()
    BufferAShaderNode.size = iResolution
    BufferAShaderMaterial.shader = BufferAShader
    BufferAShaderNode.material = BufferAShaderMaterial
    BufferAShaderMaterial.set_shader_parameter("iResolution", iResolution)

    Scanline = ShaderToyUtil.create_buffer_viewport(Vector2(iResolution.x, 2.0))
    Scanline.use_hdr_2d = false
    ScanlineShaderMaterial = ShaderMaterial.new()
    ScanlineShaderNode = ColorRect.new()
    ScanlineShaderNode.size = Vector2(iResolution.x, 2.0)  # THIS IS WHERE THE CROP OCCURS
    ScanlineShaderMaterial.shader = ScanlineShader
    ScanlineShaderNode.material = ScanlineShaderMaterial
    ScanlineShaderMaterial.set_shader_parameter("iResolution", iResolution)
    ScanlineShaderMaterial.set_shader_parameter("iChannel0", BufferA.get_texture())

    MainImage = TextureRect.new()
    MainImage.texture = BufferA.get_texture()
    MainImage.flip_v = true
    MainImageMaterial = ShaderMaterial.new()
    MainImageMaterial.shader = AlphaOverrideShader
    MainImageMaterial.set_shader_parameter("iChannel0", BufferA.get_texture())

    MainImage.material = MainImageMaterial
    BufferA.add_child(BufferAShaderNode)
    Scanline.add_child(ScanlineShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    add_child(Scanline)
    #TODO: this is an ugly little issue where if i try to set the scanline texture to height 1 it wont work
    var img: Image = Scanline.get_texture().get_image()
    print("Scanline image is actually ", img.get_width(), "×", img.get_height())


func _process(delta: float) -> void:
    iTime += delta
    BufferAShaderMaterial.set_shader_parameter("iTime", iTime)
