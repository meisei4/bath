extends Node2D
class_name IcebergManager

signal iceberg_cluster_formed(average_position: Vector2i)
signal request_forced_fracture(blocker_cell: Vector2i)

@export var minimum_iceberg_cluster_size: int = 4

func form_icebergs(glacier_data: GlacierData) -> void:
    var glacier_dimensions: Vector2i = GlacierUtil.get_glacier_dimensions(glacier_data)
    var visited_cells: Dictionary = {}

    for y in range(glacier_dimensions.y):
        for x in range(glacier_dimensions.x):
            var cell_position: Vector2i = Vector2i(x, y)
            if visited_cells.has(cell_position):
                continue
            if should_form_iceberg(glacier_data, cell_position):
                var iceberg_cluster: Array[Vector2i] = collect_iceberg_cluster(glacier_data, cell_position)
                mark_cells_as_visited(visited_cells, iceberg_cluster)
                if iceberg_cluster.size() >= minimum_iceberg_cluster_size:
                    form_iceberg(glacier_data, iceberg_cluster)
                    var average_position: Vector2i = calculate_average_position_of_cluster(iceberg_cluster)
                    iceberg_cluster_formed.emit(average_position)

func should_form_iceberg(glacier_data: GlacierData, cell_position: Vector2i) -> bool:
    return glacier_data.get_state(cell_position) == GlacierCellState.STATE.FRACTURED and glacier_data.get_time_in_state(cell_position) >= 1 and is_anchored_from_below(glacier_data, cell_position)

func collect_iceberg_cluster(glacier_data: GlacierData, start_cell: Vector2i) -> Array[Vector2i]:
    return GlacierUtil.collect_connected_cells(
        glacier_data,
        start_cell,
        func(target_cell: Vector2i) -> bool:
            return glacier_data.get_state(target_cell) == GlacierCellState.STATE.FRACTURED and glacier_data.get_time_in_state(target_cell) >= 1
    )

func mark_cells_as_visited(visited_cells: Dictionary, iceberg_cluster: Array[Vector2i]) -> void:
    for cell in iceberg_cluster:
        visited_cells[cell] = true

func form_iceberg(glacier_data: GlacierData, iceberg_cluster: Array[Vector2i]) -> void:
    for cell in iceberg_cluster:
        glacier_data.set_state(cell, GlacierCellState.STATE.ICEBERG)
        glacier_data.set_time_in_state(cell, 0)
        glacier_data.set_forced(cell, false)

func move_icebergs(glacier_data: GlacierData) -> void:
    var glacier_dimensions: Vector2i = GlacierUtil.get_glacier_dimensions(glacier_data)
    var visited_cells: Dictionary = {}

    for y in range(glacier_dimensions.y - 1, -1, -1):
        for x in range(glacier_dimensions.x):
            var cell_position: Vector2i = Vector2i(x, y)
            if visited_cells.has(cell_position):
                continue
            if glacier_data.get_state(cell_position) == GlacierCellState.STATE.ICEBERG and glacier_data.get_time_in_state(cell_position) >= 1:
                var iceberg_cluster: Array[Vector2i] = collect_iceberg_cluster_by_state(glacier_data, cell_position)
                mark_cells_as_visited(visited_cells, iceberg_cluster)
                move_cluster_down_one_step(glacier_data, iceberg_cluster)

func collect_iceberg_cluster_by_state(glacier_data: GlacierData, start_cell: Vector2i) -> Array[Vector2i]:
    return GlacierUtil.collect_connected_cells(
        glacier_data,
        start_cell,
        func(cluster_cell: Vector2i) -> bool:
            return glacier_data.get_state(cluster_cell) == GlacierCellState.STATE.ICEBERG
    )

