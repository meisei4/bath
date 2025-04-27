extends Node2D
#class_name ComputeShaderLayer

var rendering_device: RenderingDevice
var iResolution: Vector2

# What are “work-groups” and “tiles”?
# •   A work-group is the smallest set of threads a GPU lets you
#     synchronise and share on-chip memory with.  Vulkan (and GLSL) call the
#     size of that set local_size.  In the shader we fixed it to
#         layout(local_size_x = 2, local_size_y = 2)  →  2 × 2 × 1 = 4 threads
# •   Because every thread in our kernel processes exactly one output
#     pixel, one work-group therefore covers a   2 × 2   block of pixels.
#     We call that block a tile.
# •   At dispatch time we have to tell the driver *how many* of those tiles
#     we need to blanket the whole render target.  For a screen that is
#     iResolution.x × iResolution.y pixels large the math is simply
#         groups_x = ceil(iResolution.x / WORKGROUP_TILE_PIXELS_X)
#         groups_y = ceil(iResolution.y / WORKGROUP_TILE_PIXELS_Y)
#     so every pixel (even the ragged right/bottom edges) gets exactly one
#     shader invocation.
#
# •   The three numbers we finally hand to
#         compute_list_dispatch(…, groups_x, groups_y, groups_z)
#     are therefore:
#          (#tiles horizontally,  #tiles vertically,  #tiles in depth-Z)
#     and the GPU launches   groups_x × groups_y × groups_z   work-groups,
#     each containing     local_size_x × local_size_y × local_size_z
#     threads.

const WORKGROUP_TILE_PIXELS_X: int = 2  # one work-group covers 2×2 pixels horizontally
const WORKGROUP_TILE_PIXELS_Y: int = 2  # one work-group covers 2×2 pixels vertically


func _ready() -> void:
    _init_rendering_device()


#UTIL STUFF FOR Render pipeline...
func _init_rendering_device() -> void:
    rendering_device = RenderingServer.get_rendering_device()
    iResolution = Resolution.resolution


func dispatch_compute(
    compute_pipeline_rid: RID, uniform_set_rid: RID, push_constants: PackedByteArray
) -> void:
    var compute_list_int: int = rendering_device.compute_list_begin()
    rendering_device.compute_list_bind_compute_pipeline(compute_list_int, compute_pipeline_rid)
    rendering_device.compute_list_bind_uniform_set(compute_list_int, uniform_set_rid, 0)
    if push_constants.size() > 0:
        rendering_device.compute_list_set_push_constant(
            compute_list_int, push_constants, push_constants.size()
        )
    var groups_x: int = int(ceil(iResolution.x / float(WORKGROUP_TILE_PIXELS_X)))
    var groups_y: int = int(ceil(iResolution.y / float(WORKGROUP_TILE_PIXELS_Y)))
    var groups_z: int = 1
    rendering_device.compute_list_dispatch(compute_list_int, groups_x, groups_y, groups_z)
    rendering_device.compute_list_end()


const TILE_SIZE_PIXELS: int = 2
const DIRECTIONS: Array[Vector2i] = [
    Vector2i(1, 0),
    Vector2i(-1, 0),
    Vector2i(0, 1),
    Vector2i(0, -1),
]
const MIN_VERTICIES_FOR_CONVEX_HULL: int = 7
const MIN_VERTICIES_FOR_JARVIS: int = 3
const MIN_VERTICIES_FOR_ANDREW: int = 3


func _compute_hull_pool(
    boundary_tile_lists: Array[PackedVector2Array], width: int, height: int
) -> int:
    for hull_arr in CollisionMask.collision_polygon_hulls_pool:
        hull_arr.clear()
    var used: int = 0
    for boundary_tiles: PackedVector2Array in boundary_tile_lists:
        if used >= CollisionMask.MAX_COLLISION_SHAPES:
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
        CollisionMask.collision_polygon_hulls_pool[used].append_array(convex_hull_point_list)
        used += 1
    return used


