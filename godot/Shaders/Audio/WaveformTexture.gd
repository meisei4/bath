extends Node
class_name WaveformTexture

var audio_texture: ImageTexture
var audio_image: Image
var waveform_audio_effect_capture: AudioEffectCapture

var TARGET_AUDIO_BUS: AudioBus.BUS = AudioBus.BUS.MUSIC
#var TARGET_AUDIO_BUS: AudioBus.BUS = AudioBus.BUS.INPUT

var waveform_data: PackedFloat32Array  #TODO: use these as uniforms perhaps later if it improves performance
#Bottom row (y = 1): Center is at (1 + 0.5) / 2 = 0.75
#Thus, in ShaderToy:
#Sampling at y = 0.75 fetches the raw waveform data.
const TEXTURE_HEIGHT: int = 2  #y = 0 is fft spectrum, y= 1 is raw wave data
const BUFFER_SIZE: int = 512
const SAMPLE_RATE: float = 48000.0  #TODO: figure out how to get this to actually be shadertoy matched
const AUDIO_DECIBEL_FLOOR: float = -80.0  # some default "quietest" decibel level (near silence) in the audio.
const AUDIO_DECIBEL_CIELING: float = 0.0  #the loudest?? is it just from the AudioBus attribute???

const WAVEFORM_ROW: int = 1
const DEAD_CHANNEL: float = 0.0


func _ready() -> void:
    prepare_waveform_audio_effect_capture()

    #TODO: make the image only 32-bit Red channel only: Image.FORMAT_RF = Red channel Full 32 bit range
    audio_image = Image.create(BUFFER_SIZE, TEXTURE_HEIGHT, false, Image.FORMAT_RF)
    audio_texture = ImageTexture.create_from_image(audio_image)


func _process(delta: float) -> void:
    update_waveform_texture_row()
    audio_texture.set_image(audio_image)


func prepare_waveform_audio_effect_capture() -> void:
    waveform_audio_effect_capture = AudioEffectCapture.new()
    waveform_audio_effect_capture.buffer_length = 0.01666666666  #TODO: 1/60 this is bat shit and results in unaligned injection and propagation somehow?
    #waveform_audio_effect_capture.buffer_length = 0.03333333333  #TODO: 1/30 this aligns kind of with the frame rate and thus the injection intervals and propagation??
    #waveform_audio_effect_capture.buffer_length = 0.06666666666 #TODO: 1/15 this is bat shit and results in unaligned injection and propagation somehow?
    AudioEffects.add_effect(TARGET_AUDIO_BUS, waveform_audio_effect_capture)
    waveform_data.resize(BUFFER_SIZE)


func update_waveform_texture_row() -> void:
    if waveform_audio_effect_capture.can_get_buffer(BUFFER_SIZE):
        #TODO: the get_buffer may return a variable sized vector array, since its just how many could be put into the buffer
        # get_buffer behavior: The samples are signed floating-point PCM between -1 and 1.
        # You will have to scale them if you want to use them as 8 or 16-bit integer samples:
        # (v = 0x7fff * samples[0].x)
        #TODO: experiment more with how to truly control this buffer size i the audio sampling
        var captured_frames_from_current_waveform_buffer: PackedVector2Array = (
            waveform_audio_effect_capture.get_buffer(BUFFER_SIZE)
        )
        var frame_count: float = captured_frames_from_current_waveform_buffer.size()
        #TODO: we need to scale down the buffer to fit into our 512 pixel length texture (ji.e. divide whatever size comes out of the capture)
        var frames_per_pixel: float = frame_count / BUFFER_SIZE
        for x: int in range(BUFFER_SIZE):
            var start_frame_index: float = x * frames_per_pixel
            var end_frame_index: float = (x + 1) * frames_per_pixel
            if end_frame_index > frame_count:
                end_frame_index = frame_count
            var accumulated_amplitudes: float = 0.0
            var number_of_amplitude_frames_to_average: int = 0
            for i: int in range(start_frame_index, end_frame_index):
                accumulated_amplitudes += captured_frames_from_current_waveform_buffer[i].x  #TODO: assumes mono audio PLEASE! (only get x, because x=y for mono)
                number_of_amplitude_frames_to_average += 1
            var average_amplitude: float = (
                accumulated_amplitudes / number_of_amplitude_frames_to_average
                if number_of_amplitude_frames_to_average > 0
                else 0.0
            )
            # TODO: normalize the amplitudes for the waveform data to be [0,1] (not -1,1)
            # optional accumulated_amplitudes += 0x7fff * captured_frames_from_current_waveform_buffer[0].x) to get 8bit or 16bit???
            average_amplitude = average_amplitude * 0.5 + 0.5
            waveform_data[x] = average_amplitude
            var audio_texture_value: Color = Color(
                average_amplitude, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL
            )
            audio_image.set_pixel(x, WAVEFORM_ROW, audio_texture_value)
    else:
        for x: int in range(BUFFER_SIZE):
            var audio_texture_value: Color = Color(
                waveform_data[x], DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL
            )
            audio_image.set_pixel(x, WAVEFORM_ROW, audio_texture_value)
