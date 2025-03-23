extends Node
class_name GlacierUtil

static func get_glacier_dimensions(glacier_data: GlacierData) -> Vector2i:
    var height: int = glacier_data.cell_state_grid.size()
    if height == 0:
        return Vector2i(0, 0)
    var width: int = glacier_data.cell_state_grid[0].size()
    return Vector2i(width, height)

static func is_valid_cell(glacier_data: GlacierData, cell_position: Vector2i) -> bool:
    var glacier_dimensions: Vector2i = get_glacier_dimensions(glacier_data)
    return cell_position.x >= 0 and cell_position.x < glacier_dimensions.x and cell_position.y >= 0 and cell_position.y < glacier_dimensions.y

static func get_adjacent_neighbors(glacier_data: GlacierData, cell_position: Vector2i) -> Array[Vector2i]:
    var neighboring_cells: Array[Vector2i] = []
    var directions: Array[Vector2i] = [Vector2i(-1, 0), Vector2i(1, 0), Vector2i(0, -1), Vector2i(0, 1)]
    for direction in directions:
        var neighbor_cell: Vector2i = cell_position + direction
        if is_valid_cell(glacier_data, neighbor_cell):
            neighboring_cells.append(neighbor_cell)
    return neighboring_cells

static func collect_connected_cells(glacier_data: GlacierData, start_cell: Vector2i, can_traverse: Callable) -> Array[Vector2i]:
    var visited_cells: Dictionary = {}
    var cluster_cells: Array[Vector2i] = []
    var cell_stack: Array[Vector2i] = [start_cell]

    while cell_stack.size() > 0:
        var cell_position: Vector2i = cell_stack.pop_back()
        if visited_cells.has(cell_position):
            continue
        visited_cells[cell_position] = true
        if can_traverse.call(cell_position):
            cluster_cells.append(cell_position)
            var neighboring_cells: Array[Vector2i] = get_adjacent_neighbors(glacier_data, cell_position)
            for neighbor_cell in neighboring_cells:
                if not visited_cells.has(neighbor_cell):
                    cell_stack.push_back(neighbor_cell)
    return cluster_cells

static func propagate_fracture_bfs(
    glacier_data: GlacierData,
    start_cell: Vector2i,
    max_depth: int,
    fracture_probability: float,
    on_visit: Callable
) -> void:
    var visited_cells: Dictionary = {}
    var cell_queue: Array = []
    var initial_queue_item: Dictionary = { "cell": start_cell, "depth": 0 }
    cell_queue.append(initial_queue_item)

    while cell_queue.size() > 0:
        var queue_item: Dictionary = cell_queue.pop_front()
        var cell_position: Vector2i = queue_item["cell"]
        var current_depth: int = queue_item["depth"]

        if visited_cells.has(cell_position):
            continue
        visited_cells[cell_position] = true

        if glacier_data.get_state(cell_position) == GlacierCellState.STATE.INTACT:
            on_visit.callv([cell_position, current_depth])

            if current_depth < max_depth:
                process_neighbors_for_fracture(glacier_data, cell_position, current_depth, fracture_probability, cell_queue)

static func process_neighbors_for_fracture(
    glacier_data: GlacierData,
    cell_position: Vector2i,
    current_depth: int,
    fracture_probability: float,
    cell_queue: Array
) -> void:
    var cell_below: Vector2i = cell_position + Vector2i(0, 1)
    if is_valid_cell(glacier_data, cell_below) and glacier_data.get_state(cell_below) == GlacierCellState.STATE.INTACT:
        cell_queue.append({ "cell": cell_below, "depth": current_depth + 1 })

    var neighboring_cells: Array[Vector2i] = get_adjacent_neighbors(glacier_data, cell_position)
    for neighbor_cell in neighboring_cells:
        if neighbor_cell == cell_below:
            continue
        if glacier_data.get_state(neighbor_cell) == GlacierCellState.STATE.INTACT and randf() < fracture_probability:
            cell_queue.append({ "cell": neighbor_cell, "depth": current_depth + 1 })
