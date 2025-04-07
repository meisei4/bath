extends Node
class_name GlacierUtil


static func CELL_ABOVE(cell_position: Vector2i) -> Vector2i:
    return cell_position + GlacierConstants.UP


static func CELL_BELOW(cell_position: Vector2i) -> Vector2i:
    return cell_position + GlacierConstants.DOWN


static func CELL_LEFT(cell_position: Vector2i) -> Vector2i:
    return cell_position + GlacierConstants.LEFT


static func CELL_RIGHT(cell_position: Vector2i) -> Vector2i:
    return cell_position + GlacierConstants.RIGHT


const CELL_KEY: String = "cell"
const DEPTH_KEY: String = "depth"


static func multi_source_hydrofracture(
    glacier_data: GlacierData,
    initiation_cells: Array[Vector2i],
    max_depth: int,
    fracture_prob: float,
    fracture_callback: Callable
) -> void:
    var visited: Dictionary[Vector2i, bool] = {}
    var queue: Array[Dictionary] = []
    for cell: Vector2i in initiation_cells:
        queue.append({CELL_KEY: cell, DEPTH_KEY: 0})

    while queue.size() > 0:
        var current: Dictionary = queue.pop_front()
        var current_pos: Vector2i = current[CELL_KEY]
        var depth: int = current[DEPTH_KEY]

        if visited.has(current_pos):
            continue
        visited[current_pos] = true

        if glacier_data.IS_INTACT(current_pos):
            fracture_callback.callv([glacier_data, current_pos])
            if depth < max_depth:
                # Instead of BFS from one cell, keep going for all of them
                gather_cell_candidates_for_potential_fracturing(
                    glacier_data, current_pos, depth, fracture_prob, queue
                )


static func gather_cell_candidates_for_potential_fracturing(
    glacier_data: GlacierData,
    current_cell: Vector2i,
    current_depth: int,
    fracture_spread_probability: float,
    bfs_queue: Array[Dictionary]
) -> void:
    var cell_below: Vector2i = CELL_BELOW(current_cell)
    if (
        is_valid_glacier_cell(glacier_data, cell_below)
        and glacier_data.IS_AGED_AND_INTACT(cell_below)
    ):
        bfs_queue.append({CELL_KEY: cell_below, DEPTH_KEY: current_depth + 1})

    for neighbor: Vector2i in get_cardinal_neighbors(glacier_data, current_cell):
        if neighbor == cell_below:
            continue
        _try_append_neighbor(
            glacier_data, neighbor, current_depth, fracture_spread_probability, bfs_queue
        )


static func collect_connected_glacier_cells(
    glacier_data: GlacierData, starting_cell: Vector2i, cell_connectivity_predicate: Callable  #TODO: THIS DEFINES WHAT IT MEANS TO BE CONNECTED!!! takes a cell position and returns a boolean if its connected
) -> Array[Vector2i]:
    var visited_cells: Dictionary = {}
    var connected_cells: Array[Vector2i] = []
    var dfs_stack: Array[Vector2i] = [starting_cell]
    while dfs_stack.size() > 0:
        var current_cell: Vector2i = dfs_stack.pop_back()
        if visited_cells.has(current_cell):
            continue
        visited_cells[current_cell] = true

        if cell_connectivity_predicate.call(current_cell):
            connected_cells.append(current_cell)
            for neighbor: Vector2i in get_cardinal_neighbors(glacier_data, current_cell):
                if not visited_cells.has(neighbor):
                    dfs_stack.push_back(neighbor)
    return connected_cells


static func get_cardinal_neighbors_glacier_cell_states(
    glacier_data: GlacierData, cell_position: Vector2i
) -> Array[int]:
    var neighbor_states: Array[int] = []
    for neighbor: Vector2i in get_cardinal_neighbors(glacier_data, cell_position):
        neighbor_states.append(glacier_data.get_glacier_cell_state(neighbor))
    return neighbor_states


static func get_cardinal_neighbors(
    glacier_data: GlacierData, cell_position: Vector2i
) -> Array[Vector2i]:
    var neighbors: Array[Vector2i] = []
    #for direction in GlacierConstants.CARDINAL_DIRECTIONS:
    #var neighbor_position: Vector2i = cell_position + direction
    #if is_valid_glacier_cell(glacier_data, neighbor_position):
    #neighbors.append(neighbor_position)
    for direction: Vector2i in GlacierConstants.CARDINAL_DIRECTIONS:
        var neighbor: Vector2i
        match direction:
            GlacierConstants.UP:
                neighbor = CELL_ABOVE(cell_position)
            GlacierConstants.DOWN:
                neighbor = CELL_BELOW(cell_position)
            GlacierConstants.LEFT:
                neighbor = CELL_LEFT(cell_position)
            GlacierConstants.RIGHT:
                neighbor = CELL_RIGHT(cell_position)

        if is_valid_glacier_cell(glacier_data, neighbor):
            neighbors.append(neighbor)

    return neighbors


static func is_valid_glacier_cell(glacier_data: GlacierData, cell_position: Vector2i) -> bool:
    var glacier_dimensions: Vector2i = get_glacier_grid_dimensions(glacier_data)
    return (
        cell_position.x >= 0
        and cell_position.x < glacier_dimensions.x
        and cell_position.y >= 0
        and cell_position.y < glacier_dimensions.y
    )


static func get_glacier_grid_dimensions(glacier_data: GlacierData) -> Vector2i:
    var total_rows: int = glacier_data.glacier_cells_states.size()
    if total_rows == 0:
        return Vector2i(0, 0)
    var total_columns: int = glacier_data.glacier_cells_states[0].size()
    return Vector2i(total_columns, total_rows)


static func for_each_cell(
    glacier_data: GlacierData, callback: Callable, reverse_y: bool = false
) -> void:
    var glacier_dimensions: Vector2i = get_glacier_grid_dimensions(glacier_data)
    var y_range: Array = range(glacier_dimensions.y)
    if reverse_y:
        y_range = range(glacier_dimensions.y - 1, -1, -1)
    for y: int in y_range:
        for x: int in range(glacier_dimensions.x):
            var cell_position: Vector2i = Vector2i(x, y)
            callback.call(cell_position)


static func for_each_neighbor(glacier_data: GlacierData, callback: Callable) -> void:
    var visited_neighbors: Array = []
    for fracture: Vector2i in glacier_data.active_fractures:
        var neighbors: Array[Vector2i] = get_cardinal_neighbors(glacier_data, fracture)
        for neighbor: Vector2i in neighbors:
            if not (neighbor in visited_neighbors):
                visited_neighbors.append(neighbor)
                callback.call(neighbor)


static func _try_append_neighbor(
    glacier_data: GlacierData,
    neighbor: Vector2i,
    current_depth: int,
    fracture_spread_probability: float,
    bfs_queue: Array[Dictionary]
) -> void:
    if glacier_data.IS_AGED_AND_INTACT(neighbor) and randf() < fracture_spread_probability:
        bfs_queue.append({CELL_KEY: neighbor, DEPTH_KEY: current_depth + 1})
