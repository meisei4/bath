extends Node2D
class_name SoundEnvelope

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = load("res://Resources/Shaders/Audio/BufferA_sound_envelope.gdshader")
#var BufferAShader: Shader = load("res://Resources/Shaders/Audio/optimized_envelope_buffer_a.gdshader")
var BufferAShaderMaterial: ShaderMaterial

var BufferBShaderNode: ColorRect
var BufferBShader: Shader = load("res://Resources/Shaders/Audio/Image_sound_envelope.gdshader")
#var BufferBShader: Shader = load("res://Resources/Shaders/Audio/optimized_envelope_buffer_b.gdshader")
var BufferBShaderMaterial: ShaderMaterial

var waveform_texture: WaveformTexture

var BufferA: SubViewport
var BufferB: SubViewport
var MainImage: TextureRect

var iResolution: Vector2
var iChannel0: Texture
var iChannel1: Texture
var iFrame: int = 0
var iTime: float = 0.0


func _ready() -> void:
    iResolution = Resolution.resolution
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
    #var music_resource: AudioStream = load(AudioConsts.SHADERTOY_MUSIC_TRACK_EXPERIMENT)
    var music_resource: AudioStream = load(AudioConsts.HELLION)
    #var music_resource: AudioStream = load(AudioConsts.BEETH)
    AudioManager.play_music(music_resource, 0.0)

    #var input_resource: AudioStreamMicrophone = AudioStreamMicrophone.new()
    #AudioManager.play_input(input_resource, 0.0)
    #TODO: ^^^ ew, figure out how to perhaps make it more obvious that the audio texture can target whatever audio bus...
    waveform_texture = WaveformTexture.new()  #TODO: this has to target a specific audio bus internally, figure out a better way

    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    BufferB.add_child(BufferBShaderNode)
    add_child(BufferB)
    add_child(MainImage)
    add_child(waveform_texture)


#TODO: its very important to control frame rate with these audio shaders
func _process(delta: float) -> void:
    iFrame += 1
    iChannel1 = waveform_texture.audio_texture
    #TODO: remember iChannel0 for BufferA is just screen hinted in the shader
    BufferAShaderMaterial.set_shader_parameter("iChannel1", iChannel1)

    iChannel0 = BufferA.get_texture() as ViewportTexture
    BufferBShaderMaterial.set_shader_parameter("iChannel0", iChannel0)
    BufferBShaderMaterial.set_shader_parameter("iFrame", iFrame)
