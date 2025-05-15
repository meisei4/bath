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

var audio_texture: FFTTexture


func _ready() -> void:
    MusicDimensionsManager.beat_detected.connect(_on_beat_detected)
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

    #var music_resource: AudioStream = load(AudioConsts.HELLION_MP3)
    var music_resource: AudioStream = load(AudioConsts.SHADERTOY_MUSIC_TRACK_EXPERIMENT)
    AudioPoolManager.play_music(music_resource)
    audio_texture = FFTTexture.new()
    MainImage = TextureRect.new()
    MainImage.texture = BufferA.get_texture()
    MainImage.flip_v = true
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    add_child(audio_texture)


func _process(_delta: float) -> void:
    iChannel0 = audio_texture.audio_texture
    BufferAShaderMaterial.set_shader_parameter("iChannel1", iChannel0)


func _on_beat_detected(beat_index: int, delta_time: float, bpm: float) -> void:
    BufferAShaderMaterial.set_shader_parameter("bpm", bpm)
