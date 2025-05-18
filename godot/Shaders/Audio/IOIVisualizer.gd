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
    audio_texture = IOITexture.new()
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    add_child(audio_texture)


func _process(delta: float) -> void:
    if MusicDimensionsManager.melody_index >= MusicDimensionsManager.melody_onsets.size():
        return

    MusicDimensionsManager.song_time += delta
    if (
        MusicDimensionsManager.song_time
        >= MusicDimensionsManager.melody_onsets[MusicDimensionsManager.melody_index]
    ):
        #AudioPoolManager.play_sfx(MusicDimensionsManager.metronome_click)
        MusicDimensionsManager.melody_index += 1

    #iChannel0 = audio_texture.audio_texture
    #BufferAShaderMaterial.set_shader_parameter("iChannel0", iChannel0)
    var ioi: float = 60.0 / MusicDimensionsManager.bpm
    MusicDimensionsManager.time_of_next_click -= delta
    if MusicDimensionsManager.time_of_next_click <= 0.0:
        #AudioPoolManager.play_sfx(MusicDimensionsManager.metronome_click)
        MusicDimensionsManager.time_of_next_click += ioi