func _update_polygons_from_hulls(used: int) -> void:
    for i: int in range(CollisionMask.MAX_COLLISION_SHAPES):
        var collision_mask_polygon: CollisionPolygon2D = (
            CollisionMask.collision_mask_polygons_pool[i]
        )
        if i < used:
            var hull_verticies: Array = CollisionMask.collision_polygon_hulls_pool[i]
            collision_mask_polygon.disabled = false
            collision_mask_polygon.polygon = hull_verticies
            #TODO: update transform???????? polys do that automatically?
        else:
            CollisionMask.collision_mask_polygons_pool[i].disabled = true
            CollisionMask.collision_mask_polygons_pool[i].polygon = []


func _update_pixel_mask_array_pool_rgba8_or_r32ui(
    raw_data: PackedByteArray, width: int, height: int
) -> void:
    # raw_data is RGBA8: 4 bytes per pixel
    var total_pixels: int = width * height
    var index_pointer: int = 0
    for destination_index in range(total_pixels):
        # take only the R channel (byte 0 of each pixel)
        var v: int = raw_data[index_pointer]
        CollisionMask.pixel_mask_array_pool[destination_index] = 1 if (v != 0) else 0
        index_pointer += 4  # skip to next pixel’s R


func _update_pixel_mask_array_pool_r8ui(raw_data: PackedByteArray, width: int, height: int) -> void:
    var total_pixels: int = width * height
    for i in range(total_pixels):
        CollisionMask.pixel_mask_array_pool[i] = 1 if raw_data[i] != 0 else 0


func _disable_all_collision_polygons() -> void:
    for poly in CollisionMask.collision_mask_polygons_pool:
        poly.disabled = true
        poly.polygon = []  # <-- clear out old verts


func _update_tile_solidness_array(
    width: int, height: int, tile_columns: int, tile_rows: int, tile_size: int
) -> void:
    for tile_y: int in range(tile_rows):
        for tile_x: int in range(tile_columns):
            var index: int = tile_y * tile_columns + tile_x
            var is_solid: bool = _scan_any_solid_pixel_in_tile(
                CollisionMask.pixel_mask_array_pool, width, height, tile_x, tile_y, tile_size
            )
            CollisionMask.tile_solidness_array_pool[index] = 1 if is_solid else 0


func _find_all_connected_regions_in_tile_array_packed(
    tile_columns: int, tile_rows: int
) -> Array[PackedVector2Array]:
    var total_tiles: int = tile_columns * tile_rows
    for tile_index: int in range(total_tiles):
        CollisionMask.visited_array_pool[tile_index] = 0

    var region_tile_packed_lists: Array[PackedVector2Array] = []

    for tile_y: int in range(tile_rows):
        for tile_x: int in range(tile_columns):
            var linear_index: int = tile_y * tile_columns + tile_x
            if (
                CollisionMask.tile_solidness_array_pool[linear_index] == 1
                and CollisionMask.visited_array_pool[linear_index] == 0
            ):
                CollisionMask.visited_array_pool[linear_index] = 1
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


