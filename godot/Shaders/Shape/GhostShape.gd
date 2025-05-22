extends Node2D
class_name GhostShape

var SampleTexture: Image = Image.load_from_file("res://Assets/Textures/bayer.png")

var BufferAShaderNode: ColorRect
#var BufferAShader: Shader = load("res://Resources/Shaders/Shape/ghost.gdshader")
var BufferAShader: Shader = load("res://Resources/Shaders/Audio/rhythm_ball.gdshader")
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport
var MainImage: TextureRect

var iResolution: Vector2
var iChannel0: Texture
var iChannel1: Texture
var iChannel2: Texture

var fft_texture: FFTTexture
var ioi_texture: IOITexture

var pitch_dimension: PitchDimension


func _ready() -> void:
    iResolution = ResolutionManager.resolution
    BufferA = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferAShaderMaterial = ShaderMaterial.new()
    BufferAShaderNode = ColorRect.new()
    BufferAShaderNode.size = iResolution
    BufferAShaderMaterial.shader = BufferAShader
    BufferAShaderNode.material = BufferAShaderMaterial
    BufferAShaderMaterial.set_shader_parameter("iResolution", iResolution)
    iChannel0 = ImageTexture.create_from_image(SampleTexture)
    BufferAShaderMaterial.set_shader_parameter("iChannel0", iChannel0)
    BufferAShaderMaterial.set_shader_parameter("bpm", MusicDimensionsManager.bpm)
    fft_texture = FFTTexture.new()
    ioi_texture = IOITexture.new()

    var onsets_buf = MusicDimensionsManager.custom_onsets_flat_buffer
    BufferAShaderMaterial.set_shader_parameter("custom_onsets", onsets_buf)
    BufferAShaderMaterial.set_shader_parameter("custom_onset_count", onsets_buf.size())

    pitch_dimension = PitchDimension.new()

    MainImage = TextureRect.new()
    MainImage.texture = BufferA.get_texture()
    MainImage.flip_v = true
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    add_child(fft_texture)
    add_child(ioi_texture)
    add_child(pitch_dimension)


func _process(delta: float) -> void:
    iChannel1 = fft_texture.audio_texture
    BufferAShaderMaterial.set_shader_parameter("iChannel1", iChannel1)
    #iChannel2 = ioi_texture.audio_texture
    #BufferAShaderMaterial.set_shader_parameter("iChannel2", iChannel2)
    #MusicDimensionsManager.debug_bpm_onsets(delta)
    #MusicDimensionsManager.debug_custom_onsets(delta)
    MusicDimensionsManager.debug_custom_onsets_ASCII(delta)
    var hsv_buffer: PackedVector3Array = pitch_dimension.hsv_buffer
    BufferAShaderMaterial.set_shader_parameter("hsv_buffer", hsv_buffer)
