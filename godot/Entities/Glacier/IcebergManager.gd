extends Node2D
class_name IcebergManager

signal iceberg_cluster_formed(cluster_id: int, average_position: Vector2i)
signal iceberg_cluster_moved(cluster_id: int, average_position: Vector2i)
signal force_fracture_glacier_cell(cell: Vector2i)

@export var minimum_iceberg_cluster_size: int = 4
#var iceberg_clusters: Dictionary[int, Vector2] = {}  # Key: cluster id, Value: Vector2
var iceberg_cell_to_cluster_id: Dictionary[Vector2i, int] = {}
var next_iceberg_cluster_id: int = 0  # Counter to assign new IDs automatically

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
            iceberg_cell_to_cluster_id.set(iceberg_cell, next_iceberg_cluster_id)
        var iceberg_cluster_wavefront_position = calculate_iceberg_cluster_wavefront_position(iceberg_cluster)
        #iceberg_clusters.set(next_iceberg_cluster_id, iceberg_cluster_position)
        iceberg_cluster_formed.emit(next_iceberg_cluster_id, iceberg_cluster_wavefront_position)
        next_iceberg_cluster_id += 1


func form_iceberg(glacier_data: GlacierData, cell_position: Vector2i) -> void:
    glacier_data.set_glacier_cell_state(cell_position, GlacierCellState.STATE.ICEBERG)
    glacier_data.set_glacier_cells_age_in_lifecycle(cell_position, 0)


func move_icebergs(glacier_data: GlacierData) -> void:
    GlacierUtil.for_each_cell(glacier_data, func(cell_position: Vector2i) -> void:
        if glacier_data.IS_AGED_AND_ICEBERG(cell_position):
            var iceberg_cluster: Array[Vector2i] = GlacierUtil.collect_connected_glacier_cells(
                glacier_data, cell_position, glacier_data.IS_AGED_AND_ICEBERG # TODO: figure out if this function predicate should ever be changed in the future for connectivity
                                                                                # currently it just glues any adjacent icebergs together immediately -- kind of ugly near the end??
            )
            if can_iceberg_cluster_move_down(glacier_data, iceberg_cluster):
                #TODO: you need to handle this as an error ID or something  when the cell below is out of the tilemap or other
                #TODO: RACE CONDITION KEEPS HAPPENING HERE!!!!
                var iceberg_cluster_id: int = iceberg_cell_to_cluster_id.get(cell_position)
                update_cluster_position(iceberg_cluster_id, glacier_data, iceberg_cluster, iceberg_cluster.duplicate()) #HERE!!!
            else:
                for cell: Vector2i in iceberg_cluster:
                    var cell_below: Vector2i = GlacierUtil.CELL_BELOW(cell)
                    handle_blocking_cell_below(glacier_data, cell_below)
    , true) # Reverse Y traversal

func can_iceberg_cluster_move_down(
    glacier_data: GlacierData, iceberg_cluster: Array[Vector2i]
) -> bool:
    var glacier_dimensions: Vector2i = GlacierUtil.get_glacier_grid_dimensions(glacier_data)
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
    cluster_id: int, glacier_data: GlacierData, iceberg_cluster: Array[Vector2i], initial_cluster: Array[Vector2i]
) -> void:
    for cell_position: Vector2i in initial_cluster:
        glacier_data.set_glacier_cell_state(cell_position, GlacierCellState.STATE.NONE)
        #TODO: dear lord still gross
        iceberg_cell_to_cluster_id.erase(cell_position)
    for i: int in range(iceberg_cluster.size()):
        iceberg_cluster[i] = GlacierUtil.CELL_BELOW(iceberg_cluster[i])
    for cell_position: Vector2i in iceberg_cluster:
        form_iceberg(glacier_data, cell_position)
        #TODO: dear lord double gross
        iceberg_cell_to_cluster_id.set(cell_position, cluster_id)
    iceberg_cluster_moved.emit(cluster_id, calculate_iceberg_cluster_wavefront_position(iceberg_cluster))


func calculate_iceberg_cluster_wavefront_position(iceberg_cluster: Array[Vector2i]) -> Vector2:
    var min_x = INF
    var max_x = -INF
    var max_y = -INF
    for cell in iceberg_cluster:
        if cell.x < min_x:
            min_x = cell.x
        if cell.x > max_x:
            max_x = cell.x
        if cell.y > max_y:
            max_y = cell.y
    var horizontal_center = (min_x + max_x) / 2.0
    return Vector2(horizontal_center, max_y)
