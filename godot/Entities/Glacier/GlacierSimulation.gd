extends Node2D
class_name GlacierSimulation

const GLACIER_MAP_SCENE: PackedScene = preload("res://Resources/TileMaps/GlacierMap.tscn")

@export var fracturing_cycle_interval: float = 1.0

var glacier_map: TileMapLayer
var glacier_data: GlacierData = GlacierData.new()
var hydrofracture_manager: HydrofractureManager = HydrofractureManager.new()
var iceberg_manager: IcebergManager = IcebergManager.new()
var fracturing_timer: Timer = Timer.new()


func _ready() -> void:
    setup_glacier()
    fracturing_timer.wait_time = fracturing_cycle_interval
    fracturing_timer.timeout.connect(_on_simulation_tick)
    add_child(fracturing_timer)
    fracturing_timer.start()


func setup_glacier() -> void:
    glacier_map = GLACIER_MAP_SCENE.instantiate() as TileMapLayer
    add_child(glacier_map)
    glacier_data.initialize_from_tilemap(glacier_map)
    iceberg_manager.iceberg_cluster_formed.connect(_on_iceberg_formed)
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


func _on_iceberg_formed(average_position: Vector2i) -> void:
    #TODO: wait for the water shader i think? to allow for setting the avg position of the clusters to the ripple source or something
    pass


func update_tilemap() -> void:
    GlacierUtil.for_each_cell(
        glacier_data,
        func(cell_position) -> void:
            var cell_state = glacier_data.glacier_cells_states[cell_position.y][cell_position.x]
            glacier_map.set_cell(cell_position, GlacierGen.SOURCE_ID, Vector2i(0, cell_state))
    )
