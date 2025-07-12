extends Node2D
class_name GhostShape

var BufferAShaderNode: ColorRect
#var BufferAShader: Shader = preload(ResourcePaths.GHOST)
var BufferAShader: Shader = preload(ResourcePaths.MUSIC_BALL)
var BufferAShaderMaterial: ShaderMaterial
var BufferA: SubViewport
var MainImage: TextureRect

var iResolution: Vector2
var iChannel0: Texture = preload(ResourcePaths.BAYER_PNG)
var iChannel1: Texture
var iChannel2: Texture

var fft_texture: FFTTextureNode

var pitch_dimension: PitchDimensionGodot
var rhythm_dimension: RhythmDimension


func _ready() -> void:
    iResolution = ResolutionManager.resolution
    BufferA = ShaderToyUtil.create_buffer_viewport(iResolution)
    BufferAShaderMaterial = ShaderMaterial.new()
    BufferAShaderNode = ColorRect.new()
    BufferAShaderNode.size = iResolution
    BufferAShaderMaterial.shader = BufferAShader
    BufferAShaderNode.material = BufferAShaderMaterial
    BufferAShaderMaterial.set_shader_parameter("iResolution", iResolution)
    BufferAShaderMaterial.set_shader_parameter("iChannel0", iChannel0)
    fft_texture = FFTTextureNode.new()
    pitch_dimension = PitchDimensionGodot.new()
    rhythm_dimension = RhythmDimension.new()

    MainImage = TextureRect.new()
    MainImage.texture = BufferA.get_texture()
    MainImage.flip_v = true
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    add_child(fft_texture)
    add_child(pitch_dimension)
    add_child(rhythm_dimension)
    BufferAShaderNode.owner = BufferA
    BufferA.owner = self
    MainImage.owner = self
    fft_texture.owner = self
    pitch_dimension.owner = self
    rhythm_dimension.owner = self

    BufferAShaderMaterial.set_shader_parameter("bpm", rhythm_dimension.bpm)
    var f_onsets: PackedVector2Array = rhythm_dimension.f_onsets_flat_buffer
    var j_onsets: PackedVector2Array = rhythm_dimension.j_onsets_flat_buffer
    BufferAShaderMaterial.set_shader_parameter("f_onsets", f_onsets)
    BufferAShaderMaterial.set_shader_parameter("j_onsets", j_onsets)
    BufferAShaderMaterial.set_shader_parameter("f_onset_count", f_onsets.size())
    BufferAShaderMaterial.set_shader_parameter("j_onset_count", j_onsets.size())

    AudioPoolManager.play_music(pitch_dimension.get_wav_stream())


func _process(delta: float) -> void:
    BufferAShaderMaterial.set_shader_parameter("song_time", MusicDimensionsManager.song_time)
    iChannel1 = fft_texture.audio_texture
    BufferAShaderMaterial.set_shader_parameter("iChannel1", iChannel1)
    var hsv_buffer: PackedVector3Array = pitch_dimension.get_hsv_buffer()
    BufferAShaderMaterial.set_shader_parameter("hsv_buffer", hsv_buffer)
