extends Node2D
class_name CollisionMaskScanlinePolygonizer

const MAX_POLYGONS: int = 12

var isp_texture: ISPTexture
var collision_mask_concave_polygons_pool: Array[CollisionShape2D]
var collision_mask_bodies: Array[StaticBody2D]
var polygon_active_global: PackedInt32Array
var polygon_active_local: PackedInt32Array

var previous_frame_count: int = 0
var previous_rows_scrolled: int = 0
var iResolution: Vector2


func _ready() -> void:
    iResolution = ResolutionManager.resolution
    _init_isp_texture()
    _init_concave_collision_polygon_pool()
    RenderingServer.frame_post_draw.connect(_on_frame_post_draw)


const PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR: float = 6.0
const PARALLAX_NEAR_SCALAR: float = 0.025
var polygon_original_nxs: Array[PackedFloat32Array]


func projectLayer(originalCoord: Vector2) -> Vector2:
    return originalCoord / (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - originalCoord.y)


const NOISE_SCROLL_VELOCITY: Vector2 = Vector2(0.0, 0.05)
const GLOBAL_COORD_SCALAR: float = 180.0

const STRETCH_SCALAR_X: float = 1.0
const STRETCH_SCALAR_Y: float = 2.0

const NOISE_COORD_OFFSET: Vector2 = Vector2(2.0, 0.0)

const ENABLE_STRETCH_CORRECTION: bool = true
const UNIFORM_STRETCH_CORRECTION_SCALAR: float = sqrt(2.0)

const ENABLE_ROTATION: bool = true
const ROTATION_ANGLE: float = -PI * 0.25

const ROT_COS: float = cos(ROTATION_ANGLE)
const ROT_SIN: float = sin(ROTATION_ANGLE)


func compute_quantized_vertical_pixel_coord(_iTime: float) -> int:
    var base_norm_top: Vector2 = Vector2(0.0, -1.0)
    var projected_base: Vector2 = projectLayer(base_norm_top)
    var y_displacement: float = _iTime * NOISE_SCROLL_VELOCITY.y
    projected_base.y = (
        (projected_base.y * PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR) / (1.0 + projected_base.y)
    )
    var projected_top_with_scroll: float = projected_base.y + y_displacement
    projected_top_with_scroll *= STRETCH_SCALAR_Y
    projected_top_with_scroll *= UNIFORM_STRETCH_CORRECTION_SCALAR
    projected_top_with_scroll *= ROT_COS
    var fragment_y: float = (projected_top_with_scroll * iResolution.y + iResolution.y) * 0.5
    var pixel_top_index: int = floori(fragment_y)
    return pixel_top_index


var previous_frames_quantized_vertical_pixel_coord: int = 0

var iTime: float


func _on_frame_post_draw() -> void:
    iTime = FragmentShaderSignalManager.ice_sheets.iTime
    var scanline_image: Image = (
        FragmentShaderSignalManager.ice_sheets.Scanline.get_texture().get_image()
    )
    isp_texture.update_scanline_alpha_bucket_bit_masks_from_image(scanline_image)
    var buckets: PackedVector2Array = isp_texture.fill_scanline_alpha_buckets_top_row()
    var current_frames_quantized_vertical_pixel_coord: int = compute_quantized_vertical_pixel_coord(
        iTime
    )
    var new_rows_this_frame: int = (
        current_frames_quantized_vertical_pixel_coord
        - previous_frames_quantized_vertical_pixel_coord
    )
    previous_frames_quantized_vertical_pixel_coord = current_frames_quantized_vertical_pixel_coord
    print("new rows this frame: ", new_rows_this_frame)
    for i: int in range(new_rows_this_frame):
        _update_polygons_with_alpha_buckets(buckets)
        _advance_polygons_by_one_pixel()


func _init_isp_texture() -> void:
    isp_texture = ISPTexture.new()
    add_child(isp_texture)


func _init_concave_collision_polygon_pool() -> void:
    polygon_active_global.resize(MAX_POLYGONS)
    polygon_active_global.fill(0)
    polygon_active_local.resize(MAX_POLYGONS)
    polygon_active_local.fill(0)
    polygon_original_nxs.resize(MAX_POLYGONS)
    for i: int in range(MAX_POLYGONS):
        var static_body: StaticBody2D = StaticBody2D.new()
        add_child(static_body)
        var shape_node: CollisionShape2D = CollisionShape2D.new()
        shape_node.disabled = true
        var concave: ConcavePolygonShape2D = ConcavePolygonShape2D.new()
        shape_node.shape = concave
        static_body.add_child(shape_node)
        collision_mask_bodies.append(static_body)
        collision_mask_concave_polygons_pool.append(shape_node)


