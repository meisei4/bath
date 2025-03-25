extends Node2D
class_name IcebergManager

signal iceberg_cluster_formed(average_position: Vector2i)
signal force_fracture_glacier_cell(cell: Vector2i)

@export var minimum_iceberg_cluster_size: int = 4


func identify_and_form_iceberg_clusters(glacier_data: GlacierData) -> void:
    GlacierUtil.for_each_cell(glacier_data, func(cell_position) -> void:
        if glacier_data.IS_AGED_AND_FRACTURED(cell_position):
            var iceberg_cluster = GlacierUtil.collect_connected_glacier_cells(
                glacier_data, cell_position, glacier_data.IS_AGED_AND_FRACTURED
            )
            form_iceberg_cluster(glacier_data, iceberg_cluster)
    )


func form_iceberg_cluster(glacier_data: GlacierData, iceberg_cluster: Array[Vector2i]) -> void:
    if iceberg_cluster.size() >= minimum_iceberg_cluster_size:
        for iceberg_cell: Vector2i in iceberg_cluster:
            form_iceberg(glacier_data, iceberg_cell)
        iceberg_cluster_formed.emit(calculate_average_position_of_cluster(iceberg_cluster))


func form_iceberg(glacier_data: GlacierData, cell_position: Vector2i) -> void:
    glacier_data.set_glacier_cell_state(cell_position, GlacierCellState.STATE.ICEBERG)
    glacier_data.set_glacier_cells_age_in_lifecycle(cell_position, 0)


func move_icebergs(glacier_data: GlacierData) -> void:
    GlacierUtil.for_each_cell(glacier_data, func(cell_position: Vector2i) -> void:
        if glacier_data.IS_AGED_AND_ICEBERG(cell_position):
            var iceberg_cluster := GlacierUtil.collect_connected_glacier_cells(
                glacier_data, cell_position, glacier_data.IS_AGED_AND_ICEBERG # TODO: figure out if this function predicate should ever be changed in the future for connectivity
                                                                                # currently it just glues any adjacent icebergs together immediately -- kind of ugly near the end??
            )
            if can_iceberg_cluster_move_down(glacier_data, iceberg_cluster):
                update_cluster_position(glacier_data, iceberg_cluster, iceberg_cluster.duplicate())
            else:
                for cell: Vector2i in iceberg_cluster:
                    var cell_below := GlacierUtil.CELL_BELOW(cell)
                    handle_blocking_cell_below(glacier_data, cell_below)
    , true) # Reverse Y traversal

func can_iceberg_cluster_move_down(
    glacier_data: GlacierData, iceberg_cluster: Array[Vector2i]
) -> bool:
    var glacier_dimensions := GlacierUtil.get_glacier_grid_dimensions(glacier_data)
    for cell_position: Vector2i in iceberg_cluster:
        var has_reached_bottom_of_screen: bool = cell_position.y >= glacier_dimensions.y
        if has_reached_bottom_of_screen:
            return false
        var cell_below = GlacierUtil.CELL_BELOW(cell_position)
        if not GlacierUtil.is_valid_glacier_cell(glacier_data, cell_below):
            return false
        if glacier_data.IS_AGED_AND_INTACT(cell_below):
            return false
        if glacier_data.IS_AGED_AND_FRACTURED(cell_below):
            return false
        if not is_cell_open_for_iceberg_movement(glacier_data, cell_below):
            return false
    return true


func is_cell_open_for_iceberg_movement(glacier_data: GlacierData, cell_position: Vector2i) -> bool:
    return glacier_data.IS_NONE(cell_position) or glacier_data.IS_AGED_AND_ICEBERG(cell_position)


func handle_blocking_cell_below(glacier_data: GlacierData, cell_below: Vector2i) -> void:
    if not GlacierUtil.is_valid_glacier_cell(glacier_data, cell_below):
        return
    if glacier_data.IS_AGED_AND_INTACT(cell_below):
        force_fracture_glacier_cell.emit(cell_below)
    elif glacier_data.IS_AGED_AND_FRACTURED(cell_below):
        form_iceberg(glacier_data, cell_below)


func update_cluster_position(
    glacier_data: GlacierData, iceberg_cluster: Array[Vector2i], initial_cluster: Array[Vector2i]
) -> void:
    for cell_position: Vector2i in initial_cluster:
        glacier_data.set_glacier_cell_state(cell_position, GlacierCellState.STATE.NONE)
    for i: int in range(iceberg_cluster.size()):
        iceberg_cluster[i] = GlacierUtil.CELL_BELOW(iceberg_cluster[i])
    for cell_position: Vector2i in iceberg_cluster:
        form_iceberg(glacier_data, cell_position)


func calculate_average_position_of_cluster(iceberg_cluster: Array[Vector2i]) -> Vector2i:
    var total_position: Vector2i = Vector2i.ZERO
    for cell_position: Vector2i in iceberg_cluster:
        total_position += cell_position
    return total_position / iceberg_cluster.size()
