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


func _ready() -> void:
    MusicDimensionsManager.onset_event.connect(_on_onset_event)
    MusicDimensionsManager.tempo_event.connect(_on_tempo_event)
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
    fft_texture = FFTTexture.new()
    ioi_texture = IOITexture.new()

    MainImage = TextureRect.new()
    MainImage.texture = BufferA.get_texture()
    MainImage.flip_v = true
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    add_child(fft_texture)
    add_child(ioi_texture)


func _process(_delta: float) -> void:
    iChannel1 = fft_texture.audio_texture
    BufferAShaderMaterial.set_shader_parameter("iChannel1", iChannel1)
    iChannel2 = ioi_texture.audio_texture
    BufferAShaderMaterial.set_shader_parameter("iChannel2", iChannel2)


func _on_onset_event(
    onset_index: int, time_since_previous_onset: float, onsets_per_minute: float
) -> void:
    BufferAShaderMaterial.set_shader_parameter("onsets_per_minute", onsets_per_minute)


func _on_tempo_event(
    beat_index_within_bar: int,
    beat_phase_within_current_beat: float,
    beats_per_minute_true_tempo: float,
    seconds_per_beat_true_tempo: float,
    seconds_per_bar_true_tempo: float
) -> void:
    BufferAShaderMaterial.set_shader_parameter("beat_index", beat_index_within_bar)
    BufferAShaderMaterial.set_shader_parameter("beat_phase", beat_phase_within_current_beat)
    BufferAShaderMaterial.set_shader_parameter("seconds_per_beat", seconds_per_beat_true_tempo)
    BufferAShaderMaterial.set_shader_parameter("seconds_per_bar", seconds_per_bar_true_tempo)
