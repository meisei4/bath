extends Node2D
class_name WaveformVisualizer

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = load("res://Resources/Shaders/Audio/waveform.gdshader")
var BufferAShaderMaterial: ShaderMaterial
var audio_texture: WaveformTexture
var BufferA: SubViewport
var MainImage: TextureRect
var iResolution: Vector2
var iChannel0: Texture


func _ready() -> void:
    iResolution = ResolutionManager.resolution
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
    var music_resource: AudioStream = load(AudioConsts.SHADERTOY_MUSIC_TRACK_EXPERIMENT)
    AudioManager.play_music(music_resource)
    audio_texture = WaveformTexture.new()
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    add_child(audio_texture)


func _process(_delta: float) -> void:
    iChannel0 = audio_texture.audio_texture
    BufferAShaderMaterial.set_shader_parameter("iChannel0", iChannel0)
