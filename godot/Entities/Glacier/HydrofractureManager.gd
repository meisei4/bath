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

    var fracture_candidates: Array[Vector2i] = get_exposed_intact_candidates(glacier_data)
    fracture_candidates.shuffle()

    if fracture_candidates.is_empty():
        print("[HydrofractureManager] No fracture candidates this cycle.")
        return

    var selected_starting_cells: Array[Vector2i] = fracture_candidates.slice(
        0,
        min(max_new_fractures_per_cycle, fracture_candidates.size())
    )

    print("[HydrofractureManager] Fracture candidates:", fracture_candidates)
    print("[HydrofractureManager] Selected starts:", selected_starting_cells)

    for starting_cell in selected_starting_cells:
        GlacierUtil.flood_fill_fracture_bfs(
            glacier_data,
            starting_cell,
            max_fracture_depth,
            fracture_propagation_probability,
            func(fracture_cell_position: Vector2i, depth: int) -> void:
                fracture_cell(glacier_data, fracture_cell_position, depth)
        )

    if cells_newly_fractured_this_cycle.size() > 0:
        print("[HydrofractureManager] Newly fractured:", cells_newly_fractured_this_cycle)
    else:
        print("[HydrofractureManager] No new fractures occurred.")


func finalize_fracture_cycle() -> void:
    cells_fractured_in_previous_cycle = cells_newly_fractured_this_cycle.duplicate()
    cells_newly_fractured_this_cycle.clear()
    print("[HydrofractureManager] End of fracture cycle => cells_fractured_in_previous_cycle:",
        cells_fractured_in_previous_cycle)


func get_cells_fractured_in_previous_cycle() -> Array[Vector2i]:
    return cells_fractured_in_previous_cycle


func get_exposed_intact_candidates(glacier_data: GlacierData) -> Array[Vector2i]:
    var glacier_dimensions: Vector2i = GlacierUtil.get_dimensions(glacier_data)
    var candidate_cells: Array[Vector2i] = []

    for y in range(glacier_dimensions.y):
        for x in range(glacier_dimensions.x):
            var cell_position: Vector2i = Vector2i(x, y)
            var current_state: int = glacier_data.get_state(cell_position)
            if current_state != GlacierCellState.STATE.INTACT:
                continue
            if is_exposed_or_bottom_edge(glacier_data, cell_position):
                candidate_cells.append(cell_position)
    return candidate_cells


func is_exposed_or_bottom_edge(glacier_data: GlacierData, cell_position: Vector2i) -> bool:
    var glacier_dimensions: Vector2i = GlacierUtil.get_dimensions(glacier_data)
    if cell_position.y == glacier_dimensions.y - 1:
        return true

    var neighboring_cells: Array[Vector2i] = GlacierUtil.get_orthogonal_neighbors(glacier_data, cell_position)
    for neighbor_position in neighboring_cells:
        if glacier_data.get_state(neighbor_position) == GlacierCellState.STATE.NONE:
            return true

    return false


func fracture_cell(glacier_data: GlacierData, cell_position: Vector2i, depth: int) -> void:
    if glacier_data.get_state(cell_position) == GlacierCellState.STATE.INTACT:
        glacier_data.set_state(cell_position, GlacierCellState.STATE.FRACTURED)
        cells_newly_fractured_this_cycle.append(cell_position)
        cell_fractured.emit(cell_position)
        print("[HydrofractureManager] FRACTURED at:", cell_position, " depth=", depth)


func force_fracture_cell(glacier_data: GlacierData, cell_position: Vector2i) -> void:
    if glacier_data.get_state(cell_position) == GlacierCellState.STATE.INTACT:
        glacier_data.set_state(cell_position, GlacierCellState.STATE.FRACTURED)
        print("[HydrofractureManager] Force‐fractured cell at:", cell_position)
