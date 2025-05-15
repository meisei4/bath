extends Node
class_name IOITexture

#https://en.wikipedia.org/wiki/Time_point#Interonset_interval

const IOI_BIN_COUNT: int = 32
const TEXTURE_HEIGHT: int = 1
const DEAD_CHANNEL: float = 0.0

var ioi_image: Image
var audio_texture: ImageTexture


func _ready() -> void:
    ioi_image = Image.create(IOI_BIN_COUNT, TEXTURE_HEIGHT, false, Image.FORMAT_R8)
    audio_texture = ImageTexture.create_from_image(ioi_image)
    MusicDimensionsManager.update_true_tempo.connect(_on_update_true_tempo)


func _on_update_true_tempo(ioi_vote_counts: PackedInt32Array, most_voted_bin_index: int) -> void:
    update_ioi_texture_row(ioi_vote_counts, most_voted_bin_index)
    audio_texture.set_image(ioi_image)


func update_ioi_texture_row(ioi_vote_counts: PackedInt32Array, most_voted_bin_index: int) -> void:
    var max_votes: int = 1
    for v: int in ioi_vote_counts:
        if v > max_votes:
            max_votes = v
    for bin_idx: int in range(IOI_BIN_COUNT):
        var votes: int = ioi_vote_counts[bin_idx]
        var amplitude: float = float(votes) / float(max_votes)
        amplitude = pow(amplitude, 0.4)
        amplitude = max(amplitude, 0.05)
        if bin_idx == most_voted_bin_index:
            amplitude = min(amplitude + 0.25, 1.0)

        var value: float = amplitude
        if bin_idx == most_voted_bin_index:
            value = clamp(amplitude + 0.25, 0.0, 1.0)

        ioi_image.set_pixel(bin_idx, 0, Color(value, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL))
