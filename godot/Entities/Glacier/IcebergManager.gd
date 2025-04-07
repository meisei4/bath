extends Node2D
class_name IcebergManager

signal iceberg_cluster_formed(cluster_id: int, iceberg_cluster: Array[Vector2i])
 #TODO account for the iceberg cluster merging occuranced for the ripple redefinition
signal iceberg_cluster_merged(cluster_id_a: int, cluster_id_b: int)
signal iceberg_cluster_moved(cluster_id: int, iceberg_cluster: Array[Vector2i])
signal force_fracture_glacier_cell(cell: Vector2i)

var clusters: Dictionary[int, Array] = {}  # cluster_id -> list of cells

var iceberg_cell_to_cluster_id: Dictionary[Vector2i, int] = {} # Key: iceberg cell_coord, value: cluster id,
var current_iceberg_cluster_id: int = 0

#TODO: introduce this to make everything actually easy to understand instead of just low level crazy coordinate clusters in an array
class IcebergCluster:
    var id: int
    var cells: Array[Vector2i] = []
    var position: Vector2 = Vector2.ZERO
    var velocity: Vector2 = Vector2.ZERO
    var tile_positions: PackedVector2Array = PackedVector2Array()
    var active: bool = true

    func update(delta: float) -> void:
        if active:
            position += velocity * delta


func identify_and_form_iceberg_clusters(glacier_data: GlacierData) -> void:
    var visited: Dictionary[Vector2i, bool] = {}
    GlacierUtil.for_each_cell(glacier_data, func(cell_position: Vector2i) -> void:
        if not visited.has(cell_position) and glacier_data.IS_AGED_AND_FRACTURED(cell_position):
            var cluster: Array[Vector2i] = GlacierUtil.collect_connected_glacier_cells(
                glacier_data,
                cell_position,
                glacier_data.IS_AGED_AND_FRACTURED
            )
            for c: Vector2i in cluster:
                visited[c] = true
            form_iceberg_cluster(glacier_data, cluster)
    )

func form_iceberg_cluster(glacier_data: GlacierData, iceberg_cluster: Array[Vector2i]) -> void:
    if iceberg_cluster.size() >= GlacierConstants.MINIMUM_ICEBERG_CLUSTER_SIZE:
        clusters[current_iceberg_cluster_id] = iceberg_cluster.duplicate()
        for iceberg_cell: Vector2i in iceberg_cluster:
            form_iceberg(glacier_data, iceberg_cell, current_iceberg_cluster_id)
            iceberg_cell_to_cluster_id[iceberg_cell] = current_iceberg_cluster_id
        iceberg_cluster_formed.emit(current_iceberg_cluster_id, iceberg_cluster)
        current_iceberg_cluster_id += 1

func form_iceberg(glacier_data: GlacierData, iceberg_cell: Vector2i, iceberg_cluster_id: int) -> void:
    iceberg_cell_to_cluster_id[iceberg_cell] = iceberg_cluster_id
    glacier_data.set_glacier_cell_state(iceberg_cell, GlacierCellState.STATE.ICEBERG)
    glacier_data.set_glacier_cells_age_in_lifecycle(iceberg_cell, 0)
    #TODO: i dont know how to remove active fractures when they turn into icebergs, so i really dont like this


func move_icebergs(glacier_data: GlacierData) -> void:
    for cluster_id: int in clusters.keys():
        var iceberg_cluster: Array[Vector2i] = clusters[cluster_id]
        if can_iceberg_cluster_move_down(glacier_data, iceberg_cluster):
            update_cluster_position(cluster_id, glacier_data, iceberg_cluster, iceberg_cluster.duplicate())
        else:
            for iceberg_cell: Vector2i in iceberg_cluster:
                var cell_below: Vector2i = GlacierUtil.CELL_BELOW(iceberg_cell)
                handle_blocking_cell_below(glacier_data, cell_below, cluster_id)


