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

const WORKGROUP_TILE_PIXELS_X: int = 2
const WORKGROUP_TILE_PIXELS_Y: int = 2


func _ready() -> void:
    _init_rendering_device()


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


func _compute_hull_pool_cpu(boundary_tile_lists: Array[PackedVector2Array]) -> int:
    for hull_arr: PackedVector2Array in CollisionMask.collision_polygon_hulls_pool:
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


func _compute_hull_pool_gpu(boundary_tile_lists: Array[PackedVector2Array]) -> int:
    const MIN_REGION_TILES: int = 20
    const MIN_HULL_AREA: float = 50.0
    for hull_arr: PackedVector2Array in CollisionMask.collision_polygon_hulls_pool:
        hull_arr.clear()

    var used: int = 0
    for region_index: int in range(boundary_tile_lists.size()):
        var tiles: PackedVector2Array = boundary_tile_lists[region_index]
        print("\n— Region ", region_index, " tiles=", tiles.size())
        if tiles.size() < MIN_REGION_TILES:
            print("    • skipped: < MIN_REGION_TILES (", MIN_REGION_TILES, ")")
            continue

        var centers: PackedVector2Array = _convert_boundary_tiles_to_center_point_list(
            tiles, TILE_SIZE_PIXELS, ComputeShaderLayer.iResolution
        )
        print("    • center points N=", centers.size())
        if centers.size() > CollisionMask.MAX_HULL_POINTS:
            print(
                "    • skipped: N(",
                centers.size(),
                ") > MAX_HULL_POINTS (",
                CollisionMask.MAX_HULL_POINTS,
                ")"
            )
            continue
        print("    • GPU hull start: N=", centers.size())
        var hull: PackedVector2Array = CollisionMask._compute_hull_gpu(centers)
        print("    • GPU hull points=", hull.size())

        if hull.size() < MIN_VERTICIES_FOR_CONVEX_HULL:
            print(
                "    • skipped: < MIN_VERTICIES_FOR_CONVEX_HULL (",
                MIN_VERTICIES_FOR_CONVEX_HULL,
                ")"
            )
            continue

        var area: float = 0.0
        for i: int in range(hull.size()):
            var j: int = (i + 1) % hull.size()
            area += hull[i].x * hull[j].y - hull[j].x * hull[i].y
        area = abs(area) * 0.5
        print("    • hull area=", area)

        if area < MIN_HULL_AREA:
            print("    • skipped: area <", MIN_HULL_AREA)
            continue

        print("    • accepted; sample pts:")
        for k: int in range(min(4, hull.size())):
            print("       [", k, "] =", hull[k])

        CollisionMask.collision_polygon_hulls_pool[used].append_array(hull)
        used += 1
        if used >= CollisionMask.MAX_COLLISION_SHAPES:
            break
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
        else:
            CollisionMask.collision_mask_polygons_pool[i].disabled = true
            CollisionMask.collision_mask_polygons_pool[i].polygon = []


func _compute_convex_hull_jarvis(tile_center_points: PackedVector2Array) -> PackedVector2Array:
    var point_count: int = tile_center_points.size()
    if point_count < MIN_VERTICIES_FOR_JARVIS:
        return tile_center_points.duplicate()
    var leftmost_index: int = 0
    for i: int in range(1, point_count):
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
        for j: int in range(point_count):
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
    for indexPosition: int in range(numberOfPoints):
        sortablePointList[indexPosition] = boundaryPointList[indexPosition]

    sortablePointList.sort_custom(_andrew_compare)

    var lowerHullPointStack: Array[Vector2] = []
    for currentPoint: Vector2 in sortablePointList:
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
    for reverseIndex: int in range(numberOfPoints - 1, -1, -1):
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
    for indexPosition: int in range(combinedHullPointList.size()):
        resultHullArray[indexPosition] = combinedHullPointList[indexPosition]

    return resultHullArray


func _andrew_compare(point_a: Vector2, point_b: Vector2) -> bool:
    if point_a.x == point_b.x:
        return point_a.y < point_b.y
    return point_a.x < point_b.x


func compute_convex_hull_marching_squares(
    region_tiles: PackedVector2Array
) -> PackedVector2Array:
    var minimum_tile_x: float = INF
    var minimum_tile_y: float = INF
    var maximum_tile_x: float = -INF
    var maximum_tile_y: float = -INF
    for tile: Vector2 in region_tiles:
        minimum_tile_x = min(minimum_tile_x, tile.x)
        minimum_tile_y = min(minimum_tile_y, tile.y)
        maximum_tile_x = max(maximum_tile_x, tile.x)
        maximum_tile_y = max(maximum_tile_y, tile.y)
    var width_in_tiles: int = int(maximum_tile_x - minimum_tile_x + 1)
    var height_in_tiles: int = int(maximum_tile_y - minimum_tile_y + 1)

    var mask: PackedByteArray = PackedByteArray()
    mask.resize(width_in_tiles * height_in_tiles)
    for tile: Vector2 in region_tiles:
        var local_x: int = int(tile.x - minimum_tile_x)
        var local_y: int = int(tile.y - minimum_tile_y)
        mask[local_y * width_in_tiles + local_x] = 1

    var origin: Vector2i = Vector2i(int(minimum_tile_x), int(minimum_tile_y))
    var polygon: PackedVector2Array = marchingSquaresContour(
        mask, width_in_tiles, height_in_tiles, TILE_SIZE_PIXELS, origin
    )
    if polygon.size() >= MIN_VERTICIES_FOR_ANDREW:
        polygon = _compute_convex_hull_andrew(polygon)

    return polygon


