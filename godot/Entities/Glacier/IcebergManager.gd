extends Node2D
class_name IcebergManager

signal iceberg_created(position: Vector2i)
signal tilemap_updated()

@export var min_mass: int = 10
@export var max_icebergs: int = 20
@export var iceberg_flow_interval_seconds: float = 1.0

func identify_and_create_icebergs(glacier_data: GlacierData) -> int:
    var visited_cells: Dictionary = {}
    var new_icebergs_count: int = 0

    for y: int in range(glacier_data.mass_distribution.size()):
        for x: int in range(glacier_data.mass_distribution[y].size()):
            var pos: Vector2i = Vector2i(x, y)
            if visited_cells.has(pos):
                continue
            if glacier_data.get_state(pos) != GlacierCellState.STATE.FRACTURED:
                continue

            var cluster: Array[Vector2i] = []
            var mass: int = perform_mass_flood_fill(glacier_data, pos, visited_cells, cluster)
            if mass >= min_mass and new_icebergs_count < max_icebergs:
                convert_cluster_to_iceberg(glacier_data, cluster)
                new_icebergs_count += 1
    return new_icebergs_count

func perform_mass_flood_fill(
    glacier_data: GlacierData,
    start_pos: Vector2i,
    visited: Dictionary,
    cluster: Array[Vector2i]
) -> int:
    var stack: Array[Vector2i] = [start_pos]
    var mass: int = 0

    while stack.size() > 0:
        var current: Vector2i = stack.pop_back()
        if visited.has(current):
            continue
        if glacier_data.get_state(current) != GlacierCellState.STATE.FRACTURED:
            continue

        visited[current] = true
        mass += 1
        cluster.append(current)

        var neighbors = glacier_data.get_surrounding_cells(current)
        for neighbor: Vector2i in neighbors:
            if not visited.has(neighbor) and glacier_data.get_state(neighbor) == GlacierCellState.STATE.FRACTURED:
                stack.append(neighbor)
    return mass

func convert_cluster_to_iceberg(glacier_data: GlacierData, cluster: Array[Vector2i]) -> void:
    # bounding box
    var min_x: int = cluster[0].x
    var max_x: int = cluster[0].x
    var min_y: int = cluster[0].y
    var max_y: int = cluster[0].y
    for pos: Vector2i in cluster:
        if pos.x < min_x: min_x = pos.x
        if pos.x > max_x: max_x = pos.x
        if pos.y < min_y: min_y = pos.y
        if pos.y > max_y: max_y = pos.y

    var bbox_height: int = max_y - min_y + 1
    var bottom_weight: float = 0.5
    var threshold_y: int = max_y - int(bbox_height * bottom_weight)

    # convert only the lower portion to ICEBERG
    for pos: Vector2i in cluster:
        if pos.y >= threshold_y:
            glacier_data.set_state(pos, GlacierCellState.STATE.ICEBERG)

    # pick an average position to emit
    var sum_x: int = 0
    var sum_y: int = 0
    var count: int = 0
    for pos: Vector2i in cluster:
        if pos.y >= threshold_y:
            sum_x += pos.x
            sum_y += pos.y
            count += 1
    if count > 0:
        var avg_pos: Vector2i = Vector2i(sum_x / count, sum_y / count)
        iceberg_created.emit(avg_pos)
    print("[IcebergManager] Converted cluster => ICEBERG, threshold_y:", threshold_y, " cluster:", cluster)


##
## CLUSTER-BASED ICEBERG FLOW (ONE ROW PER CYCLE)
##
func update_iceberg_flow(glacier_data: GlacierData) -> void:
    var dims: Vector2i = glacier_data.get_dimensions()
    var visited: Dictionary = {}
    var clusters_moved: bool = false
    var cluster_count: int = 0

    print("\n[IcebergManager] ---- ICEBERG FLOW CYCLE START ----")

    # Process rows from bottom to top to avoid multiple moves in the same cycle
    for y: int in range(dims.y - 1, -1, -1):
        for x: int in range(dims.x):
            var pos: Vector2i = Vector2i(x, y)
            if visited.has(pos):
                continue
            if glacier_data.get_state(pos) != GlacierCellState.STATE.ICEBERG:
                continue

            # flood-fill to get this cluster
            var cluster: Array[Vector2i] = get_iceberg_cluster(glacier_data, pos, visited)
            cluster_count += 1
            print("[IcebergManager]  Cluster #", cluster_count, ": ", cluster)

            # check if cluster can move down
            var can_move: bool = true
            for cell: Vector2i in cluster:
                var below: Vector2i = cell + Vector2i(0, 1)
                # outside the grid => blocked
                if below.y >= dims.y:
                    can_move = false
                    print("[IcebergManager]   -> blocked by bottom edge at cell:", cell)
                    break
                # if below is NOT NONE and not in cluster => blocked
                var below_state: int = glacier_data.get_state(below)
                if below_state != GlacierCellState.STATE.NONE and cluster.find(below) == -1:
                    can_move = false
                    print("[IcebergManager]   -> blocked below:", below, " (state=", below_state, ")")
                    break

            if can_move:
                print("[IcebergManager]   -> Moving cluster DOWN:", cluster)
                # clear the old positions
                for cell: Vector2i in cluster:
                    glacier_data.set_state(cell, GlacierCellState.STATE.NONE)
                # set the new positions
                var new_positions: Array[Vector2i] = []
                for cell: Vector2i in cluster:
                    var new_cell: Vector2i = cell + Vector2i(0, 1)
                    new_positions.append(new_cell)
                    glacier_data.set_state(new_cell, GlacierCellState.STATE.ICEBERG)
                clusters_moved = true
                print("[IcebergManager]   -> New positions:", new_positions)
            else:
                print("[IcebergManager]   -> NOT moved:", cluster)

    if clusters_moved:
        emit_signal("tilemap_updated")
        print("[IcebergManager] Some clusters moved this cycle.")
    else:
        print("[IcebergManager] No clusters moved this cycle.")

    print("[IcebergManager] Flow cycle ended. Found", cluster_count, "clusters.")
    print("[IcebergManager] --------------------------------\n")


func get_iceberg_cluster(glacier_data: GlacierData, start: Vector2i, visited: Dictionary) -> Array[Vector2i]:
    var cluster: Array[Vector2i] = []
    var stack: Array[Vector2i] = [start]

    while stack.size() > 0:
        var cell: Vector2i = stack.pop_back()
        if visited.has(cell):
            continue
        visited[cell] = true

        if glacier_data.get_state(cell) == GlacierCellState.STATE.ICEBERG:
            cluster.append(cell)
            var offsets: Array[Vector2i] = [
                Vector2i(-1, 0),
                Vector2i(1, 0),
                Vector2i(0, -1),
                Vector2i(0, 1)
            ]
            for offset: Vector2i in offsets:
                var neighbor: Vector2i = cell + offset
                if glacier_data.is_valid_cell(neighbor) and not visited.has(neighbor):
                    if glacier_data.get_state(neighbor) == GlacierCellState.STATE.ICEBERG:
                        stack.append(neighbor)
    return cluster