#TODO: fuck with this later, its a very important algo for graphics anyways, needs proper focus. stop getting carried away!!
func _compute_collision_polygons_marching_squares(
    tile_column_count: int, tile_row_count: int, tile_size_pixels: int
) -> int:
    for polygon_array in CollisionMask.collision_polygon_hulls_pool:
        polygon_array.clear()
    var shape_count: int = 0

    # This will hold every little line segment we generate
    # Each entry is an Array[Vector2] of length 2
    var edge_segment_list: Array = []
    # Map from "x,y" string to Array of segment-indices that touch that point
    var segment_indices_by_point: Dictionary = {}

    # --- STEP 1: march every 2×2 tile cell ---
    for cell_y in range(tile_row_count):
        for cell_x in range(tile_column_count):
            # Sample the four corners A,B,C,D of this cell.
            # If out of bounds, treat as empty (false).
            var sample_index_a: int = (cell_y + 1) * tile_column_count + cell_x
            var sample_index_b: int = (cell_y + 1) * tile_column_count + (cell_x + 1)
            var sample_index_c: int = cell_y * tile_column_count + cell_x
            var sample_index_d: int = cell_y * tile_column_count + (cell_x + 1)

            var is_corner_a_solid: bool = false
            var is_corner_b_solid: bool = false
            var is_corner_c_solid: bool = false
            var is_corner_d_solid: bool = false

            if cell_y + 1 < tile_row_count and cell_x < tile_column_count:
                is_corner_a_solid = (CollisionMask.tile_solidness_array_pool[sample_index_a] == 1)
            if cell_y + 1 < tile_row_count and cell_x + 1 < tile_column_count:
                is_corner_b_solid = (CollisionMask.tile_solidness_array_pool[sample_index_b] == 1)
            if cell_x < tile_column_count and cell_y < tile_row_count:
                is_corner_c_solid = (CollisionMask.tile_solidness_array_pool[sample_index_c] == 1)
            if cell_x + 1 < tile_column_count and cell_y < tile_row_count:
                is_corner_d_solid = (CollisionMask.tile_solidness_array_pool[sample_index_d] == 1)

            # Build the 4-bit case index
            var case_index: int = 0
            if is_corner_a_solid:
                case_index += 8
            if is_corner_b_solid:
                case_index += 4
            if is_corner_c_solid:
                case_index += 2
            if is_corner_d_solid:
                case_index += 1

            # Precompute the 4 midpoints (in *pixel* coordinates)
            var left_midpoint: Vector2 = Vector2(
                cell_x * tile_size_pixels, (cell_y + 0.5) * tile_size_pixels
            )
            var right_midpoint: Vector2 = Vector2(
                (cell_x + 1) * tile_size_pixels, (cell_y + 0.5) * tile_size_pixels
            )
            var top_midpoint: Vector2 = Vector2(
                (cell_x + 0.5) * tile_size_pixels, (cell_y + 1) * tile_size_pixels
            )
            var bottom_midpoint: Vector2 = Vector2(
                (cell_x + 0.5) * tile_size_pixels, cell_y * tile_size_pixels
            )

            # Pick segments based on the classic 16-case table
            var cell_segments: Array = []
            match case_index:
                0, 15:
                    # no boundary here
                    pass
                1:
                    cell_segments.append([bottom_midpoint, right_midpoint])
                2:
                    cell_segments.append([left_midpoint, bottom_midpoint])
                3:
                    cell_segments.append([left_midpoint, right_midpoint])
                4:
                    cell_segments.append([top_midpoint, right_midpoint])
                5:
                    cell_segments.append([left_midpoint, top_midpoint])
                    cell_segments.append([bottom_midpoint, right_midpoint])
                6:
                    cell_segments.append([bottom_midpoint, top_midpoint])
                7:
                    cell_segments.append([left_midpoint, top_midpoint])
                8:
                    cell_segments.append([top_midpoint, left_midpoint])
                9:
                    cell_segments.append([bottom_midpoint, top_midpoint])
                10:
                    cell_segments.append([left_midpoint, bottom_midpoint])
                    cell_segments.append([top_midpoint, right_midpoint])
                11:
                    cell_segments.append([top_midpoint, right_midpoint])
                12:
                    cell_segments.append([left_midpoint, right_midpoint])
                13:
                    cell_segments.append([bottom_midpoint, right_midpoint])
                14:
                    cell_segments.append([left_midpoint, bottom_midpoint])

            # Add each segment into our master list & index by endpoints
            for segment in cell_segments:
                var edge_index: int = edge_segment_list.size()
                edge_segment_list.append(segment)

                var first_point: Vector2 = segment[0]
                var second_point: Vector2 = segment[1]
                var first_key: String = str(first_point.x) + "," + str(first_point.y)
                var second_key: String = str(second_point.x) + "," + str(second_point.y)

                if not segment_indices_by_point.has(first_key):
                    segment_indices_by_point[first_key] = []
                segment_indices_by_point[first_key].append(edge_index)

                if not segment_indices_by_point.has(second_key):
                    segment_indices_by_point[second_key] = []
                segment_indices_by_point[second_key].append(edge_index)
        # end for cell_x
    # end for cell_y

    # --- STEP 2: stitch segments into closed loops ---
    var used_segment_flag_list: Array = []
    used_segment_flag_list.resize(edge_segment_list.size())
    for i in range(used_segment_flag_list.size()):
        used_segment_flag_list[i] = false

    for start_edge_index in range(edge_segment_list.size()):
        if used_segment_flag_list[start_edge_index]:
            continue

        # seed a new contour
        used_segment_flag_list[start_edge_index] = true
        var current_edge: Array = edge_segment_list[start_edge_index]
        var contour_points: Array = []
        contour_points.append(current_edge[0])
        contour_points.append(current_edge[1])
        var walk_point: Vector2 = current_edge[1]

        while true:
            # closed if we hit the first point again
            if walk_point == contour_points[0]:
                break

            var walk_key: String = str(walk_point.x) + "," + str(walk_point.y)
            var candidate_edges: Array = segment_indices_by_point[walk_key]
            var next_edge_index: int = -1
            for candidate_index in candidate_edges:
                if not used_segment_flag_list[candidate_index]:
                    next_edge_index = candidate_index
                    break
            if next_edge_index < 0:
                # open contour—give up
                break

            used_segment_flag_list[next_edge_index] = true
            var next_edge: Array = edge_segment_list[next_edge_index]
            var next_point: Vector2
            if next_edge[0] == walk_point:
                next_point = next_edge[1]
            else:
                next_point = next_edge[0]

            contour_points.append(next_point)
            walk_point = next_point
        # end while

        # store the closed poly (up to your MAX)
        if shape_count >= CollisionMask.MAX_COLLISION_SHAPES:
            break
        var packed_array: PackedVector2Array = PackedVector2Array()
        packed_array.resize(contour_points.size())
        for pi in range(contour_points.size()):
            packed_array[pi] = contour_points[pi]
        CollisionMask.collision_polygon_hulls_pool[shape_count].append_array(packed_array)
        shape_count += 1
    # end for each edge

    return shape_count


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

    sortablePointList.sort_custom(_andrew_compare)

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


