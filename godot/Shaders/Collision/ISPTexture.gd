extends Node
class_name ISPTexture

const TEXTURE_HEIGHT: int = 2          # fixed – 2-pixel viewport
const ROW_HEIGHT:    int = 1           # one pixel per stored row
var   TEXTURE_WIDTH: int = ResolutionManager.resolution.x

var scanline_alpha_buckets_bit_mask_0: PackedByteArray   # top raster row
var scanline_alpha_buckets_bit_mask_1: PackedByteArray   # bottom raster row

func _ready() -> void:
    scanline_alpha_buckets_bit_mask_0 = PackedByteArray()
    scanline_alpha_buckets_bit_mask_1 = PackedByteArray()
    scanline_alpha_buckets_bit_mask_0.resize(TEXTURE_WIDTH * ROW_HEIGHT)
    scanline_alpha_buckets_bit_mask_1.resize(TEXTURE_WIDTH * ROW_HEIGHT)


func update_scanline_alpha_bucket_bit_masks_from_image(scanline_image: Image) -> void:
    #scanline_image.flip_y() # unsure still
    var raw_rgba: PackedByteArray = scanline_image.get_data() # 4 bytes per pixel
    var stride: int = TEXTURE_WIDTH * 4 # TO SHIFT IN THE FLATTENED BYTE BUFFER
    for x: int in range(TEXTURE_WIDTH):
        scanline_alpha_buckets_bit_mask_0[x] = raw_rgba[4 * x + 3]
    for x: int in range(TEXTURE_WIDTH):
        scanline_alpha_buckets_bit_mask_1[x] = raw_rgba[stride + 4 * x + 3]


func _build_scanline_alpha_buckets_from_1D_mask(alpha_byte_mask: PackedByteArray) -> PackedVector2Array:
    var buckets: PackedVector2Array = PackedVector2Array()
    var in_bucket: bool = false
    var start_x: int = 0
    for x: int in range(TEXTURE_WIDTH):
        if alpha_byte_mask[x] != 0 and not in_bucket:
            start_x = x
            in_bucket = true
        elif alpha_byte_mask[x] == 0 and in_bucket:
            buckets.push_back(Vector2(start_x, x - 1))
            in_bucket = false
    if in_bucket:
        buckets.push_back(Vector2(start_x, TEXTURE_WIDTH - 1))
    return buckets


func fill_scanline_alpha_buckets_top_row() -> PackedVector2Array:
    return _build_scanline_alpha_buckets_from_1D_mask(scanline_alpha_buckets_bit_mask_0)


func fill_scanline_alpha_buckets_bottom_row() -> PackedVector2Array:
    return _build_scanline_alpha_buckets_from_1D_mask(scanline_alpha_buckets_bit_mask_1)