# TODO 1: Add logic to ensure that blocking cells turn into FRACTURED state, and the whole cluster enters the movement phase only after waiting for at least 1 second to allow the blocking cell to become ICEBERG state.
# TODO 2: Investigate and fix issues related to iceberg clusters stopping movement when new icebergs form/merge with them, and those icebergs are blocked elsewhere (e.g. by older icebergs or fractured cells).
# TODO 3: Address the speed issue where some iceberg clusters move down at a rate of 2 cells per second, which may be too fast for the intended gameplay or mechanics.
func move_cluster_down_one_step(glacier_data: GlacierData, iceberg_cluster: Array[Vector2i]) -> bool:
    var glacier_dimensions: Vector2i = GlacierUtil.get_glacier_dimensions(glacier_data)
    var initial_cluster: Array[Vector2i] = iceberg_cluster.duplicate()

    if not try_move_cluster(glacier_data, iceberg_cluster, glacier_dimensions):
        return false

    update_cluster_position(glacier_data, iceberg_cluster, initial_cluster)
    return true

func try_move_cluster(glacier_data: GlacierData, iceberg_cluster: Array[Vector2i], glacier_dimensions: Vector2i) -> bool:
    for cell in iceberg_cluster:
        var cell_below: Vector2i = cell + Vector2i(0, 1)
        if cell_below.y >= glacier_dimensions.y:
            return false

        var state_below: int = glacier_data.get_state(cell_below)
        match state_below:
            GlacierCellState.STATE.INTACT:
                request_forced_fracture.emit(cell_below)
                return false

            GlacierCellState.STATE.FRACTURED:
                if not handle_fractured_cell(glacier_data, iceberg_cluster, cell_below):
                    return false

            GlacierCellState.STATE.ICEBERG:
                if not iceberg_cluster.has(cell_below):
                    iceberg_cluster.append(cell_below)
    return true

func handle_fractured_cell(glacier_data: GlacierData, iceberg_cluster: Array[Vector2i], cell_below: Vector2i) -> bool:
    if glacier_data.is_forced(cell_below):
        if glacier_data.get_time_in_state(cell_below) >= 1:
            unify_cluster_with_cell(glacier_data, iceberg_cluster, cell_below)
        else:
            return false
    else:
        unify_cluster_with_cell(glacier_data, iceberg_cluster, cell_below)
    return true

func unify_cluster_with_cell(glacier_data: GlacierData, iceberg_cluster: Array[Vector2i], start_cell: Vector2i) -> void:
    var additional_cells: Array[Vector2i] = GlacierUtil.collect_connected_cells(
        glacier_data,
        start_cell,
        func(p: Vector2i) -> bool:
            return glacier_data.is_forced(p)
    )
    iceberg_cluster += additional_cells

func update_cluster_position(glacier_data: GlacierData, iceberg_cluster: Array[Vector2i], initial_cluster: Array[Vector2i]) -> void:
    for cell in initial_cluster:
        glacier_data.set_state(cell, GlacierCellState.STATE.NONE)
    for i in range(iceberg_cluster.size()):
        iceberg_cluster[i] += Vector2i(0, 1)
    for cell in iceberg_cluster:
        glacier_data.set_state(cell, GlacierCellState.STATE.ICEBERG)
        if initial_cluster.has(cell):
            glacier_data.set_time_in_state(cell, 0)

func is_anchored_from_below(glacier_data: GlacierData, cell_position: Vector2i) -> bool:
    var glacier_dimensions: Vector2i = GlacierUtil.get_glacier_dimensions(glacier_data)
    if cell_position.y == glacier_dimensions.y - 1:
        return true
    var cell_below: Vector2i = cell_position + Vector2i(0, 1)
    if GlacierUtil.is_valid_cell(glacier_data, cell_below):
        var state_below: int = glacier_data.get_state(cell_below)
        return state_below == GlacierCellState.STATE.FRACTURED or state_below == GlacierCellState.STATE.ICEBERG
    return false

func calculate_average_position_of_cluster(cluster_cells: Array[Vector2i]) -> Vector2i:
    var total_position: Vector2i = Vector2i.ZERO
    for cell in cluster_cells:
        total_position += cell
    return total_position / cluster_cells.size()
