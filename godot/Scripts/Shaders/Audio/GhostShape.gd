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

var fft_texture: FFTTexture
var ioi_texture: IOITexture

var pitch_dimension: PitchDimension
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
    fft_texture = FFTTexture.new()
    #ioi_texture = IOITexture.new()
    pitch_dimension = PitchDimension.new()
    rhythm_dimension = RhythmDimension.new()

    MainImage = TextureRect.new()
    MainImage.texture = BufferA.get_texture()
    MainImage.flip_v = true
    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    add_child(fft_texture)
    #add_child(ioi_texture)
    add_child(pitch_dimension)
    add_child(rhythm_dimension)
    BufferAShaderNode.owner = BufferA
    BufferA.owner = self
    MainImage.owner = self
    fft_texture.owner = self
    #ioi_texture.owner = self
    pitch_dimension.owner = self
    rhythm_dimension.owner = self

    BufferAShaderMaterial.set_shader_parameter("bpm", rhythm_dimension.bpm)
    var f_onsets: PackedVector2Array = rhythm_dimension.f_onsets_flat_buffer
    var j_onsets: PackedVector2Array = rhythm_dimension.j_onsets_flat_buffer
    BufferAShaderMaterial.set_shader_parameter("f_onsets", f_onsets)
    BufferAShaderMaterial.set_shader_parameter("j_onsets", j_onsets)
    BufferAShaderMaterial.set_shader_parameter("f_onset_count", f_onsets.size())
    BufferAShaderMaterial.set_shader_parameter("j_onset_count", j_onsets.size())


func _process(delta: float) -> void:
    BufferAShaderMaterial.set_shader_parameter("song_time", MusicDimensionsManager.song_time)
    iChannel1 = fft_texture.audio_texture
    BufferAShaderMaterial.set_shader_parameter("iChannel1", iChannel1)
    #iChannel2 = ioi_texture.audio_texture
    #BufferAShaderMaterial.set_shader_parameter("iChannel2", iChannel2)
    var hsv_buffer: PackedVector3Array = pitch_dimension.hsv_buffer
    #var fft_hsv_dummy: Vector3 = Vector3(0, 0, 1)
    #var light_ball_hsv_dummy: Vector3 = Vector3(0, 0, 1)
    #var hsv_buffer: PackedVector3Array = PackedVector3Array([light_ball_hsv_dummy, fft_hsv_dummy])
    BufferAShaderMaterial.set_shader_parameter("hsv_buffer", hsv_buffer)