func _update_polygons_with_alpha_buckets(alpha_buckets: PackedVector2Array) -> void:
    var num_buckets: int = alpha_buckets.size() / 2
    for bucket_index: int in range(num_buckets):
        var bucket_start: Vector2 = alpha_buckets[bucket_index * 2]
        var bucket_end: Vector2 = alpha_buckets[bucket_index * 2 + 1]
        var bucket_x_start: float = bucket_start.x
        var bucket_x_end: float = bucket_end.x
        var matched: bool = false
        for i: int in range(MAX_POLYGONS):
            if polygon_active_global[i] == 1:
                var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
                var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
                var segments: PackedVector2Array = concave.segments
                var seg_size: int = segments.size()
                if seg_size >= 2:
                    var top_left: float = segments[0].x
                    var top_right: float = segments[1].x
                    if bucket_x_start <= top_right and bucket_x_end >= top_left:
                        var localY: int = -int(shape_node.position.y)
                        segments.insert(0, Vector2(bucket_x_end, localY))
                        segments.insert(0, Vector2(bucket_x_start, localY))
                        concave.segments = segments
                        polygon_active_local[i] += 1
                        matched = true
                        var nx_left: float = (2.0 * bucket_x_start - iResolution.x) / iResolution.y
                        var nx_right: float = (2.0 * bucket_x_end - iResolution.x) / iResolution.y
                        var fragY_spawn: float = localY - shape_node.position.y
                        var orig_normY: float = (2.0 * fragY_spawn - iResolution.y) / iResolution.y
                        var world_left: float = (
                            nx_left * (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - orig_normY)
                        )
                        var world_right: float = (
                            nx_right * (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - orig_normY)
                        )
                        polygon_original_nxs[i].insert(0, world_right)
                        polygon_original_nxs[i].insert(0, world_left)
                        break

        if not matched:
            for i: int in range(MAX_POLYGONS):
                if polygon_active_global[i] == 0:
                    polygon_active_global[i] = 1
                    polygon_active_local[i] = 1
                    var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
                    var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
                    var new_segments: PackedVector2Array = PackedVector2Array()
                    new_segments.push_back(Vector2(bucket_x_start, 0))
                    new_segments.push_back(Vector2(bucket_x_end, 0))
                    concave.segments = new_segments
                    shape_node.disabled = false
                    var nx_left: float = (2.0 * bucket_x_start - iResolution.x) / iResolution.y
                    var nx_right: float = (2.0 * bucket_x_end - iResolution.x) / iResolution.y
                    var fragY_spawn: float = 0 + shape_node.position.y  # = shape_node.position.y (which is zero at creation)
                    var orig_normY: float = (2.0 * fragY_spawn - iResolution.y) / iResolution.y
                    var world_left: float = (
                        nx_left * (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - orig_normY)
                    )
                    var world_right: float = (
                        nx_right * (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - orig_normY)
                    )
                    polygon_original_nxs[i].append(world_left)
                    polygon_original_nxs[i].append(world_right)
                    break


const ONE_PIXEL: float = 1.0


func _advance_polygons_by_one_pixel() -> void:
    for i: int in range(MAX_POLYGONS):
        if polygon_active_global[i] == 1:
            var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
            shape_node.position.y += ONE_PIXEL
            _correct_polygon_horizontal(i)
            if shape_node.position.y > iResolution.y:
                _clear_polygon(i)


func _correct_polygon_horizontal(i: int) -> void:
    var orig_nxs: PackedFloat32Array = polygon_original_nxs[i]
    if orig_nxs == null:
        return

    var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
    var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
    var segments: PackedVector2Array = concave.segments
    if orig_nxs.size() != segments.size():
        return

    for j: int in range(segments.size()):
        var local_pt: Vector2 = segments[j]
        var fragY: float = local_pt.y + shape_node.position.y
        var normY_shader: float = (2.0 * fragY - iResolution.y) / iResolution.y
        var denom_shader: float = PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - normY_shader
        var scale_shader: float = 1.0 / denom_shader
        var worldX: float = orig_nxs[j]  # (stored earlier when this vertex was spawned)
        var projX: float = worldX * scale_shader
        var scrX: float = projX * (iResolution.y * 0.5) + (iResolution.x * 0.5)
        segments.set(j, Vector2(scrX, local_pt.y))
    concave.segments = segments


func _clear_polygon(index: int) -> void:
    polygon_active_global[index] = 0
    polygon_active_local[index] = 0
    polygon_original_nxs[index].clear()

    var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[index]
    var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
    concave.segments = PackedVector2Array()
    shape_node.position = Vector2.ZERO
    shape_node.disabled = true
