extends Node
class_name GlacierData

var mass_distribution: Array = []

func initialize_from_tilemap(tilemap: TileMapLayer) -> void:
    # Get the bounding rect of all used cells in the tilemap.
    var used_rect = tilemap.get_used_rect()

    # If you just want to handle a known width/height (e.g., 16×24),
    # you can skip used_rect and just loop over 0..width, 0..height
    # if you always fill them, but used_rect is more general.
    var width: int = used_rect.size.x
    var height: int = used_rect.size.y

    # Clear and resize mass_distribution to match the tilemap’s bounding rect
    mass_distribution.clear()
    mass_distribution.resize(height)
    for y in range(height):
        mass_distribution[y] = []

    # Fill the mass_distribution from the tilemap's atlas coords
    for rel_y in range(height):
        for rel_x in range(width):
            var cell_x: int = used_rect.position.x + rel_x
            var cell_y: int = used_rect.position.y + rel_y
            var cell_position: Vector2i = Vector2i(cell_x, cell_y)

            # Read the atlas coordinate from the tilemap
            var atlas_coords: Vector2i = tilemap.get_cell_atlas_coords(cell_position)
            # By your design, atlas_coords.y corresponds to GlacierCellState.STATE
            if atlas_coords == Vector2i(-1, -1):
                # Means no tile is set at this position
                mass_distribution[rel_y].append(GlacierCellState.STATE.NONE)
            else:
                # Use the y coordinate of the atlas as the cell's state
                # If your tile definitions are 0=NONE,1=INTACT,2=FRACTURED,3=ICEBERG, etc.
                mass_distribution[rel_y].append(atlas_coords.y)


func get_state(cell_position: Vector2i) -> int:
    if is_valid_cell(cell_position):
        return mass_distribution[cell_position.y][cell_position.x]
    return GlacierCellState.STATE.NONE

func set_state(cell_position: Vector2i, state: int) -> void:
    if is_valid_cell(cell_position):
        mass_distribution[cell_position.y][cell_position.x] = state

func is_valid_cell(cell_position: Vector2i) -> bool:
    var x: int = cell_position.x
    var y: int = cell_position.y
    return (y >= 0 and y < mass_distribution.size() and x >= 0 and x < mass_distribution[y].size())

func get_dimensions() -> Vector2i:
    var height: int = mass_distribution.size()
    var width: int = mass_distribution[0].size() if height > 0 else 0
    return Vector2i(width, height)

func get_surrounding_cells(cell_position: Vector2i) -> Array[Vector2i]:
    var neighbors: Array[Vector2i] = []
    var offsets: Array[Vector2i] = [Vector2i(-1, 0), Vector2i(1, 0), Vector2i(0, -1), Vector2i(0, 1)]
    for offset: Vector2i in offsets:
        var neighbor: Vector2i = cell_position + offset
        if is_valid_cell(neighbor):
            neighbors.append(neighbor)
    return neighbors
