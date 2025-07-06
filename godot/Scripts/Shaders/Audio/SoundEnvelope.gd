extends Node2D
class_name SoundEnvelope

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = preload(ResourcePaths.BUFFERA_SOUND_ENVELOPE)
#var BufferAShader: Shader = preload(ResourcePaths.OPTIMIZED_ENVELOPE_BUFFER_A)
var BufferAShaderMaterial: ShaderMaterial

var BufferBShaderNode: ColorRect
var BufferBShader: Shader = preload(ResourcePaths.IMAGE_SOUND_ENVELOPE)
#var BufferBShader: Shader = preload(ResourcePaths.OPTIMIZED_ENVELOPE_BUFFER_B)
var BufferBShaderMaterial: ShaderMaterial

var waveform_texture: WaveformTextureNode

var BufferA: SubViewport
var BufferB: SubViewport
var MainImage: TextureRect

var iResolution: Vector2
var iChannel0: Texture
var iChannel1: Texture
var iFrame: int = 0
var iTime: float = 0.0

var ogg_stream: AudioStreamOggVorbis = preload(ResourcePaths.SHADERTOY_MUSIC_EXPERIMENT_OGG)


func _ready() -> void:
    iResolution = ResolutionManager.resolution
    BufferA = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferAShaderMaterial = ShaderMaterial.new()
    BufferAShaderNode = ColorRect.new()
    BufferAShaderNode.size = iResolution
    BufferAShaderMaterial.shader = BufferAShader
    BufferAShaderNode.material = BufferAShaderMaterial
    BufferAShaderMaterial.set_shader_parameter("iResolution", iResolution)
    BufferAShaderMaterial.set_shader_parameter("iFrame", iFrame)

    BufferB = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferBShaderMaterial = ShaderMaterial.new()
    BufferBShaderNode = ColorRect.new()
    BufferBShaderNode.size = iResolution
    BufferBShaderMaterial.shader = BufferBShader
    BufferBShaderNode.material = BufferBShaderMaterial
    BufferBShaderMaterial.set_shader_parameter("iResolution", iResolution)

    MainImage = TextureRect.new()
    MainImage.texture = BufferB.get_texture()
    MainImage.flip_v = true
    waveform_texture = WaveformTextureNode.new()

    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    BufferB.add_child(BufferBShaderNode)
    add_child(BufferB)
    add_child(MainImage)
    add_child(waveform_texture)
    BufferAShaderNode.owner = BufferA
    BufferA.owner = self
    BufferBShaderNode.owner = BufferB
    BufferB.owner = self
    MainImage.owner = self

    AudioPoolManager.play_music(ogg_stream)


func _process(_delta: float) -> void:
    iFrame += 1
    iChannel1 = waveform_texture.audio_texture
    BufferAShaderMaterial.set_shader_parameter("iChannel1", iChannel1)
    iChannel0 = BufferA.get_texture() as ViewportTexture
    BufferBShaderMaterial.set_shader_parameter("iChannel0", iChannel0)
    BufferBShaderMaterial.set_shader_parameter("iFrame", iFrame)
