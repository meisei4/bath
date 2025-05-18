extends Node
class_name FFTTexture

const TEXTURE_HEIGHT: int = 1
const BUFFER_SIZE: int = 512
const MDN_BINS: int = 1024
const FFT_ROW: int = 0
const DEAD_CHANNEL: float = 0.0

var audio_texture: ImageTexture
var audio_image: Image
var fft_data: PackedFloat32Array


func _ready() -> void:
    fft_data.resize(BUFFER_SIZE)
    audio_image = Image.create(BUFFER_SIZE, TEXTURE_HEIGHT, false, Image.FORMAT_R8)
    audio_texture = ImageTexture.create_from_image(audio_image)


func _process(delta: float) -> void:
    update_fft_texture_row()
    audio_texture.set_image(audio_image)


func update_fft_texture_row() -> void:
    for bin_index: int in range(BUFFER_SIZE):
        var from_hz: float = bin_index * (MusicDimensionsManager.SAMPLE_RATE * 0.5) / MDN_BINS
        var to_hz: float = (bin_index + 1) * (MusicDimensionsManager.SAMPLE_RATE * 0.5) / MDN_BINS
        var smoothed_fft_value: float = (
            MusicDimensionsManager
            . compute_smooth_energy_for_frequency_range(from_hz, to_hz, fft_data[bin_index])
        )
        fft_data[bin_index] = smoothed_fft_value
        audio_image.set_pixel(
            bin_index, FFT_ROW, Color(smoothed_fft_value, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL)
        )
