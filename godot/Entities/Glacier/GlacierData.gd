extends Node
class_name GlacierData

var cell_state_grid: Array = []
var cell_time_grid: Array = []
var cell_forced_grid: Array = []

func initialize_from_tilemap(tilemap: TileMapLayer) -> void:
    var used_tile_rect: Rect2i = tilemap.get_used_rect()
    var width: int = used_tile_rect.size.x
    var height: int = used_tile_rect.size.y

    cell_state_grid.clear()
    cell_time_grid.clear()
    cell_forced_grid.clear()

    cell_state_grid.resize(height)
    cell_time_grid.resize(height)
    cell_forced_grid.resize(height)

    for y: int in range(height):
        cell_state_grid[y] = []
        cell_time_grid[y] = []
        cell_forced_grid[y] = []

        for rel_x: int in range(width):
            var absolute_x: int = used_tile_rect.position.x + rel_x
            var absolute_y: int = used_tile_rect.position.y + y
            var cell_position: Vector2i = Vector2i(absolute_x, absolute_y)
            var tile_atlas_coords: Vector2i = tilemap.get_cell_atlas_coords(cell_position)

            if tile_atlas_coords == Vector2i(-1, -1):
                cell_state_grid[y].append(GlacierCellState.STATE.NONE)
            else:
                cell_state_grid[y].append(tile_atlas_coords.y)

            cell_time_grid[y].append(0)
            cell_forced_grid[y].append(false)

func get_state(cell_position: Vector2i) -> int:
    return cell_state_grid[cell_position.y][cell_position.x]

func set_state(cell_position: Vector2i, new_state: int) -> void:
    cell_state_grid[cell_position.y][cell_position.x] = new_state

func get_time_in_state(cell_position: Vector2i) -> int:
    return cell_time_grid[cell_position.y][cell_position.x]

func set_time_in_state(cell_position: Vector2i, value: int) -> void:
    cell_time_grid[cell_position.y][cell_position.x] = value

func increment_time_in_state() -> void:
    for y: int in range(cell_time_grid.size()):
        for x: int in range(cell_time_grid[y].size()):
            cell_time_grid[y][x] += 1

func is_forced(cell_position: Vector2i) -> bool:
    return cell_forced_grid[cell_position.y][cell_position.x]

func set_forced(cell_position: Vector2i, value: bool) -> void:
    cell_forced_grid[cell_position.y][cell_position.x] = value
