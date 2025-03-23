extends Node
class_name GlacierUtil

class BFSItem:
    var cell_position: Vector2i
    var depth: int

static func get_dimensions(glacier_data: GlacierData) -> Vector2i:
    var height: int = glacier_data.cell_state_grid.size()
    if height == 0:
        return Vector2i(0, 0)
    var width: int = glacier_data.cell_state_grid[0].size()
    return Vector2i(width, height)


static func is_valid_cell(glacier_data: GlacierData, cell_position: Vector2i) -> bool:
    var glacier_dimensions: Vector2i = get_dimensions(glacier_data)
    if cell_position.x < 0 or cell_position.x >= glacier_dimensions.x:
        return false
    if cell_position.y < 0 or cell_position.y >= glacier_dimensions.y:
        return false
    return true


static func get_orthogonal_neighbors(glacier_data: GlacierData, cell_position: Vector2i) -> Array[Vector2i]:
    var neighboring_cells: Array[Vector2i] = []
    var directions: Array[Vector2i] = [
        Vector2i(-1, 0),
        Vector2i(1, 0),
        Vector2i(0, -1),
        Vector2i(0, 1)
    ]
    for direction in directions:
        var neighbor_cell_position: Vector2i = cell_position + direction
        if is_valid_cell(glacier_data, neighbor_cell_position):
            neighboring_cells.append(neighbor_cell_position)
    return neighboring_cells


static func flood_fill(
    glacier_data: GlacierData,
    start_cell_position: Vector2i,
    is_traversable: Callable
) -> Array[Vector2i]:
    var visited: Dictionary[Vector2i, bool] = {}
    var connected_cells: Array[Vector2i] = []
    var stack: Array[Vector2i] = [start_cell_position]

    while stack.size() > 0:
        var current_cell: Vector2i = stack.pop_back()
        if visited.has(current_cell):
            continue
        visited[current_cell] = true

        if is_traversable.call(current_cell):
            connected_cells.append(current_cell)

            var neighboring_cells: Array[Vector2i] = get_orthogonal_neighbors(glacier_data, current_cell)
            for neighbor_cell in neighboring_cells:
                if not visited.has(neighbor_cell):
                    stack.append(neighbor_cell)

    return connected_cells


static func flood_fill_fracture_bfs(
    glacier_data: GlacierData,
    start_cell_position: Vector2i,
    max_depth: int,
    fracture_probability: float,
    on_visitfracture: Callable
) -> void:
    var visited: Dictionary[Vector2i, bool] = {}
    var queue: Array[BFSItem] = []

    var first_item := BFSItem.new()
    first_item.cell_position = start_cell_position
    first_item.depth = 0
    queue.append(first_item)

    while queue.size() > 0:
        var queue_item: BFSItem = queue.pop_front()
        var current_cell_position: Vector2i = queue_item.cell_position
        var current_depth: int = queue_item.depth

        if visited.has(current_cell_position):
            continue
        visited[current_cell_position] = true

        if glacier_data.get_state(current_cell_position) == GlacierCellState.STATE.INTACT:
            on_visitfracture.callv([current_cell_position, current_depth])

            if current_depth < max_depth:
                var below: Vector2i = current_cell_position + Vector2i(0, 1)
                if is_valid_cell(glacier_data, below):
                    if glacier_data.get_state(below) == GlacierCellState.STATE.INTACT:
                        var down_item := BFSItem.new()
                        down_item.cell_position = below
                        down_item.depth = current_depth + 1
                        queue.append(down_item)

                var neighboring_cells: Array[Vector2i] = get_orthogonal_neighbors(glacier_data, current_cell_position)
                for neighbor_cell in neighboring_cells:
                    if neighbor_cell == below:
                        continue
                    if glacier_data.get_state(neighbor_cell) == GlacierCellState.STATE.INTACT:
                        if randf() < fracture_probability:
                            var side_item := BFSItem.new()
                            side_item.cell_position = neighbor_cell
                            side_item.depth = current_depth + 1
                            queue.append(side_item)
