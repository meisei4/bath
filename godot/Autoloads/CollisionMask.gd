extends StaticBody2D

var iResolution: Vector2
var iTime: float

var rendering_device: RenderingDevice
var compute_shader_file: RDShaderFile = load(
    "res://Resources/Shaders/Compute/glacier_collision_mask.glsl"
)
var compute_shader_spirv: RDShaderSPIRV
var compute_shader_rid: RID
var compute_pipeline_rid: RID

const COLLISION_MASK_SSBO_UNIFORM_BINDING: int = 0
var collision_mask_uniform: RDUniform

var collision_mask_texture_format: RDTextureFormat
var collision_mask_view: RDTextureView
var collision_mask_texture_view_rid: RID
var collision_mask_texture: Texture2DRD

var gpu_side_collision_mask_ssbo_uniform_set_rid: RID

const MAX_COLLISION_SHAPES: int = 12
var collision_mask_polygons_pool: Array[CollisionPolygon2D] = []
var collision_polygon_hulls_pool: Array[PackedVector2Array] = []
var pixel_mask_array_pool: PackedByteArray = PackedByteArray()
var tile_solidness_array_pool: PackedByteArray = PackedByteArray()
var visited_array_pool: PackedByteArray = PackedByteArray()

var push_constants: PackedByteArray
const PUSH_CONSTANTS_BYTE_BLOCK_SIZE: int = 16
const PUSH_CONSTANTS_BYTE_ALIGNMENT_0: int = 0
const PUSH_CONSTANTS_BYTE_ALIGNMENT_4: int = 4
const PUSH_CONSTANTS_BYTE_ALIGNMENT_8: int = 8
const PUSH_CONSTANTS_BYTE_ALIGNMENT_12: int = 12

const WORKGROUP_TILE_PIXELS_X: int = 2
const WORKGROUP_TILE_PIXELS_Y: int = 2

const TILE_SIZE_PIXELS: int = 2
const DIRS: Array[Vector2i] = [
    Vector2i(1, 0),
    Vector2i(-1, 0),
    Vector2i(0, 1),
    Vector2i(0, -1),
]


func generate_collision_polygons() -> void:
    #debug_print_ascii()
    var width: int = int(iResolution.x)
    var height: int = int(iResolution.y)

    var raw_pixel_data: PackedByteArray = rendering_device.texture_get_data(
        collision_mask_texture_view_rid, 0
    )
    #_update_pixel_mask_array_pool_rgba8_or_r32ui(raw_pixel_data, width, height)
    _update_pixel_mask_array_pool_r8ui(raw_pixel_data, width, height)
    var tile_column_count: int = _calculate_tile_column_count(width, TILE_SIZE_PIXELS)
    var tile_row_count: int = _calculate_tile_row_count(height, TILE_SIZE_PIXELS)
    _update_tile_solidness_array(width, height, tile_column_count, tile_row_count, TILE_SIZE_PIXELS)
    var connected_regions: Array[PackedVector2Array] = _find_all_connected_regions_in_tile_array_packed(
        tile_column_count, tile_row_count
    )
    var boundary_tile_lists: Array[PackedVector2Array] = _find_boundary_tiles_for_each_region_packed(
        connected_regions, tile_solidness_array_pool, tile_column_count, tile_row_count
    )
    var used: int = _compute_hull_pool(boundary_tile_lists, width, height)
    _update_polygons_from_hulls(used)


const MIN_VERTICIES_FOR_CONVEX_HULL: int = 7


func _compute_hull_pool(
    boundary_tile_lists: Array[PackedVector2Array], width: int, height: int
) -> int:
    for hull_arr in collision_polygon_hulls_pool:
        hull_arr.clear()
    var used: int = 0
    for boundary_tiles: PackedVector2Array in boundary_tile_lists:
        if used >= MAX_COLLISION_SHAPES:
            break
        var center_point_list: PackedVector2Array = _convert_boundary_tiles_to_center_point_list(
            boundary_tiles, TILE_SIZE_PIXELS, iResolution
        )
        var convex_hull_point_list: PackedVector2Array = _compute_convex_hull_andrew(
            center_point_list
        )
        #TODO: andrew is faster gift wrapping I guess
        #var convex_hull_point_list: PackedVector2Array = _compute_convex_hull_jarvis(center_point_list)
        if convex_hull_point_list.size() < MIN_VERTICIES_FOR_CONVEX_HULL:
            continue
        collision_polygon_hulls_pool[used].append_array(convex_hull_point_list)
        used += 1
    return used


