extends Node2D
class_name GlacierSimulation

const GLACIER_MAP_SCENE: PackedScene = preload("res://Resources/TileMaps/GlacierMap.tscn")

@export var fracturing_cycle_interval: float = 1.0
@export var formation_delay_seconds: float = 1.0
@export var movement_delay_seconds: float = 1.0

var glacier_map: TileMapLayer
var glacier_data: GlacierData = GlacierData.new()
var hydrofracture_manager: HydrofractureManager = HydrofractureManager.new()
var iceberg_manager: IcebergManager = IcebergManager.new()

var fracturing_timer: Timer = Timer.new()

var queued_tasks: Array[Dictionary] = []

var simulation_time: float = 0.0

var formation_pending: bool = false
var movement_pending: bool = false


func _ready() -> void:
    setup_glacier()
    fracturing_timer.wait_time = fracturing_cycle_interval
    fracturing_timer.timeout.connect(_on_fracture_timer)
    add_child(fracturing_timer)
    fracturing_timer.start()


func _process(delta_time: float) -> void:
    simulation_time += delta_time
    while queued_tasks.size() > 0 and queued_tasks[0]["execute_at_time"] <= simulation_time:
        var current_task: Dictionary = queued_tasks.pop_front()
        var action: String = current_task["action"]
        match action:
            "formation":
                run_formation_phase()
                formation_pending = false
                if not movement_pending:
                    queue_task_in_seconds(movement_delay_seconds, "movement")
                    movement_pending = true
            "movement":
                run_movement_phase()
                movement_pending = false
            _:
                pass


func setup_glacier() -> void:
    glacier_map = GLACIER_MAP_SCENE.instantiate() as TileMapLayer
    add_child(glacier_map)

    glacier_data.initialize_from_tilemap(glacier_map)

    iceberg_manager.iceberg_cluster_formed.connect(_on_iceberg_formed)
    iceberg_manager.request_forced_fracture.connect(_on_iceberg_manager_request_forced_fracture)

    update_tilemap()


func _on_fracture_timer() -> void:
    hydrofracture_manager.run_fracture_phase(glacier_data)
    hydrofracture_manager.finalize_fracture_cycle()
    update_tilemap()

    if not formation_pending:
        queue_task_in_seconds(formation_delay_seconds, "formation")
        formation_pending = true


func run_formation_phase() -> void:
    var cells_fractured_last_cycle: Array[Vector2i] = hydrofracture_manager.get_cells_fractured_in_previous_cycle()
    iceberg_manager.iceberg_formation_phase(glacier_data, cells_fractured_last_cycle)
    update_tilemap()


func run_movement_phase() -> void:
    iceberg_manager.move_icebergs(glacier_data)
    update_tilemap()


func update_tilemap() -> void:
    for y: int in range(glacier_data.cell_state_grid.size()):
        for x: int in range(glacier_data.cell_state_grid[y].size()):
            var cell_state: int = glacier_data.cell_state_grid[y][x]
            glacier_map.set_cell(Vector2i(x, y), GlacierGen.SOURCE_ID, Vector2i(0, cell_state))


func queue_task_in_seconds(delay: float, action: String) -> void:
    var execute_time: float = simulation_time + delay
    var scheduled_task: Dictionary = {
        "execute_at_time": execute_time,
        "action": action
    }
    queued_tasks.push_back(scheduled_task)


func _on_iceberg_formed(average_position: Vector2i) -> void:
    # Process additional logic when an iceberg cluster is formed.
    pass


func _on_iceberg_manager_request_forced_fracture(blocker_cell_position: Vector2i) -> void:
    hydrofracture_manager.force_fracture_cell(glacier_data, blocker_cell_position)
    iceberg_manager.notify_cell_fractured(blocker_cell_position, glacier_data)
    update_tilemap()
