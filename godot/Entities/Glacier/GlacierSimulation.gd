extends Node2D
class_name GlacierSimulation

const GLACIER_MAP_SCENE: PackedScene = preload("res://Resources/TileMaps/GlacierMap.tscn")
const GLACIAL_PARTICLES_SCENE: PackedScene = preload(
    "res://godot/Components/Particles/GlacialParticles.tscn"
)

#TODO: i already forgot why these should be @exports vs consts... something about being able to edit them in the GUI i guess?
@export var hydrofracture_interval_seconds: float = 1.0
@export var fractures_per_cycle: int = 1
@export var max_fracture_depth: int = 10
# fracture_coefficient is a kind of "weight" on the rate of propagation (bigger the number -> faster/more fractures propagate)
@export var fracturing_coefficient: float = 0.4

#TODO: havent looked at messing with "mass" yet, it's part of the IcebergManager algorithm stuff
@export var min_iceberg_mass: int = 10
@export var max_active_icebergs: int = 2
@export var iceberg_flow_interval_seconds: float = 1.0  # Adjust as needed

var glacier_map: TileMapLayer
var glacier_data: GlacierData =  GlacierData.new()
var hydrofracture_manager: HydrofractureManager = HydrofractureManager.new()
var iceberg_manager: IcebergManager = IcebergManager.new()

var hydrofracture_timer: Timer = Timer.new()
var iceberg_flow_timer: Timer = Timer.new()

const ICEBERG_GROUP: String = "Icebergs"
var active_icebergs_count: int = 0


func _ready() -> void:
    setup_glacier()
    setup_hydrofracture_timer()
    setup_iceberg_flow_timer()



func setup_glacier() -> void:
    glacier_map = GLACIER_MAP_SCENE.instantiate() as TileMapLayer
    glacier_data.initialize_from_tilemap(glacier_map)
    hydrofracture_manager.max_fracture_depth = self.max_fracture_depth
    hydrofracture_manager.fracturing_coefficient = self.fracturing_coefficient
    iceberg_manager.min_mass = self.min_iceberg_mass
    iceberg_manager.max_icebergs = self.max_active_icebergs
    iceberg_manager.iceberg_created.connect(_on_iceberg_created)
    add_child(glacier_map)
    update_tilemap()

func update_tilemap() -> void:
    for y: int in range(glacier_data.mass_distribution.size()):
        for x: int in range(glacier_data.mass_distribution[y].size()):
            var state: GlacierCellState.STATE = glacier_data.mass_distribution[y][x]
            var cell_position: Vector2i = Vector2i(x, y)
            glacier_map.set_cell(cell_position, GlacierGen.SOURCE_ID, Vector2i(0, state))


func setup_hydrofracture_timer() -> void:
    hydrofracture_timer.wait_time = hydrofracture_interval_seconds
    hydrofracture_timer.timeout.connect(_on_hydrofracture_cycle)
    add_child(hydrofracture_timer)
    hydrofracture_timer.start()


func setup_iceberg_flow_timer() -> void:
    iceberg_flow_timer.wait_time = iceberg_flow_interval_seconds
    iceberg_flow_timer.timeout.connect(_on_iceberg_flow_cycle)
    add_child(iceberg_flow_timer)
    iceberg_flow_timer.start()


func _on_hydrofracture_cycle() -> void:
    var fractures_started: int = hydrofracture_manager.run_cycle(glacier_data,fractures_per_cycle)
    if fractures_started == 0:
        return

    var new_icebergs: int  = iceberg_manager.identify_and_create_icebergs(glacier_data)
    active_icebergs_count += new_icebergs

    if active_icebergs_count >= max_active_icebergs:
        hydrofracture_timer.stop()
    update_tilemap()


func _on_iceberg_flow_cycle() -> void:
    iceberg_manager.update_iceberg_flow(glacier_data)
    update_tilemap()


func _on_iceberg_created(_position: Vector2i) -> void:
    var particles_instance: CPUParticles2D = GLACIAL_PARTICLES_SCENE.instantiate() as CPUParticles2D
    #TODO: this next line is a very ugly way to get Tile coordinates -> global coordinates (16x16 tiles)
    particles_instance.position = _position * 16
    #add_child(particles_instance)
