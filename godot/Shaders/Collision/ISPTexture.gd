extends Node
class_name ISPTexture

const TEXTURE_HEIGHT: int = 2
var TEXTURE_WIDTH: int = ResolutionManager.resolution.x
const DEAD_CHANNEL: float = 0.0

var scanline_alpha_bucket_data: PackedByteArray


func _ready() -> void:
    scanline_alpha_bucket_data.resize(TEXTURE_WIDTH * TEXTURE_HEIGHT)


func update_scanline_mask_from_scanline_image(_scanline_image: Image) -> void:
    var raw_rgba: PackedByteArray = _scanline_image.get_data()
    for i: int in range(TEXTURE_WIDTH * TEXTURE_HEIGHT):
        var alpha_byte: int = raw_rgba[4 * i + 3]
        scanline_alpha_bucket_data[i] = alpha_byte


func get_alpha_buckets_in_scanline() -> PackedVector2Array:
    var alpha_buckets: PackedVector2Array = PackedVector2Array()
    var w: int = TEXTURE_WIDTH
    var h: int = TEXTURE_HEIGHT
    for row: int in range(h):
        var in_bucket: bool = false
        var bucket_start: int = 0
        for x: int in range(w):
            var alpha: int = scanline_alpha_bucket_data[row * w + x]
            if alpha != 0 and not in_bucket:
                bucket_start = x
                in_bucket = true
            elif alpha == 0 and in_bucket:
                alpha_buckets.push_back(Vector2(bucket_start, row))
                alpha_buckets.push_back(Vector2(x - 1, row))
                in_bucket = false
        if in_bucket:
            alpha_buckets.push_back(Vector2(bucket_start, row))
            alpha_buckets.push_back(Vector2(w - 1, row))
    return alpha_buckets
