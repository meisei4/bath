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

var util: TileUtilities
func _ready() -> void:
    util = TileUtilities.new()
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
const MIN_VERTICIES_FOR_ANDREW: int = 3


func _compute_hull_pool_cpu(connected_regions: Array[PackedVector2Array]) -> int:
    for hull_arr: PackedVector2Array in CollisionMask.collision_polygon_hulls_pool:
        hull_arr.clear()
    var used: int = 0
    for connected_region: PackedVector2Array in connected_regions:
        if used >= CollisionMask.MAX_COLLISION_SHAPES:
            break
        var convex_hull_point_list: PackedVector2Array = _compute_convex_hull_marching_squares(connected_region)
        if convex_hull_point_list.size() < MIN_VERTICIES_FOR_CONVEX_HULL:
            continue
        CollisionMask.collision_polygon_hulls_pool[used].append_array(convex_hull_point_list)
        used += 1

    return used


func _compute_hull_pool_cpu_with_region_cache(connected_regions: Array[PackedVector2Array]) -> int:
    var region_identifier_list: Array[int] = RegionCache.matchAndCacheRegions(connected_regions)
    var used: int = 0
    for region_index: int in range(region_identifier_list.size()):
        var cache_entry: RegionCache.RegionCacheEntry = RegionCache.region_cache_entry_list[region_index]
        if cache_entry.collisionShapePolygon.is_empty():
            cache_entry.collisionShapePolygon = (
                ComputeShaderLayer
                . _compute_convex_hull_marching_squares(cache_entry.tileCoordinatesList)
            )
            cache_entry.minimumTileY = RegionCache.computeMinimumTileY(cache_entry.tileCoordinatesList)
        else:
            var new_minimum_y: int = RegionCache.computeMinimumTileY(cache_entry.tileCoordinatesList)
            var delta_tiles: int = new_minimum_y - cache_entry.minimumTileY
            var delta_pixels: int = delta_tiles * ComputeShaderLayer.TILE_SIZE_PIXELS
            for vert_index: int in range(cache_entry.collisionShapePolygon.size()):
                var vpos: Vector2 = cache_entry.collisionShapePolygon[vert_index]
                vpos.y -= delta_pixels
                cache_entry.collisionShapePolygon[vert_index] = vpos
            cache_entry.minimumTileY = new_minimum_y
        if used < CollisionMask.MAX_COLLISION_SHAPES:
            CollisionMask.collision_polygon_hulls_pool[used] = cache_entry.collisionShapePolygon
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
        else:
            CollisionMask.collision_mask_polygons_pool[i].disabled = true
            CollisionMask.collision_mask_polygons_pool[i].polygon = []


func _compute_convex_hull_marching_squares(
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


func _orientation(a: Vector2, b: Vector2, c: Vector2) -> int:
    var cross_product: float = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    if cross_product > 0.0:
        return -1
    elif cross_product < 0.0:
        return 1
    else:
        return 0


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


func _calculate_tile_column_count(image_width: int, tile_size: int) -> int:
    return int((image_width + tile_size - 1) / tile_size)


func _calculate_tile_row_count(image_height: int, tile_size: int) -> int:
    return int((image_height + tile_size - 1) / tile_size)


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
