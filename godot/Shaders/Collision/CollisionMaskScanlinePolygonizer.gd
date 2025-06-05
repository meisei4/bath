extends Node2D
class_name CollisionMaskScanlinePolygonizer

const MAX_POLYGONS: int = 12

#TODO: hacked and they don't work as you'd think. two issues:
# 1. gdscript is using f64, glsl is using f32, all the velocity math gets fucked
# 2. gpu frame buffers are asynchronous and thus the collision shapes will never be scrolling at the same rate as if the cpu can capture it

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


func compute_quantized_vertical_pixel_coord_wrong(iTime: float) -> int:
    var dummy_norm_coord: Vector2 = Vector2(0.0, 0.0)
    var projected: Vector2 = projectLayer(dummy_norm_coord)
    var local_noise_scale: float = PARALLAX_NEAR_SCALAR

    var x_displacement: float = iTime * NOISE_SCROLL_VELOCITY.x
    var y_displacement: float = iTime * NOISE_SCROLL_VELOCITY.y
    var displaced_coordinate: Vector2 = projected + Vector2(x_displacement, y_displacement)
    var scaled_coordinate: Vector2 = displaced_coordinate * GLOBAL_COORD_SCALAR
    var stretched_coordinate: Vector2 = Vector2(
        scaled_coordinate.x * STRETCH_SCALAR_X, scaled_coordinate.y * STRETCH_SCALAR_Y
    )
    if ENABLE_STRETCH_CORRECTION:
        stretched_coordinate *= UNIFORM_STRETCH_CORRECTION_SCALAR
    if ENABLE_ROTATION:
        var tx: float = stretched_coordinate.x
        var ty: float = stretched_coordinate.y
        stretched_coordinate = Vector2(ROT_COS * tx - ROT_SIN * ty, ROT_SIN * tx + ROT_COS * ty)
    var local_noise_scaled_coordinate: Vector2 = stretched_coordinate * local_noise_scale
    var final_noise_coordinate: Vector2 = local_noise_scaled_coordinate - NOISE_COORD_OFFSET
    var approximate_pixel_space_coord = final_noise_coordinate.y * iResolution.y
    var current_frames_quantized_vertical_pixel_coord: int = floori(approximate_pixel_space_coord)
    return current_frames_quantized_vertical_pixel_coord


func compute_quantized_vertical_pixel_coord(iTime: float) -> int:
    var base_norm_top: Vector2 = Vector2(0.0, -1.0)
    var projected_base: Vector2 = projectLayer(base_norm_top)
    var y_displacement: float = iTime * NOISE_SCROLL_VELOCITY.y
    var projected_top_with_scroll: float = projected_base.y + y_displacement
    projected_top_with_scroll *= STRETCH_SCALAR_Y
    projected_top_with_scroll *= UNIFORM_STRETCH_CORRECTION_SCALAR
    projected_top_with_scroll -= NOISE_COORD_OFFSET.y
    var norm_top_after_scroll: float = (
        (projected_top_with_scroll * PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR)
        / (1.0 + projected_top_with_scroll)
    )
    var fragment_y: float = (projected_top_with_scroll * iResolution.y + iResolution.y) * 0.5
    var pixel_top_index: int = floori(fragment_y)
    return pixel_top_index


var previous_frames_quantized_vertical_pixel_coord: int = 0


func _on_frame_post_draw() -> void:
    var iTime: float = FragmentShaderSignalManager.ice_sheets.iTime
    var scanline_image: Image = (
        FragmentShaderSignalManager.ice_sheets.Scanline.get_texture().get_image()
    )
    isp_texture.update_scanline_mask_from_scanline_image(scanline_image)
    var buckets: PackedVector2Array = isp_texture.get_alpha_buckets_in_scanline()
    var current_frames_quantized_vertical_pixel_coord = compute_quantized_vertical_pixel_coord(
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
                        var nx_left = (2.0 * bucket_x_start - iResolution.x) / iResolution.y
                        var nx_right = (2.0 * bucket_x_end - iResolution.x) / iResolution.y
                        var orig_normY = (2.0 * 0 - iResolution.y) / iResolution.y  # = -1.0 at the top scanline
                        var world_left = (
                            nx_left * (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - orig_normY)
                        )
                        var world_right = (
                            nx_right * (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - orig_normY)
                        )
                        polygon_original_nxs[i].insert(0, world_right)  # matches segments[1]
                        polygon_original_nxs[i].insert(0, world_left)  # matches segments[0]
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
                    var orig_normY = -1.0  # because it’s still the top scanline
                    var world_left = (
                        nx_left * (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - orig_normY)
                    )
                    var world_right = (
                        nx_right * (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - orig_normY)
                    )
                    polygon_original_nxs[i].append(world_left)  # index 0 → matches segments[0]
                    polygon_original_nxs[i].append(world_right)
                    break


const ONE_PIXEL: float = 1.0
var polygon_centroid_cache: Dictionary = {}


func _advance_polygons_by_one_pixel() -> void:
    var new_centroid_cache: Dictionary = {}
    var previous_centroids: Array[Vector2] = []
    var previous_matched: Array[bool] = []
    for key in polygon_centroid_cache.keys():
        previous_centroids.append(polygon_centroid_cache[key] as Vector2)
        previous_matched.append(false)
    var MATCH_THRESHOLD: float = 4.0 * 4.0
    for i: int in range(MAX_POLYGONS):
        if polygon_active_global[i] == 1:
            var shape_node: CollisionShape2D = collision_mask_concave_polygons_pool[i]
            #shape_node.position.y += isp_texture.TEXTURE_HEIGHT
            shape_node.position.y += ONE_PIXEL
            #for sub in isp_texture.TEXTURE_HEIGHT:
            _correct_polygon_horizontal(i)
            var concave: ConcavePolygonShape2D = shape_node.shape as ConcavePolygonShape2D
            var segments: PackedVector2Array = concave.segments
            var centroid: Vector2 = Vector2.ZERO
            var point_count: int = segments.size()
            for pt in segments:
                centroid += pt + shape_node.position
            if point_count > 0:
                centroid /= point_count
            else:
                centroid = shape_node.position

            var touching_top: bool = centroid.y <= 0.0
            var touching_bottom: bool = centroid.y >= iResolution.y
            var fully_inside: bool = not touching_top and not touching_bottom
            var best_match_idx: int = -1
            var best_dist: float = INF
            for j: int in range(previous_centroids.size()):
                if previous_matched[j]:
                    continue
                var dist: float = centroid.distance_to(previous_centroids[j])
                if dist < best_dist and dist < MATCH_THRESHOLD:
                    best_dist = dist
                    best_match_idx = j

            if best_match_idx != -1:
                previous_matched[best_match_idx] = true
                var dy: float = centroid.y - previous_centroids[best_match_idx].y

            if not touching_bottom:
                new_centroid_cache[str(i)] = centroid
            if shape_node.position.y > iResolution.y:
                _clear_polygon(i)

    polygon_centroid_cache = new_centroid_cache


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