func can_iceberg_cluster_move_down(
    glacier_data: GlacierData, iceberg_cluster: Array[Vector2i]
) -> bool:
    var glacier_dimensions: Vector2i = GlacierUtil.get_glacier_grid_dimensions(glacier_data)
    for cell_position: Vector2i in iceberg_cluster:
        var has_reached_bottom_of_screen: bool = cell_position.y >= glacier_dimensions.y
        if has_reached_bottom_of_screen:
            #TODO: some how remove the ripples from the icebergs that reach the bottom of the screen????
            #clusters.erase(iceberg_cell_to_cluster_id.get(cell_position))
            return false
        var cell_below: Vector2i = GlacierUtil.CELL_BELOW(cell_position)
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


func handle_blocking_cell_below(glacier_data: GlacierData, cell_below: Vector2i, iceberg_cluster_id: int) -> void:
    if not GlacierUtil.is_valid_glacier_cell(glacier_data, cell_below):
        return
    if glacier_data.IS_AGED_AND_INTACT(cell_below):
        force_fracture_glacier_cell.emit(cell_below)
    elif glacier_data.IS_AGED_AND_FRACTURED(cell_below):
        if iceberg_cell_to_cluster_id.has(cell_below):
            var existing_cluster_id: int = iceberg_cell_to_cluster_id[cell_below]
            if existing_cluster_id != iceberg_cluster_id:
                merge_iceberg_clusters(iceberg_cluster_id, existing_cluster_id)
        else:
            add_cell_to_cluster(cell_below, iceberg_cluster_id, glacier_data)


func update_cluster_position(
    iceberg_cluster_id: int, glacier_data: GlacierData, iceberg_cluster: Array[Vector2i], initial_cluster: Array[Vector2i]
) -> void:
    for cell_position: Vector2i in initial_cluster:
        glacier_data.set_glacier_cell_state(cell_position, GlacierCellState.STATE.NONE)
        #TODO: dear lord still gross
        iceberg_cell_to_cluster_id.erase(cell_position)
    for i: int in range(iceberg_cluster.size()):
        iceberg_cluster[i] = GlacierUtil.CELL_BELOW(iceberg_cluster[i])
    for cell_position: Vector2i in iceberg_cluster:
        form_iceberg(glacier_data, cell_position, iceberg_cluster_id)
        #TODO: dear lord double gross
        iceberg_cell_to_cluster_id.set(cell_position, iceberg_cluster_id)
    iceberg_cluster_moved.emit(iceberg_cluster_id, calculate_iceberg_cluster_anchor_in_tile_coordinates(iceberg_cluster))

#TODO: this is the x,y origin of the bounding box around the provided iceberg cluster (to create a local-space per cluster)
func calculate_iceberg_cluster_anchor_in_tile_coordinates(iceberg_cluster: Array[Vector2i]) -> Vector2i:
    var min_x: float = INF # TODO: not sure what to do about float INF, idk
    var min_y: float = INF
    for iceberg_cell: Vector2i in iceberg_cluster:
        if iceberg_cell.x < min_x:
            min_x = iceberg_cell.x
        if iceberg_cell.y < min_y:
            min_y = iceberg_cell.y
    var iceberg_cluster_anchor_coordinates: Vector2i = Vector2i(min_x, min_y)
    return iceberg_cluster_anchor_coordinates


func merge_iceberg_clusters(cluster_a_id: int, cluster_b_id: int) -> void:
    if not clusters.has(cluster_a_id) or not clusters.has(cluster_b_id):
        return
    var cluster_a: Array = clusters[cluster_a_id]
    var cluster_b: Array = clusters[cluster_b_id]
    for cell: Vector2i in cluster_b:
        if cell not in cluster_a:
            cluster_a.append(cell)
            iceberg_cell_to_cluster_id[cell] = cluster_a_id

    clusters[cluster_a_id] = cluster_a
    clusters.erase(cluster_b_id)
    iceberg_cluster_merged.emit(cluster_a_id, cluster_b_id)

func add_cell_to_cluster(cell: Vector2i, cluster_id: int, glacier_data: GlacierData) -> void:
    if clusters.has(cluster_id):
        clusters[cluster_id].append(cell)
        iceberg_cell_to_cluster_id[cell] = cluster_id
        glacier_data.set_glacier_cell_state(cell, GlacierCellState.STATE.ICEBERG)
        glacier_data.set_glacier_cells_age_in_lifecycle(cell, 0)
