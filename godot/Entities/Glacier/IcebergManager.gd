extends Node2D
class_name IcebergManager

signal iceberg_cluster_formed(average_position: Vector2i)
signal tilemap_update_requested()
signal request_forced_fracture(blocker_cell: Vector2i)

@export var minimum_iceberg_cluster_size: int = 4
@export var max_icebergs_to_form_per_cycle: int = 10

var active_moving_iceberg_cluster: Array[Vector2i] = []

func iceberg_formation_phase(glacier_data: GlacierData, fractured_cells: Array[Vector2i]) -> int:
    var glacier_dimensions: Vector2i = GlacierUtil.get_dimensions(glacier_data)
    var visited: Dictionary[Vector2i, bool] = {}
    var new_icebergs_formed: int = 0

    print("[IcebergManager] Global scan for FRACTURED => ICEBERG conversion.")
    for y in range(glacier_dimensions.y):
        for x in range(glacier_dimensions.x):
            var current_cell_position: Vector2i = Vector2i(x, y)
            if visited.has(current_cell_position):
                continue

            # Only consider FRACTURED cells that have been in that state for at least 1 second.
            if glacier_data.get_state(current_cell_position) == GlacierCellState.STATE.FRACTURED and glacier_data.get_time_in_state(current_cell_position) >= 1:
                if is_anchored_from_below(glacier_data, current_cell_position):
                    var fractured_cells_cluster: Array[Vector2i] = GlacierUtil.flood_fill(
                        glacier_data,
                        current_cell_position,
                        func(pos: Vector2i) -> bool:
                            return glacier_data.get_state(pos) == GlacierCellState.STATE.FRACTURED
                    )

                    for cell_position_in_cluster in fractured_cells_cluster:
                        visited[cell_position_in_cluster] = true

                    if fractured_cells_cluster.size() >= minimum_iceberg_cluster_size:
                        for cell_position_in_cluster in fractured_cells_cluster:
                            glacier_data.set_state(cell_position_in_cluster, GlacierCellState.STATE.ICEBERG)
                            glacier_data.set_time_in_state(cell_position_in_cluster, 0)  # reset timer when converting
                            glacier_data.set_forced(cell_position_in_cluster, false)      # clear forced flag
                        var average_position: Vector2i = calculate_average_position_of_cluster(fractured_cells_cluster)
                        iceberg_cluster_formed.emit(average_position)
                        print("[IcebergManager] Converted cluster of size:", fractured_cells_cluster.size(), "to ICEBERG.")
                        new_icebergs_formed += 1
                        if new_icebergs_formed >= max_icebergs_to_form_per_cycle:
                            break
                else:
                    visited[current_cell_position] = true

        if new_icebergs_formed >= max_icebergs_to_form_per_cycle:
            break

    if new_icebergs_formed > 0:
        print("[IcebergManager] Formed", new_icebergs_formed, "new iceberg(s).")
    else:
        print("[IcebergManager] No new icebergs formed.")
    return new_icebergs_formed

func move_icebergs(glacier_data: GlacierData) -> void:
    print("\n[IcebergManager] -- MOVE ICEBERGS PHASE --")
    var glacier_dimensions: Vector2i = GlacierUtil.get_dimensions(glacier_data)
    var visited: Dictionary[Vector2i, bool] = {}
    var number_of_clusters_found: int = 0
    var number_of_clusters_moved: int = 0

    for y in range(glacier_dimensions.y - 1, -1, -1):
        for x in range(glacier_dimensions.x):
            var current_cell_position: Vector2i = Vector2i(x, y)
            if visited.has(current_cell_position):
                continue
            if glacier_data.get_state(current_cell_position) == GlacierCellState.STATE.ICEBERG and glacier_data.get_time_in_state(current_cell_position) >= 1:
                number_of_clusters_found += 1

                var iceberg_cells_cluster: Array[Vector2i] = GlacierUtil.flood_fill(
                    glacier_data,
                    current_cell_position,
                    func(cluster_cell: Vector2i) -> bool:
                        return glacier_data.get_state(cluster_cell) == GlacierCellState.STATE.ICEBERG
                )

                for cell_position_in_cluster in iceberg_cells_cluster:
                    visited[cell_position_in_cluster] = true

                print("[IcebergManager] Found iceberg cluster #", number_of_clusters_found, ":", iceberg_cells_cluster)
                active_moving_iceberg_cluster = iceberg_cells_cluster.duplicate()
                if move_cluster_down_one_step(glacier_data, active_moving_iceberg_cluster):
                    number_of_clusters_moved += 1
                active_moving_iceberg_cluster.clear()
    if number_of_clusters_moved > 0:
        tilemap_update_requested.emit()
        print("[IcebergManager] Moved clusters:", number_of_clusters_moved)
    else:
        print("[IcebergManager] No clusters moved.")
    print("------------------------------------------\n")

