extends Node2D
class_name FFTVisualizer

var BufferAShaderNode: ColorRect
var BufferAShader: Shader = load(
    "res://Resources/Shaders/Audio/fft.gdshader"
)
var BufferAShaderMaterial: ShaderMaterial

var audio_texture: WaveformTexture

var BufferA: SubViewport
var MainImage: TextureRect

var iResolution: Vector2
var iChannel0: Texture


func _ready() -> void:
    var res: Vector2i = Vector2i(855, 480)
    DisplayServer.window_set_size(res)  #TODO: this doesnt do what you think it does
    iResolution = res
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
    var music_resource: AudioStream = load(AudioConsts.HELLION_MP3)
    #var music_resource: AudioStream = load(AudioConsts.HELLION)
    AudioManager.play_music(music_resource)
    #TODO: ^^^ ew, figure out how to perhaps make it more obvious that the audio texture can target whatever audio bus...
    audio_texture = WaveformTexture.new()  #TODO: this has to target a specific audio bus internally, figure out a better way

    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    add_child(audio_texture)


#TODO: its very important to control frame rate with these audio shaders
func _process(_delta: float) -> void:
    iChannel0 = audio_texture.audio_texture
    BufferAShaderMaterial.set_shader_parameter("iChannel0", iChannel0)