func _andrew_compare(point_a: Vector2, point_b: Vector2) -> bool:
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
    for direction: Vector2i in DIRECTIONS:
        var neighbor_x = cell.x + direction.x
        var neighbor_y = cell.y + direction.y
        if (
            neighbor_x >= 0
            and neighbor_y >= 0
            and neighbor_x < tile_columns
            and neighbor_y < tile_rows
        ):
            var index = neighbor_y * tile_columns + neighbor_x
            if (
                CollisionMask.tile_solidness_array_pool[index] == 1
                and CollisionMask.visited_array_pool[index] == 0
            ):
                CollisionMask.visited_array_pool[index] = 1
                queue.append(Vector2i(neighbor_x, neighbor_y))


#TODO: stupid util function...
func _sprite_texture2d_to_rd(sprite_texture: Texture2D) -> Texture2DRD:
    var sprite_texture_format: RDTextureFormat = RDTextureFormat.new()
    sprite_texture_format.texture_type = RenderingDevice.TEXTURE_TYPE_2D
    sprite_texture_format.format = RenderingDevice.DATA_FORMAT_R8G8B8A8_UNORM
    sprite_texture_format.width = sprite_texture.get_width()
    sprite_texture_format.height = sprite_texture.get_height()
    sprite_texture_format.mipmaps = 1
    sprite_texture_format.usage_bits = (
        RenderingDevice.TEXTURE_USAGE_SAMPLING_BIT | RenderingDevice.TEXTURE_USAGE_CAN_UPDATE_BIT
    )
    var view: RDTextureView = RDTextureView.new()
    var view_rid: RID = rendering_device.texture_create(sprite_texture_format, view)
    var sprite_texture_image: Image = sprite_texture.get_image()
    rendering_device.texture_update(view_rid, 0, sprite_texture_image.get_data())
    var sprite_texture_rd: Texture2DRD = Texture2DRD.new()
    sprite_texture_rd.set_texture_rd_rid(view_rid)
    return sprite_texture_rd
