extends Node
class_name GlacierUtil

const UP: Vector2i = Vector2i(0, -1)
const DOWN: Vector2i = Vector2i(0, 1)
const LEFT: Vector2i = Vector2i(-1, 0)
const RIGHT: Vector2i = Vector2i(1, 0)

const CARDINAL_DIRECTIONS: Array[Vector2i] = [LEFT, RIGHT, UP, DOWN]


static func CELL_ABOVE(cell_position: Vector2i) -> Vector2i:
    return cell_position + UP


static func CELL_BELOW(cell_position: Vector2i) -> Vector2i:
    return cell_position + DOWN


static func CELL_LEFT(cell_position: Vector2i) -> Vector2i:
    return cell_position + LEFT


static func CELL_RIGHT(cell_position: Vector2i) -> Vector2i:
    return cell_position + RIGHT


const CELL_KEY: String = "cell"
const DEPTH_KEY: String = "depth"


static func propagate_hydrofracture_using_bfs(
    glacier_data: GlacierData,
    starting_cell: Vector2i,
    maximum_fracture_depth: int,
    fracture_spread_probability: float,
    on_fracture_callback: Callable
) -> void:
    # use Breadth-First Search (BFS) to simulate the propagation of a hydrofracture from a starting cell
    var visited_cells: Dictionary[Vector2i, bool] = {}
    var bfs_queue: Array[Dictionary] = [{CELL_KEY: starting_cell, DEPTH_KEY: 0}]
    while bfs_queue.size() > 0:
        var current_element: Dictionary = bfs_queue.pop_front()
        var current_cell: Vector2i = current_element[CELL_KEY]
        var current_depth: int = current_element[DEPTH_KEY]
        if visited_cells.has(current_cell):
            continue
        visited_cells[current_cell] = true
        if glacier_data.IS_INTACT(current_cell):
            on_fracture_callback.callv([glacier_data, current_cell])
            if current_depth < maximum_fracture_depth:
                gather_cell_candidates_for_potential_fracturing(
                    glacier_data,
                    current_cell,
                    current_depth,
                    fracture_spread_probability,
                    bfs_queue
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
        if glacier_data.IS_AGED_AND_INTACT(neighbor) and randf() < fracture_spread_probability:
            bfs_queue.append({CELL_KEY: neighbor, DEPTH_KEY: current_depth + 1})


static func collect_connected_glacier_cells(
    glacier_data: GlacierData, starting_cell: Vector2i, cell_connectivity_predicate: Callable  #TODO: THIS DEFINES WHAT IT MEANS TO BE CONNECTED!!! takes a cell position and returns a boolean if its connected
) -> Array[Vector2i]:
    # use Depth-First Search (DFS) to collect all connected cells that meet a certain cell_connectivity_predicate.
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
    #for direction in CARDINAL_DIRECTIONS:
    #var neighbor_position: Vector2i = cell_position + direction
    #if is_valid_glacier_cell(glacier_data, neighbor_position):
    #neighbors.append(neighbor_position)
    for direction: Vector2i in CARDINAL_DIRECTIONS:
        var neighbor: Vector2i
        match direction:
            UP:
                neighbor = CELL_ABOVE(cell_position)
            DOWN:
                neighbor = CELL_BELOW(cell_position)
            LEFT:
                neighbor = CELL_LEFT(cell_position)
            RIGHT:
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
    var y_range = range(glacier_dimensions.y)
    if reverse_y:
        y_range = range(glacier_dimensions.y - 1, -1, -1)
    for y in y_range:
        for x in range(glacier_dimensions.x):
            var cell_position: Vector2i = Vector2i(x, y)
            callback.call(cell_position)
