extends Node2D
class_name GlacierSimulation

const GLACIER_MAP_SCENE: PackedScene = preload("res://Resources/TileMaps/GlacierMap.tscn")

@export var fracturing_cycle_interval: float = 0.5

var glacier_map: TileMapLayer
var glacier_data: GlacierData = GlacierData.new()
var hydrofracture_manager: HydrofractureManager = HydrofractureManager.new()
var iceberg_manager: IcebergManager = IcebergManager.new()
var fracturing_timer: Timer = Timer.new()

var water_caustics_shader: WaterCaustics = WaterCaustics.new()


func _ready() -> void:
    setup_glacier()
    fracturing_timer.wait_time = fracturing_cycle_interval
    fracturing_timer.timeout.connect(_on_simulation_tick)
    add_child(fracturing_timer)
    fracturing_timer.start()
    water_caustics_shader.z_index = -1
    add_child(water_caustics_shader)


func setup_glacier() -> void:
    glacier_map = GLACIER_MAP_SCENE.instantiate() as TileMapLayer
    add_child(glacier_map)
    glacier_data.initialize_from_tilemap(glacier_map)
    iceberg_manager.iceberg_cluster_formed.connect(_on_iceberg_cluster_formed)
    iceberg_manager.iceberg_cluster_moved.connect(_on_iceberg_cluster_moved)

    iceberg_manager.force_fracture_glacier_cell.connect(
        _on_iceberg_manager_force_fracture_glacier_cell
    )
    update_tilemap()


func _on_simulation_tick() -> void:
    glacier_data.increase_glacier_cells_age_in_lifecycle()
    hydrofracture_manager.run_hydrofracture_cycle(glacier_data)
    iceberg_manager.identify_and_form_iceberg_clusters(glacier_data)
    iceberg_manager.move_icebergs(glacier_data)
    update_tilemap()


func _on_iceberg_manager_force_fracture_glacier_cell(cell_position: Vector2i) -> void:
    hydrofracture_manager.fracture_glacier_cell(glacier_data, cell_position)
    update_tilemap()


func _on_iceberg_cluster_formed(cluster_id: int, iceberg_cluster: Array[Vector2i]) -> void:
    var tile_size: float = 16.0  # TODO: FIX THIS EVERYTHWERE IN THE CODE whole codebase!!!
    var iceberg_cluster_anchor_in_tile_coordinates: Vector2i = (
        iceberg_manager.calculate_iceberg_cluster_anchor_in_tile_coordinates(iceberg_cluster)
    )

    (
        water_caustics_shader
        . update_iceberg_clusters_anchor_position_from_discrete_tile_space_to_continious_interpolated_screen_space(
            cluster_id, iceberg_cluster_anchor_in_tile_coordinates
        )
    )
    var start_index: int = water_caustics_shader.iceberg_tile_positions.size()
    for iceberg_cell: Vector2i in iceberg_cluster:
        var local_position_in_iceberg_cluster_bounding_box: Vector2 = (
            (iceberg_cell - iceberg_cluster_anchor_in_tile_coordinates) * tile_size
        )
        water_caustics_shader.iceberg_tile_positions.append(
            local_position_in_iceberg_cluster_bounding_box
        )
    var end_index: int = water_caustics_shader.iceberg_tile_positions.size()
    water_caustics_shader.cluster_offsets.append(start_index)
    water_caustics_shader.cluster_offsets.append(end_index)


func _on_iceberg_cluster_moved(
    cluster_id: int, iceberg_cluster_anchor_tile_coordinates: Vector2i
) -> void:
    (
        water_caustics_shader
        . update_iceberg_clusters_anchor_position_from_discrete_tile_space_to_continious_interpolated_screen_space(
            cluster_id, iceberg_cluster_anchor_tile_coordinates
        )
    )
    update_tilemap()


func update_tilemap() -> void:
    GlacierUtil.for_each_cell(
        glacier_data,
        func(cell_position: Vector2i) -> void:
            var cell_state: GlacierCellState.STATE = (
                glacier_data.glacier_cells_states[cell_position.y][cell_position.x]
            )
            if cell_state == GlacierCellState.STATE.NONE:
                glacier_map.erase_cell(cell_position)
            else:
                glacier_map.set_cell(cell_position, GlacierGen.SOURCE_ID, Vector2i(0, cell_state))
    )
