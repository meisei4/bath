extends Node
class_name GlacierData

var cell_state_grid: Array = []
var cell_time_grid: Array = []  # stores each cell’s time in its current state
var cell_forced_grid: Array = []  # grid of booleans indicating if a cell was forced fractured

func initialize_from_tilemap(tilemap: TileMapLayer) -> void:
    var used_rect = tilemap.get_used_rect()
    var width = used_rect.size.x
    var height = used_rect.size.y

    cell_state_grid.clear()
    cell_state_grid.resize(height)
    cell_time_grid.clear()
    cell_time_grid.resize(height)
    cell_forced_grid.clear()
    cell_forced_grid.resize(height)
    for y in range(height):
        cell_state_grid[y] = []
        cell_time_grid[y] = []
        cell_forced_grid[y] = []
        for rel_x in range(width):
            var cell_x = used_rect.position.x + rel_x
            var cell_y = used_rect.position.y + y
            var cell_pos = Vector2i(cell_x, cell_y)
            var atlas_coords = tilemap.get_cell_atlas_coords(cell_pos)
            if atlas_coords == Vector2i(-1, -1):
                cell_state_grid[y].append(GlacierCellState.STATE.NONE)
            else:
                cell_state_grid[y].append(atlas_coords.y)
            cell_time_grid[y].append(0)
            cell_forced_grid[y].append(false)

func get_state(pos: Vector2i) -> int:
    return cell_state_grid[pos.y][pos.x]

func set_state(pos: Vector2i, new_state: int) -> void:
    cell_state_grid[pos.y][pos.x] = new_state

func get_time_in_state(pos: Vector2i) -> int:
    return cell_time_grid[pos.y][pos.x]

func set_time_in_state(pos: Vector2i, value: int) -> void:
    cell_time_grid[pos.y][pos.x] = value

func increment_time_in_state() -> void:
    for y in range(cell_time_grid.size()):
        for x in range(cell_time_grid[y].size()):
            cell_time_grid[y][x] += 1

func is_forced(pos: Vector2i) -> bool:
    return cell_forced_grid[pos.y][pos.x]

func set_forced(pos: Vector2i, value: bool) -> void:
    cell_forced_grid[pos.y][pos.x] = value
