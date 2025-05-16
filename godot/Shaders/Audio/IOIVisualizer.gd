extends Node2D
class_name IOIVisualizer


var BufferAShaderNode: ColorRect
var BufferAShader: Shader = load("res://Resources/Shaders/Audio/ioi.gdshader")
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport
var MainImage: TextureRect

var iResolution: Vector2
var iChannel0: Texture

var audio_texture: IOITexture

var metronome_click: AudioStream = preload("res://Resources/Audio/metronome_click.wav")
var time_of_next_click: float = 0.0


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
    AudioPoolManager.play_music(music_resource)
    audio_texture = IOITexture.new()
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    add_child(audio_texture)


var last_printed_bpm: float = -1.0
func _process(delta: float) -> void:
    iChannel0 = audio_texture.audio_texture
    BufferAShaderMaterial.set_shader_parameter("iChannel0", iChannel0)
    if audio_texture.bpm < audio_texture.MIN_BPM:
        return
    var ioi: float = 60.0 / audio_texture.bpm
    time_of_next_click -= delta
    if time_of_next_click <= 0.0:
        AudioPoolManager.play_sfx(metronome_click)
        time_of_next_click += ioi

    if abs(audio_texture.bpm - last_printed_bpm) > 0.1:
        print(audio_texture.bpm)
        last_printed_bpm = audio_texture.bpm
