extends Node
class_name GlacierConstants

const TILE_SIZE_1D: int = 8
const SUBDIVISION_FACTOR: int = 2
const DS_RESOLUTION: Vector2i = Vector2i(256, 384)
const IMAGE_TEXTURE_SIZE: Vector2i = Vector2i(TILE_SIZE_1D, TILE_SIZE_1D)
const TEXTURE_REGION_SIZE: Vector2i = Vector2i(TILE_SIZE_1D, TILE_SIZE_1D)
const MARGIN: Vector2i = (TEXTURE_REGION_SIZE - IMAGE_TEXTURE_SIZE) / 2
const ATLAS_MARGINS: Vector2i = Vector2i(0, 0)
const TILE_SIZE: Vector2i = IMAGE_TEXTURE_SIZE
const GRID_TILE_SIZE: Vector2i = Vector2i(1, 1)
const ATLAS_SEPARATION: Vector2i = Vector2i(0, 0)

const TOTAL_GRID_WIDTH_IN_TILES: int = 32 * SUBDIVISION_FACTOR
const TOTAL_GRID_HEIGHT_IN_TILES: int = 48 * SUBDIVISION_FACTOR
const GLACIER_HEIGHT_IN_TILES: int = 16 * SUBDIVISION_FACTOR

const SOURCE_ID: int = 234

const UP: Vector2i = Vector2i(0, -1)
const DOWN: Vector2i = Vector2i(0, 1)
const LEFT: Vector2i = Vector2i(-1, 0)
const RIGHT: Vector2i = Vector2i(1, 0)

const CARDINAL_DIRECTIONS: Array[Vector2i] = [LEFT, RIGHT, UP, DOWN]

const MAXIMUM_FRACTURE_DEPTH: int = 6
const FRACTURE_PROPAGATION_PROBABILITY: float = 0.40
const MAXIMUM_NEW_FRACTURES_PER_CYCLE: int = 1

const MINIMUM_ICEBERG_CLUSTER_SIZE: int = 30

const FRACTURING_CYCLE_INTERVAL: float = 0.1
const SIMULATION_TICK_INTERVAL: float = 0.1  #TODO: match this with the actual glacier sim somewhere
const MAX_TOTAL_ICEBERG_TILES: int = 1024  #TODO: derive this from the amount of INTACT state cells at the beginning of the simulation
