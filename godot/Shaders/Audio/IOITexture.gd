extends Node
class_name IOITexture

#https://en.wikipedia.org/wiki/Time_point#Interonset_interval

const IOI_BIN_COUNT: int = 32
const TEXTURE_HEIGHT: int = 1
const DEAD_CHANNEL: float = 0.0

var ioi_image: Image
var ioi_texture: ImageTexture


func _ready() -> void:
    ioi_image = Image.create(IOI_BIN_COUNT, TEXTURE_HEIGHT, false, Image.FORMAT_R8)
    ioi_texture = ImageTexture.create_from_image(ioi_image)


func update_from_histogram(vote_array: PackedInt32Array, dominant_bin_index: int) -> void:
    var max_votes: int = 1
    for v: int in vote_array:
        if v > max_votes:
            max_votes = v
    for bin_idx: int in IOI_BIN_COUNT:
        var votes: int = vote_array[bin_idx]
        var amplitude: float = float(votes) / float(max_votes)

        var value: float = amplitude
        if bin_idx == dominant_bin_index:
            value = clamp(amplitude + 0.25, 0.0, 1.0)

        ioi_image.set_pixel(bin_idx, 0, Color(value, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL))

    ioi_texture.set_image(ioi_image)
