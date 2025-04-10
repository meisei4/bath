extends Node2D
class_name AudioTextureVisualizer

var BufferAShaderNode: ColorRect
#var BufferAShader: Shader = load("res://Resources/Shaders/Audio/basic_waveform.gdshader")
var BufferAShader: Shader = load(
    "res://Resources/Shaders/Audio/basic_fast_fourier_transform_spectrum.gdshader"
)
var BufferAShaderMaterial: ShaderMaterial

var shadertoy_audio_texture: ShaderToyAudioTexture

var BufferA: SubViewport
var MainImage: TextureRect

var iResolution: Vector2
var iChannel0: Texture


func _ready() -> void:
    var res: Vector2i = Vector2i(855, 480)
    DisplayServer.window_set_size(res)  #TODO: this doesnt do what you think it does
    iResolution = res
    BufferA = create_buffer_viewport(iResolution)
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
    #var music_resource: AudioStream = load(AudioConsts.HELLION)
    AudioManager.play_music(music_resource)
    #TODO: ^^^ ew, figure out how to perhaps make it more obvious that the audio texture can target whatever audio bus...
    shadertoy_audio_texture = ShaderToyAudioTexture.new()  #TODO: this has to target a specific audio bus internally, figure out a better way

    BufferA.add_child(BufferAShaderNode)
    add_child(BufferA)
    add_child(MainImage)
    add_child(shadertoy_audio_texture)


func create_buffer_viewport(resolution: Vector2) -> SubViewport:
    var subviewport: SubViewport = SubViewport.new()
    subviewport.size = resolution
    subviewport.disable_3d = true
    #TODO: be able to sample 16 bit at least??? even though i try to set the image format to FORMAT_RF (32-bit floats when i draw to the audio texture
    subviewport.use_hdr_2d = true
    RenderingServer.set_default_clear_color(Color.BLACK)
    subviewport.render_target_clear_mode = SubViewport.CLEAR_MODE_ALWAYS
    subviewport.render_target_update_mode = SubViewport.UPDATE_ALWAYS
    return subviewport


#TODO: its very important to control frame rate with these audio shaders
func _process(_delta: float) -> void:
    iChannel0 = shadertoy_audio_texture.audio_texture
    BufferAShaderMaterial.set_shader_parameter("iChannel0", iChannel0)
