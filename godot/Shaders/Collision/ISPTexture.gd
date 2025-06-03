extends Node
class_name ISPTexture

const TEXTURE_HEIGHT: int = 1
var TEXTURE_WIDTH: int
const DEAD_CHANNEL: float = 0.0

var scanline_image_r8: Image
var scanline_image: Image
var scanline_texture: ImageTexture
var scanline_mask_data: PackedByteArray


func _ready() -> void:
    TEXTURE_WIDTH = ResolutionManager.resolution.x
    scanline_image_r8 = Image.create(TEXTURE_WIDTH, TEXTURE_HEIGHT, false, Image.FORMAT_R8)
    scanline_texture = ImageTexture.create_from_image(scanline_image_r8)
    scanline_mask_data.resize(TEXTURE_WIDTH * TEXTURE_HEIGHT)


func update_from_full_screen_image(full_screen_image: Image) -> void:
    full_screen_image.flip_y()
    var scanline_region: Rect2i = Rect2i(0, 0, TEXTURE_WIDTH, TEXTURE_HEIGHT)
    scanline_image = full_screen_image.get_region(scanline_region)
    scanline_image.convert(Image.FORMAT_RGBA8)
    scanline_image.flip_y()
    var raw_rgba: PackedByteArray = scanline_image.get_data()
    for i: int in range(TEXTURE_WIDTH * TEXTURE_HEIGHT):
        var alpha_byte: int = raw_rgba[4 * i + 3]
        scanline_mask_data[i] = alpha_byte
        scanline_image_r8.set_pixel(
            i % TEXTURE_WIDTH,
            i / TEXTURE_WIDTH,
            Color(alpha_byte / 255.0, DEAD_CHANNEL, DEAD_CHANNEL, DEAD_CHANNEL)
        )
    scanline_texture.update(scanline_image_r8)


func get_edge_buckets_in_scanline() -> PackedVector2Array:
    var edge_buckets: PackedVector2Array = PackedVector2Array()
    var w: int = TEXTURE_WIDTH
    var h: int = TEXTURE_HEIGHT
    for row: int in range(h):
        var in_bucket: bool = false
        var bucket_start: int = 0
        for x: int in range(w):
            var alpha: int = scanline_mask_data[row * w + x]
            if alpha != 0 and not in_bucket:
                bucket_start = x
                in_bucket = true
            elif alpha == 0 and in_bucket:
                edge_buckets.push_back(Vector2(bucket_start, row))
                edge_buckets.push_back(Vector2(x - 1, row))
                in_bucket = false
        if in_bucket:
            edge_buckets.push_back(Vector2(bucket_start, row))
            edge_buckets.push_back(Vector2(w - 1, row))
    return edge_buckets