func _update_polygons_from_hulls(used: int) -> void:
    for i: int in range(MAX_COLLISION_SHAPES):
        var collision_mask_polygon: CollisionPolygon2D = collision_mask_polygons_pool[i]
        if i < used:
            var hull_verticies: Array = collision_polygon_hulls_pool[i]
            collision_mask_polygon.disabled = false
            collision_mask_polygon.polygon = hull_verticies
            #TODO: update transform???????? polys do that automatically?
        else:
            collision_mask_polygons_pool[i].disabled = true
            collision_mask_polygons_pool[i].polygon = []


func debug_print_ascii(tile_width: int = 4, tile_height: int = 8) -> void:
    var width: int = int(iResolution.x)
    var height: int = int(iResolution.y)
    var nonzero_pixel_count: int = 0
    for index: int in range(pixel_mask_array_pool.size()):
        if pixel_mask_array_pool[index] == 1:
            nonzero_pixel_count += 1
    print(" non-zero mask pixels:", nonzero_pixel_count)

    var tile_column_count: int = _calculate_tile_column_count(width, tile_width)
    var tile_row_count: int = _calculate_tile_row_count(height, tile_height)
    print(
        "ASCII square:",
        tile_column_count,
        "×",
        tile_row_count,
        "(tile_w=",
        tile_width,
        ", tile_h=",
        tile_height,
        ")"
    )

    for row_index: int in range(tile_row_count):
        var sample_y_position: int = clamp(row_index * tile_height + tile_height / 2, 0, height - 1)
        var source_y: int = height - 1 - sample_y_position
        var line_text: String = ""
        for column_index: int in range(tile_column_count):
            var sample_x_position: int = clamp(
                column_index * tile_width + tile_width / 2, 0, width - 1
            )
            line_text += (
                "#" if pixel_mask_array_pool[source_y * width + sample_x_position] == 1 else "."
            )
        print(" ", line_text)


func _ready() -> void:
    _init_collision_polygons()
    _init_rendering_device()
    _init_compute_shader_pipeline()
    _init_collision_mask_texture()
    #TODO: this is gross in CollisionMask AND PerspectiveTiltMask
    gpu_side_collision_mask_ssbo_uniform_set_rid = rendering_device.uniform_set_create(
        [collision_mask_uniform], compute_shader_rid, 0
    )
    _init_pools()
    RenderingServer.frame_pre_draw.connect(_dispatch_compute)
    RenderingServer.frame_post_draw.connect(generate_collision_polygons)


func _init_pools() -> void:
    var w = int(iResolution.x)
    var h = int(iResolution.y)
    pixel_mask_array_pool.resize(w * h)
    var cols = _calculate_tile_column_count(w, TILE_SIZE_PIXELS)
    var rows = _calculate_tile_row_count(h, TILE_SIZE_PIXELS)
    tile_solidness_array_pool.resize(cols * rows)
    visited_array_pool.resize(cols * rows)

    collision_polygon_hulls_pool.clear()
    collision_polygon_hulls_pool.resize(MAX_COLLISION_SHAPES)
    for i in range(MAX_COLLISION_SHAPES):
        var hull_buf: PackedVector2Array
        hull_buf.resize(0)  # empty
        collision_polygon_hulls_pool[i] = hull_buf


func _update_pixel_mask_array_pool_rgba8_or_r32ui(
    raw_data: PackedByteArray, width: int, height: int
) -> void:
    # raw_data is RGBA8: 4 bytes per pixel
    var total_pixels: int = width * height
    var index_pointer: int = 0
    for destination_index in range(total_pixels):
        # take only the R channel (byte 0 of each pixel)
        var v: int = raw_data[index_pointer]
        pixel_mask_array_pool[destination_index] = 1 if (v != 0) else 0
        index_pointer += 4  # skip to next pixel’s R


func _update_pixel_mask_array_pool_r8ui(raw_data: PackedByteArray, width: int, height: int) -> void:
    var total_pixels: int = width * height
    for i in range(total_pixels):
        pixel_mask_array_pool[i] = 1 if raw_data[i] != 0 else 0


