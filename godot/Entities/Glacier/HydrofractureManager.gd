extends Node2D
class_name HydrofractureManager

signal cell_fractured(cell_position: Vector2i)

@export var max_fracture_depth: int = 5
@export var fracture_propagation_probability: float = 0.4
@export var max_new_fractures_per_cycle: int = 2

var cells_fractured_in_previous_cycle: Array[Vector2i] = []
var cells_newly_fractured_this_cycle: Array[Vector2i] = []

func run_fracture_phase(glacier_data: GlacierData) -> void:
    cells_newly_fractured_this_cycle.clear()
    var fracture_candidates = get_exposed_intact_candidates(glacier_data)
    fracture_candidates.shuffle()
    if fracture_candidates.is_empty():
        return

    var selected_starts = select_fracture_starts(fracture_candidates)
    fracture_cells(glacier_data, selected_starts)
    cells_fractured_in_previous_cycle = cells_newly_fractured_this_cycle.duplicate()
    cells_newly_fractured_this_cycle.clear()

func select_fracture_starts(fracture_candidates: Array[Vector2i]) -> Array[Vector2i]:
    return fracture_candidates.slice(0, min(max_new_fractures_per_cycle, fracture_candidates.size()))

func fracture_cells(glacier_data: GlacierData, selected_starts: Array[Vector2i]) -> void:
    for start_cell in selected_starts:
        GlacierUtil.propagate_fracture_bfs(
            glacier_data,
            start_cell,
            max_fracture_depth,
            fracture_propagation_probability,
            func(fracture_cell_position: Vector2i, depth: int) -> void:
                fracture_cell(glacier_data, fracture_cell_position, depth)
        )

func get_exposed_intact_candidates(glacier_data: GlacierData) -> Array[Vector2i]:
    var dims = GlacierUtil.get_glacier_dimensions(glacier_data)
    var candidates: Array[Vector2i] = []
    for y in range(dims.y):
        for x in range(dims.x):
            var cell_position = Vector2i(x, y)
            if can_fracture_cell(glacier_data, cell_position):
                candidates.append(cell_position)

    return candidates

func can_fracture_cell(glacier_data: GlacierData, cell_position: Vector2i) -> bool:
    return glacier_data.get_state(cell_position) == GlacierCellState.STATE.INTACT and is_exposed_or_bottom_edge(glacier_data, cell_position)

func is_exposed_or_bottom_edge(glacier_data: GlacierData, cell_position: Vector2i) -> bool:
    var dims = GlacierUtil.get_glacier_dimensions(glacier_data)
    if cell_position.y == dims.y - 1:
        return true
    return has_exposed_neighbor(glacier_data, cell_position)

func has_exposed_neighbor(glacier_data: GlacierData, cell_position: Vector2i) -> bool:
    for neighbor in GlacierUtil.get_adjacent_neighbors(glacier_data, cell_position):
        if glacier_data.get_state(neighbor) == GlacierCellState.STATE.NONE:
            return true
    return false

func fracture_cell(glacier_data: GlacierData, cell_position: Vector2i, depth: int) -> void:
    if glacier_data.get_state(cell_position) == GlacierCellState.STATE.INTACT:
        glacier_data.set_state(cell_position, GlacierCellState.STATE.FRACTURED)
        glacier_data.set_time_in_state(cell_position, 0)
        cells_newly_fractured_this_cycle.append(cell_position)
        emit_cell_fractured_signal(cell_position)

func emit_cell_fractured_signal(cell_position: Vector2i) -> void:
    cell_fractured.emit(cell_position)

func force_fracture_cell(glacier_data: GlacierData, cell_position: Vector2i) -> void:
    if glacier_data.get_state(cell_position) == GlacierCellState.STATE.INTACT:
        glacier_data.set_state(cell_position, GlacierCellState.STATE.FRACTURED)
        glacier_data.set_time_in_state(cell_position, 0)
        glacier_data.set_forced(cell_position, true)
