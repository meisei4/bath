extends Node2D
class_name HydrofractureManager

signal fracture_propagated(position: Vector2i)

var max_fracture_depth: int = 5
var fracturing_coefficient: float = 0.5

func run_cycle(glacier_data: GlacierData, fractures_per_cycle: int) -> int:
    var fracture_zones: Array[Vector2i] = find_fracture_zones(glacier_data)
    if fracture_zones.is_empty():
        return 0

    fracture_zones.shuffle()
    var selected_fracture_starts: Array[Vector2i] = fracture_zones.slice(0, min(fractures_per_cycle, fracture_zones.size()))

    for fracture_start: Vector2i in selected_fracture_starts:
        propagate_fracture(glacier_data, fracture_start)
    return selected_fracture_starts.size()

func find_fracture_zones(glacier_data: GlacierData) -> Array[Vector2i]:
    var dims: Vector2i = glacier_data.get_dimensions()
    var fracture_zones: Array[Vector2i] = []

    for y in range(dims.y):
        for x in range(dims.x):
            var pos: Vector2i = Vector2i(x, y)
            var current_state: int = glacier_data.get_state(pos)

            # Only consider cells that could potentially fracture
            if current_state not in [GlacierCellState.STATE.INTACT, GlacierCellState.STATE.FRACTURED]:
                continue

            # Check all 4 neighbors (left, right, up, down)
            var offsets = [Vector2i(-1, 0), Vector2i(1, 0), Vector2i(0, -1), Vector2i(0, 1)]
            for offset in offsets:
                var neighbor: Vector2i = pos + offset
                # Skip out-of-bounds neighbors
                if not glacier_data.is_valid_cell(neighbor):
                    continue
                # If a valid neighbor is NONE, this cell is on a boundary
                if glacier_data.get_state(neighbor) == GlacierCellState.STATE.NONE:
                    fracture_zones.append(pos)
                    break  # Already tagged it, so move on to the next cell
    return fracture_zones


func is_fracture_zone(cell_position: Vector2i, glacier_data: GlacierData) -> bool:
    var current_state: int = glacier_data.get_state(cell_position)
    if current_state not in [GlacierCellState.STATE.INTACT, GlacierCellState.STATE.FRACTURED]:
        return false

    # For testing, we force the bottom row to be a fracture zone.
    var dims: Vector2i = glacier_data.get_dimensions()
    if cell_position.y == dims.y - 1:
        return true

    var surrounding_cells: Array[Vector2i] = glacier_data.get_surrounding_cells(cell_position)
    for cell: Vector2i in surrounding_cells:
        if glacier_data.get_state(cell) == GlacierCellState.STATE.NONE:
            return true
    return false

func propagate_fracture(glacier_data: GlacierData, cell_position: Vector2i, current_depth: int = 0) -> void:
    if current_depth > max_fracture_depth:
        return

    if glacier_data.get_state(cell_position) != GlacierCellState.STATE.INTACT:
        return

    glacier_data.set_state(cell_position, GlacierCellState.STATE.FRACTURED)
    print("FRACTURE OCCURED AT: ", cell_position)

    fracture_propagated.emit(cell_position)

    var surrounding_cells: Array[Vector2i] = glacier_data.get_surrounding_cells(cell_position)
    for cell: Vector2i in surrounding_cells:
        if randf() < fracturing_coefficient and glacier_data.get_state(cell) == GlacierCellState.STATE.INTACT:
            propagate_fracture(glacier_data, cell, current_depth + 1)