func _init_collision_polygons() -> void:
    for i: int in range(MAX_COLLISION_SHAPES):
        var static_body: StaticBody2D = StaticBody2D.new()
        add_child(static_body)
        var collision_polygon: CollisionPolygon2D = CollisionPolygon2D.new()
        collision_polygon.disabled = true
        static_body.add_child(collision_polygon)
        collision_mask_polygons_pool.append(collision_polygon)


func _init_rendering_device() -> void:
    iResolution = Resolution.resolution
    rendering_device = RenderingServer.get_rendering_device()


func _init_compute_shader_pipeline() -> void:
    compute_shader_spirv = compute_shader_file.get_spirv()
    compute_shader_rid = rendering_device.shader_create_from_spirv(compute_shader_spirv)
    compute_pipeline_rid = rendering_device.compute_pipeline_create(compute_shader_rid)


func _init_collision_mask_texture() -> void:
    collision_mask_texture_format = RDTextureFormat.new()
    collision_mask_texture_format.texture_type = RenderingDevice.TEXTURE_TYPE_2D

    collision_mask_texture_format.format = RenderingDevice.DATA_FORMAT_R8_UINT
    #TODO: image format support bug???
    #collision_mask_texture_format.format = RenderingDevice.DATA_FORMAT_R32_UINT
    #collision_mask_texture_format.format = RenderingDevice.DATA_FORMAT_R8G8B8A8_UNORM

    collision_mask_texture_format.width = iResolution.x as int
    collision_mask_texture_format.height = iResolution.y as int
    collision_mask_texture_format.depth = 1
    collision_mask_texture_format.array_layers = 1
    collision_mask_texture_format.mipmaps = 1
    collision_mask_texture_format.usage_bits = (
        RenderingDevice.TEXTURE_USAGE_STORAGE_BIT
        | RenderingDevice.TEXTURE_USAGE_CAN_UPDATE_BIT
        | RenderingDevice.TEXTURE_USAGE_SAMPLING_BIT
        | RenderingDevice.TEXTURE_USAGE_CAN_COPY_FROM_BIT
    )
    collision_mask_view = RDTextureView.new()
    collision_mask_texture_view_rid = rendering_device.texture_create(
        collision_mask_texture_format, collision_mask_view
    )
    collision_mask_texture = Texture2DRD.new()
    collision_mask_texture.set_texture_rd_rid(collision_mask_texture_view_rid)

    collision_mask_uniform = RDUniform.new()
    collision_mask_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_IMAGE
    collision_mask_uniform.binding = COLLISION_MASK_SSBO_UNIFORM_BINDING
    collision_mask_uniform.add_id(collision_mask_texture_view_rid)


func _dispatch_compute() -> void:
    var compute_list_int: int = rendering_device.compute_list_begin()
    rendering_device.compute_list_bind_compute_pipeline(compute_list_int, compute_pipeline_rid)
    rendering_device.compute_list_bind_uniform_set(
        compute_list_int, gpu_side_collision_mask_ssbo_uniform_set_rid, 0
    )
    push_constants = PackedByteArray()
    push_constants.resize(PUSH_CONSTANTS_BYTE_BLOCK_SIZE)
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_0, iResolution.x)
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_4, iResolution.y)
    #TODO: this iTime global singleton value gets hack updated in the GlacierFlow test scene for now
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_8, iTime)
    push_constants.encode_u32(PUSH_CONSTANTS_BYTE_ALIGNMENT_12, 0)
    rendering_device.compute_list_set_push_constant(
        compute_list_int, push_constants, push_constants.size()
    )
    var groups_x = int(ceil(iResolution.x / float(WORKGROUP_TILE_PIXELS_X)))
    var groups_y = int(ceil(iResolution.y / float(WORKGROUP_TILE_PIXELS_Y)))
    rendering_device.compute_list_dispatch(compute_list_int, groups_x, groups_y, 1)
    rendering_device.compute_list_end()


#AUXILIARIES!!!!!!!!!!!!
func _disable_all_collision_polygons() -> void:
    for poly in collision_mask_polygons_pool:
        poly.disabled = true
        poly.polygon = []  # <-- clear out old verts


func _update_tile_solidness_array(
    width: int, height: int, tile_columns: int, tile_rows: int, tile_size: int
) -> void:
    for tile_y: int in range(tile_rows):
        for tile_x: int in range(tile_columns):
            var index: int = tile_y * tile_columns + tile_x
            var is_solid: bool = _scan_any_solid_pixel_in_tile(
                pixel_mask_array_pool, width, height, tile_x, tile_y, tile_size
            )
            tile_solidness_array_pool[index] = 1 if is_solid else 0


