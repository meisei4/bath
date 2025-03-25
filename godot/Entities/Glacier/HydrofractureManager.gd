extends Node2D
class_name HydrofractureManager

signal cell_fractured(cell_position: Vector2i)

@export var maximum_fracture_depth: int = 5
@export var fracture_propagation_probability: float = 0.4
@export var maximum_new_fractures_per_cycle: int = 2


func run_hydrofracture_cycle(glacier_data: GlacierData) -> void:
    var hydrofracture_initiation_candidates: Array[Vector2i] = gather_hydrofracture_initiation_candidates(
        glacier_data
    )
    hydrofracture_initiation_candidates.shuffle()
    var hydrofracture_initiation_cells: Array[Vector2i] = reduce_hydrofracture_candidates_for_current_cycle(
        hydrofracture_initiation_candidates
    )
    propagate_fractures(glacier_data, hydrofracture_initiation_cells)


func gather_hydrofracture_initiation_candidates(glacier_data: GlacierData) -> Array[Vector2i]:
    var candidate_cells: Array[Vector2i] = []
    GlacierUtil.for_each_cell(
        glacier_data,
        func(cell_position) -> void:
            if cell_is_eligible_for_hydrofracture(glacier_data, cell_position):
                candidate_cells.append(cell_position)
    )
    return candidate_cells


func cell_is_eligible_for_hydrofracture(glacier_data: GlacierData, cell_position: Vector2i) -> bool:
    var is_touching_none_galcier_region: bool = (
        GlacierCellState.STATE.NONE
        in GlacierUtil.get_cardinal_neighbors_glacier_cell_states(glacier_data, cell_position)
    )
    return glacier_data.IS_AGED_AND_INTACT(cell_position) and is_touching_none_galcier_region


func reduce_hydrofracture_candidates_for_current_cycle(
    fracture_candidates: Array[Vector2i]
) -> Array[Vector2i]:
    return fracture_candidates.slice(
        0, min(maximum_new_fractures_per_cycle, fracture_candidates.size())
    )


func propagate_fractures(
    glacier_data: GlacierData, hydrofracture_initiation_cells: Array[Vector2i]
) -> void:
    for cell_position: Vector2i in hydrofracture_initiation_cells:
        GlacierUtil.propagate_hydrofracture_using_bfs(
            glacier_data,
            cell_position,
            maximum_fracture_depth,
            fracture_propagation_probability,
            self.fracture_glacier_cell  #TODO: figure out if this will ever need to be a different function???
        )


func fracture_glacier_cell(glacier_data: GlacierData, cell_position: Vector2i) -> void:
    glacier_data.set_glacier_cell_state(cell_position, GlacierCellState.STATE.FRACTURED)
    glacier_data.set_glacier_cells_age_in_lifecycle(cell_position, 0)
    cell_fractured.emit(cell_position)