func move_cluster_down_one_step(glacier_data: GlacierData, iceberg_cells_cluster: Array[Vector2i]) -> bool:
    var glacier_dimensions: Vector2i = GlacierUtil.get_dimensions(glacier_data)
    for cell in iceberg_cells_cluster:
        var below: Vector2i = cell + Vector2i(0, 1)
        if below.y >= glacier_dimensions.y:
            print("[IcebergManager] Cluster blocked by bottom edge at", cell)
            return false

        var state_below: int = glacier_data.get_state(below)
        if state_below == GlacierCellState.STATE.NONE:
            continue
        if state_below == GlacierCellState.STATE.INTACT:
            emit_signal("request_forced_fracture", below)
        elif state_below == GlacierCellState.STATE.FRACTURED or state_below == GlacierCellState.STATE.ICEBERG:
            if not iceberg_cells_cluster.has(below):
                unify_cluster_with_cell(glacier_data, iceberg_cells_cluster, below)
    for cell in iceberg_cells_cluster:
        var below_again: Vector2i = cell + Vector2i(0, 1)
        if below_again.y >= glacier_dimensions.y:
            print("[IcebergManager] Still blocked by edge after forced fracturing.")
            return false
        var state_below: int = glacier_data.get_state(below_again)
        if state_below != GlacierCellState.STATE.NONE and not iceberg_cells_cluster.has(below_again):
            print("[IcebergManager] Still blocked after merges/fractures:", below_again)
            return false
    for cell in iceberg_cells_cluster:
        glacier_data.set_state(cell, GlacierCellState.STATE.NONE)
    for i in range(iceberg_cells_cluster.size()):
        iceberg_cells_cluster[i] = iceberg_cells_cluster[i] + Vector2i(0, 1)
    for cell in iceberg_cells_cluster:
        glacier_data.set_state(cell, GlacierCellState.STATE.ICEBERG)
        glacier_data.set_time_in_state(cell, 0)  # NEW: reset timer after moving
    print("[IcebergManager] Moved cluster down one row:", iceberg_cells_cluster)
    return true


func is_anchored_from_below(glacier_data: GlacierData, cell_position: Vector2i) -> bool:
    var glacier_dimensions: Vector2i = GlacierUtil.get_dimensions(glacier_data)
    if cell_position.y == glacier_dimensions.y - 1:
        return true

    var below: Vector2i = cell_position + Vector2i(0, 1)
    if GlacierUtil.is_valid_cell(glacier_data, below):
        var state_below: int = glacier_data.get_state(below)
        if state_below == GlacierCellState.STATE.FRACTURED or state_below == GlacierCellState.STATE.ICEBERG:
            return true
    return false


func calculate_average_position_of_cluster(cluster_cells: Array[Vector2i]) -> Vector2i:
    var total_x: int = 0
    var total_y: int = 0
    for cell in cluster_cells:
        total_x += cell.x
        total_y += cell.y
    var average_x: int = total_x / cluster_cells.size()
    var average_y: int = total_y / cluster_cells.size()
    return Vector2i(average_x, average_y)

func unify_cluster_with_cell(
    glacier_data: GlacierData,
    iceberg_cells_cluster: Array[Vector2i],
    start_cell: Vector2i
) -> void:
    # TODO: Review and refine unification logic.
    # Currently, this function flood-fills from the starting cell and adds all forced cells (cells where is_forced() returns true)
    # into the moving cluster. This can sometimes cause a full merge cycle with another iceberg cluster,
    # resulting in a temporary pause in movement if forced cells (with timer == 0) are merged into an already moving cluster.
    # Consider adjusting the criteria so that only forced cells from the same forced event are merged,
    # or convert forced cells to ICEBERG earlier to avoid resetting the movement delay.
    var connected_cells: Array[Vector2i] = GlacierUtil.flood_fill(
        glacier_data,
        start_cell,
        func(pos: Vector2i) -> bool:
            # Only include cells that are marked as forced.
            return glacier_data.is_forced(pos)
    )

    for cell_position_in_cluster in connected_cells:
        if not iceberg_cells_cluster.has(cell_position_in_cluster):
            iceberg_cells_cluster.append(cell_position_in_cluster)
    print("[IcebergManager] unify_cluster_with_cell => cluster now size:", iceberg_cells_cluster.size())