func _find_all_connected_regions_in_tile_array_packed(
    tile_columns: int, tile_rows: int
) -> Array[PackedVector2Array]:
    var total_tiles: int = tile_columns * tile_rows
    for tile_index: int in range(total_tiles):
        visited_array_pool[tile_index] = 0

    var region_tile_packed_lists: Array[PackedVector2Array] = []

    for tile_y: int in range(tile_rows):
        for tile_x: int in range(tile_columns):
            var linear_index: int = tile_y * tile_columns + tile_x
            if (
                tile_solidness_array_pool[linear_index] == 1
                and visited_array_pool[linear_index] == 0
            ):
                visited_array_pool[linear_index] = 1
                var queue: Array[Vector2i] = [Vector2i(tile_x, tile_y)]
                var packed_tile_list: PackedVector2Array = PackedVector2Array()
                packed_tile_list.resize(0)

                while queue.size() > 0:
                    var current_cell: Vector2i = queue.pop_back()
                    packed_tile_list.append(Vector2(current_cell.x, current_cell.y))
                    _enqueue_neighbors(current_cell, tile_columns, tile_rows, queue)

                region_tile_packed_lists.append(packed_tile_list)
    return region_tile_packed_lists


func _find_boundary_tiles_for_each_region_packed(
    region_tile_packed_lists: Array,
    tile_solidness_array_pool: PackedByteArray,
    tile_columns: int,
    tile_rows: int
) -> Array[PackedVector2Array]:
    var boundary_tile_packed_lists: Array[PackedVector2Array] = []

    for packed_tile_list in region_tile_packed_lists:
        var region_boundary_packed: PackedVector2Array = PackedVector2Array()
        region_boundary_packed.resize(0)

        for i: int in range(packed_tile_list.size()):
            var v: Vector2 = packed_tile_list[i]
            var cell_x: int = int(v.x)
            var cell_y: int = int(v.y)

            var is_boundary_tile: bool = false
            if (
                cell_x + 1 >= tile_columns
                or tile_solidness_array_pool[cell_y * tile_columns + (cell_x + 1)] == 0
            ):
                is_boundary_tile = true
            if (
                cell_x - 1 < 0
                or tile_solidness_array_pool[cell_y * tile_columns + (cell_x - 1)] == 0
            ):
                is_boundary_tile = true
            if (
                cell_y + 1 >= tile_rows
                or tile_solidness_array_pool[(cell_y + 1) * tile_columns + cell_x] == 0
            ):
                is_boundary_tile = true
            if (
                cell_y - 1 < 0
                or tile_solidness_array_pool[(cell_y - 1) * tile_columns + cell_x] == 0
            ):
                is_boundary_tile = true

            if is_boundary_tile:
                region_boundary_packed.append(Vector2(cell_x, cell_y))
        boundary_tile_packed_lists.append(region_boundary_packed)
    return boundary_tile_packed_lists


func _convert_boundary_tiles_to_center_point_list(
    boundary_tiles: Array[Vector2i], tile_size: int, resolution: Vector2
) -> PackedVector2Array:
    var size: int = boundary_tiles.size()
    var center_points: PackedVector2Array = PackedVector2Array()
    center_points.resize(size)
    var image_height: int = int(resolution.y)
    for i: int in range(size):
        var tile: Vector2i = boundary_tiles[i]
        var center_x: float = tile.x * tile_size + tile_size * 0.5
        var inverted_y: float = image_height - (tile.y * tile_size + tile_size * 0.5)
        center_points[i] = Vector2(center_x, inverted_y)
    return center_points


const MIN_VERTICIES_FOR_JARVIS: int = 3
const MIN_VERTICIES_FOR_ANDREW: int = 3


func _compute_convex_hull_jarvis(tile_center_points: PackedVector2Array) -> PackedVector2Array:
    var point_count: int = tile_center_points.size()
    if point_count < MIN_VERTICIES_FOR_JARVIS:
        return tile_center_points.duplicate()
    var leftmost_index: int = 0
    for i in range(1, point_count):
        var p: Vector2 = tile_center_points[i]
        var q: Vector2 = tile_center_points[leftmost_index]
        if p.x < q.x or (p.x == q.x and p.y < q.y):
            leftmost_index = i

    var hull_points: PackedVector2Array = PackedVector2Array()
    hull_points.resize(0)
    var current_index: int = leftmost_index
    while true:
        hull_points.append(tile_center_points[current_index])
        var next_index: int = (current_index + 1) % point_count
        for j in range(point_count):
            if (
                _orientation(
                    tile_center_points[current_index],
                    tile_center_points[next_index],
                    tile_center_points[j]
                )
                == -1
            ):
                next_index = j
        current_index = next_index
        if current_index == leftmost_index:
            break
    return hull_points


