extends Node
class_name GlacierData

var glacier_cells_states: Array = []
var dirty_cells: Array = []
var glacier_cells_ages_in_lifecycle: Array = []  #TODO: really still???
var active_fractures: Array[Vector2i] = []  #TODO this is dangerous, figure out how to actually update and control this


func initialize_from_tilemap(tilemap: TileMapLayer) -> void:
    var used_tile_rect: Rect2i = tilemap.get_used_rect()
    var width: int = used_tile_rect.size.x
    var height: int = used_tile_rect.size.y

    glacier_cells_states.clear()
    glacier_cells_ages_in_lifecycle.clear()

    glacier_cells_states.resize(height)
    glacier_cells_ages_in_lifecycle.resize(height)

    for y: int in range(height):
        glacier_cells_states[y] = []
        glacier_cells_ages_in_lifecycle[y] = []

        for rel_x: int in range(width):
            var absolute_x: int = used_tile_rect.position.x + rel_x
            var absolute_y: int = used_tile_rect.position.y + y
            var cell_position: Vector2i = Vector2i(absolute_x, absolute_y)
            dirty_cells.append(cell_position)  #TODO: ugly but needed for first pass
            var tile_atlas_coords: Vector2i = tilemap.get_cell_atlas_coords(cell_position)

            if tile_atlas_coords == Vector2i(-1, -1):
                glacier_cells_states[y].append(GlacierCellState.STATE.NONE)
            else:
                glacier_cells_states[y].append(tile_atlas_coords.y)

            glacier_cells_ages_in_lifecycle[y].append(0)


func IS_INTACT(cell_position: Vector2i) -> bool:
    return get_glacier_cell_state(cell_position) == GlacierCellState.STATE.INTACT


func IS_FRACTURED(cell_position: Vector2i) -> bool:
    return get_glacier_cell_state(cell_position) == GlacierCellState.STATE.FRACTURED


func IS_ICEBERG(cell_position: Vector2i) -> bool:
    return get_glacier_cell_state(cell_position) == GlacierCellState.STATE.ICEBERG


func IS_NONE(cell_position: Vector2i) -> bool:
    return get_glacier_cell_state(cell_position) == GlacierCellState.STATE.NONE


func HAS_AGED_ONE_CYCLE(cell_position: Vector2i) -> bool:
    return get_glacier_cells_age_in_lifecycle(cell_position) >= 1


func IS_AGED_AND_INTACT(cell_position: Vector2i) -> bool:
    return HAS_AGED_ONE_CYCLE(cell_position) and IS_INTACT(cell_position)


func IS_AGED_AND_FRACTURED(cell_position: Vector2i) -> bool:
    return HAS_AGED_ONE_CYCLE(cell_position) and IS_FRACTURED(cell_position)


func IS_AGED_AND_ICEBERG(cell_position: Vector2i) -> bool:
    return HAS_AGED_ONE_CYCLE(cell_position) and IS_ICEBERG(cell_position)


func get_glacier_cell_state(cell_position: Vector2i) -> int:
    return glacier_cells_states[cell_position.y][cell_position.x]


func set_glacier_cell_state(cell_position: Vector2i, new_state: int) -> void:
    glacier_cells_states[cell_position.y][cell_position.x] = new_state
    dirty_cells.append(cell_position)


func get_glacier_cells_age_in_lifecycle(cell_position: Vector2i) -> int:
    return glacier_cells_ages_in_lifecycle[cell_position.y][cell_position.x]


func set_glacier_cells_age_in_lifecycle(cell_position: Vector2i, value: int) -> void:
    glacier_cells_ages_in_lifecycle[cell_position.y][cell_position.x] = value


func increase_glacier_cells_age_in_lifecycle() -> void:
    for y: int in range(glacier_cells_ages_in_lifecycle.size()):
        for x: int in range(glacier_cells_ages_in_lifecycle[y].size()):
            glacier_cells_ages_in_lifecycle[y][x] += 1
