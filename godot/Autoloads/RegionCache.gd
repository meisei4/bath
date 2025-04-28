extends Node
#class_name RegionCache

class RegionCacheEntry:
    var tileCoordinatesList: PackedVector2Array = PackedVector2Array()
    var centroidPosition: Vector2 = Vector2()
    var minimumTileY: int = 0
    var collisionShapePolygon: PackedVector2Array = PackedVector2Array()


var region_cache_identifier_list: Array[int] = []
var region_cache_entry_list: Array[RegionCacheEntry] = []
var next_region_identifier: int = 0


func matchAndCacheRegions(regionTileLists: Array[PackedVector2Array]) -> Array[int]:
    var new_identifier_list: Array[int] = []
    var new_entry_list: Array[RegionCacheEntry] = []
    var assigned_identifiers: Array[int] = []

    for regionTiles: PackedVector2Array in regionTileLists:
        var sumOfPositions: Vector2 = Vector2(0, 0)
        for tilePos: Vector2 in regionTiles:
            sumOfPositions += tilePos
        var centroidPos: Vector2 = sumOfPositions / float(regionTiles.size())

        var bestMatchIdentifier: int = -1
        var bestMatchDistance: float = 1e9
        for cacheIndex: int in range(region_cache_identifier_list.size()):
            var existingIdentifier: int = region_cache_identifier_list[cacheIndex]
            var existingEntry: RegionCacheEntry = region_cache_entry_list[cacheIndex]
            var distance: float = existingEntry.centroidPosition.distance_to(centroidPos)
            if distance < 5.0 and distance < bestMatchDistance:
                bestMatchDistance = distance
                bestMatchIdentifier = existingIdentifier

        if bestMatchIdentifier != -1:
            var existingIndex: int = region_cache_identifier_list.find(bestMatchIdentifier)
            var entryToUpdate: RegionCacheEntry = region_cache_entry_list[existingIndex]
            entryToUpdate.tileCoordinatesList = regionTiles
            entryToUpdate.centroidPosition = centroidPos

            new_identifier_list.append(bestMatchIdentifier)
            new_entry_list.append(entryToUpdate)
            assigned_identifiers.append(bestMatchIdentifier)
        else:
            var newIdentifier: int = next_region_identifier
            next_region_identifier += 1

            var newEntry: RegionCacheEntry = RegionCacheEntry.new()
            newEntry.tileCoordinatesList = regionTiles
            newEntry.centroidPosition = centroidPos
            newEntry.minimumTileY = computeMinimumTileY(regionTiles)
            newEntry.collisionShapePolygon = PackedVector2Array()

            new_identifier_list.append(newIdentifier)
            new_entry_list.append(newEntry)
            assigned_identifiers.append(newIdentifier)

    region_cache_identifier_list = new_identifier_list
    region_cache_entry_list = new_entry_list

    return assigned_identifiers


func computeMinimumTileY(regionTiles: PackedVector2Array) -> int:
    var minimumY: int = int(1e9)
    for tilePos: Vector2 in regionTiles:
        minimumY = min(minimumY, int(tilePos.y))
    return minimumY