func _compute_convex_hull_andrew(boundaryPointList: PackedVector2Array) -> PackedVector2Array:
    var numberOfPoints: int = boundaryPointList.size()
    if numberOfPoints < MIN_VERTICIES_FOR_ANDREW:
        return boundaryPointList.duplicate()

    var sortablePointList: Array[Vector2] = []
    sortablePointList.resize(numberOfPoints)
    for indexPosition in range(numberOfPoints):
        sortablePointList[indexPosition] = boundaryPointList[indexPosition]

    sortablePointList.sort_custom(_compare_points_by_x_then_y)

    var lowerHullPointStack: Array[Vector2] = []
    for currentPoint in sortablePointList:
        while (
            lowerHullPointStack.size() >= 2
            and (
                _orientation(
                    lowerHullPointStack[lowerHullPointStack.size() - 2],
                    lowerHullPointStack[lowerHullPointStack.size() - 1],
                    currentPoint
                )
                <= 0
            )
        ):
            lowerHullPointStack.pop_back()
        lowerHullPointStack.append(currentPoint)

    var upperHullPointStack: Array[Vector2] = []
    for reverseIndex in range(numberOfPoints - 1, -1, -1):
        var currentPoint: Vector2 = sortablePointList[reverseIndex]
        while (
            upperHullPointStack.size() >= 2
            and (
                _orientation(
                    upperHullPointStack[upperHullPointStack.size() - 2],
                    upperHullPointStack[upperHullPointStack.size() - 1],
                    currentPoint
                )
                <= 0
            )
        ):
            upperHullPointStack.pop_back()
        upperHullPointStack.append(currentPoint)

    lowerHullPointStack.pop_back()
    upperHullPointStack.pop_back()
    var combinedHullPointList: Array[Vector2] = lowerHullPointStack + upperHullPointStack

    var resultHullArray: PackedVector2Array = PackedVector2Array()
    resultHullArray.resize(combinedHullPointList.size())
    for indexPosition in range(combinedHullPointList.size()):
        resultHullArray[indexPosition] = combinedHullPointList[indexPosition]

    return resultHullArray


func _compare_points_by_x_then_y(point_a: Vector2, point_b: Vector2) -> bool:
    if point_a.x == point_b.x:
        return point_a.y < point_b.y
    return point_a.x < point_b.x


func _orientation(a: Vector2, b: Vector2, c: Vector2) -> int:
    var cross_product: float = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    if cross_product > 0.0:
        return -1
    elif cross_product < 0.0:
        return 1
    else:
        return 0


func _calculate_tile_column_count(image_width: int, tile_size: int) -> int:
    return int((image_width + tile_size - 1) / tile_size)


func _calculate_tile_row_count(image_height: int, tile_size: int) -> int:
    return int((image_height + tile_size - 1) / tile_size)


func _scan_any_solid_pixel_in_tile(
    pixel_mask_array: PackedByteArray,
    width: int,
    height: int,
    tile_x: int,
    tile_y: int,
    tile_size: int
) -> bool:
    var start_x: int = tile_x * tile_size
    var end_x: int = min((tile_x + 1) * tile_size, width)
    var start_y: int = tile_y * tile_size
    var end_y: int = min((tile_y + 1) * tile_size, height)

    for y in range(start_y, end_y):
        for x in range(start_x, end_x):
            if pixel_mask_array[y * width + x] == 1:
                return true
    return false


func _enqueue_neighbors(cell: Vector2i, tile_columns: int, tile_rows: int, queue: Array) -> void:
    var cx = cell.x
    var cy = cell.y
    for dir in DIRS:
        var nx = cx + dir.x
        var ny = cy + dir.y
        if nx >= 0 and ny >= 0 and nx < tile_columns and ny < tile_rows:
            var idx = ny * tile_columns + nx
            if tile_solidness_array_pool[idx] == 1 and visited_array_pool[idx] == 0:
                visited_array_pool[idx] = 1
                queue.append(Vector2i(nx, ny))