func _update_pixel_mask_array_pool_rgba8_or_r32ui(
    raw_data: PackedByteArray, width: int, height: int
) -> void:
    # raw_data is RGBA8: 4 bytes per pixel
    var total_pixels: int = width * height
    var index_pointer: int = 0
    for destination_index: int in range(total_pixels):
        # take only the R channel (byte 0 of each pixel)
        var v: int = raw_data[index_pointer]
        CollisionMask.pixel_mask_array_pool[destination_index] = 1 if (v != 0) else 0
        index_pointer += 4  # skip to next pixel’s R


func _update_pixel_mask_array_pool_r8ui(raw_data: PackedByteArray, width: int, height: int) -> void:
    var total_pixels: int = width * height
    for i: int in range(total_pixels):
        CollisionMask.pixel_mask_array_pool[i] = 1 if raw_data[i] != 0 else 0


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
    region_tile_packed_lists: Array[PackedVector2Array],
    tile_solidness_array_pool: PackedByteArray,
    tile_columns: int,
    tile_rows: int
) -> Array[PackedVector2Array]:
    var boundary_tile_packed_lists: Array[PackedVector2Array] = []

    for packed_tile_list: PackedVector2Array in region_tile_packed_lists:
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

    for y: int in range(start_y, end_y):
        for x: int in range(start_x, end_x):
            if pixel_mask_array[y * width + x] == 1:
                return true
    return false


func _enqueue_neighbors(cell: Vector2i, tile_columns: int, tile_rows: int, queue: Array) -> void:
    for direction: Vector2i in DIRECTIONS:
        var neighbor_x: int = cell.x + direction.x
        var neighbor_y: int = cell.y + direction.y
        if (
            neighbor_x >= 0
            and neighbor_y >= 0
            and neighbor_x < tile_columns
            and neighbor_y < tile_rows
        ):
            var index: int = neighbor_y * tile_columns + neighbor_x
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


func marchingSquaresContour(
    tileMask: PackedByteArray,
    tileColumnCount: int,
    tileRowCount: int,
    tileSizeInPixels: int,
    originTileCoordinate: Vector2i
) -> PackedVector2Array:
    var contourVertices: PackedVector2Array = PackedVector2Array()
    var directionOffsetList: Array[Vector2] = [
        Vector2(1, 0), Vector2(0, 1), Vector2(-1, 0), Vector2(0, -1)
    ]
    var directionCount: int = directionOffsetList.size()

    var startingTileCoordinate: Vector2 = Vector2(-1, -1)
    for rowIndex: int in range(tileRowCount):
        for columnIndex: int in range(tileColumnCount):
            var arrayIndex: int = rowIndex * tileColumnCount + columnIndex
            if tileMask[arrayIndex] == 1:
                for directionVector: Vector2 in directionOffsetList:
                    var neighborTileX: int = columnIndex + int(directionVector.x)
                    var neighborTileY: int = rowIndex + int(directionVector.y)
                    var isOutsideMask: bool = (
                        neighborTileX < 0
                        or neighborTileX >= tileColumnCount
                        or neighborTileY < 0
                        or neighborTileY >= tileRowCount
                    )
                    var neighborIndex: int = neighborTileY * tileColumnCount + neighborTileX
                    var isEmptyNeighbor: bool = false
                    if not isOutsideMask:
                        isEmptyNeighbor = (tileMask[neighborIndex] == 0)
                    if isOutsideMask or isEmptyNeighbor:
                        startingTileCoordinate = Vector2(columnIndex, rowIndex)
                        break

                if startingTileCoordinate.x >= 0:
                    break

        if startingTileCoordinate.x >= 0:
            break

    if startingTileCoordinate.x < 0:
        return contourVertices

    var currentTileCoordinate: Vector2 = startingTileCoordinate
    var previousDirectionIndex: int = directionCount - 1
    while true:
        for searchOffset: int in range(directionCount):
            var testDirectionIndex: int = (previousDirectionIndex + searchOffset) % directionCount
            var directionVector: Vector2 = directionOffsetList[testDirectionIndex]
            var testTileX: int = int(currentTileCoordinate.x) + int(directionVector.x)
            var testTileY: int = int(currentTileCoordinate.y) + int(directionVector.y)
            var isInsideBounds: bool = (
                testTileX >= 0
                and testTileX < tileColumnCount
                and testTileY >= 0
                and testTileY < tileRowCount
            )
            var testIndex: int = testTileY * tileColumnCount + testTileX
            if isInsideBounds and tileMask[testIndex] == 1:
                var pixelX: float = (
                    (
                        (originTileCoordinate.x + currentTileCoordinate.x)
                        + (1 if directionVector.x > 0 else 0)
                    )
                    * tileSizeInPixels
                )
                var pixelY: float = (
                    iResolution.y
                    - (
                        (
                            (originTileCoordinate.y + currentTileCoordinate.y)
                            + (1 if directionVector.y < 0 else 0)
                        )
                        * tileSizeInPixels
                    )
                )
                contourVertices.append(Vector2(pixelX, pixelY))
                currentTileCoordinate = Vector2(testTileX, testTileY)
                previousDirectionIndex = (testDirectionIndex + directionCount - 1) % directionCount
                break

        if currentTileCoordinate == startingTileCoordinate:
            break

    return contourVertices
