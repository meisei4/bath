extends Node
class_name GlacierData

var cell_state_grid: Array = []

func initialize_from_tilemap(tilemap: TileMapLayer) -> void:
    var used_rect = tilemap.get_used_rect()
    var width = used_rect.size.x
    var height = used_rect.size.y

    cell_state_grid.clear()
    cell_state_grid.resize(height)
    for y in range(height):
        cell_state_grid[y] = []

    for rel_y in range(height):
        for rel_x in range(width):
            var cell_x = used_rect.position.x + rel_x
            var cell_y = used_rect.position.y + rel_y
            var cell_pos = Vector2i(cell_x, cell_y)
            var atlas_coords = tilemap.get_cell_atlas_coords(cell_pos)

            if atlas_coords == Vector2i(-1, -1):
                # Mark as NONE
                cell_state_grid[rel_y].append(GlacierCellState.STATE.NONE)
            else:
                # atlas_coords.y = the appropriate enum state
                cell_state_grid[rel_y].append(atlas_coords.y)


func get_state(pos: Vector2i) -> int:
    return cell_state_grid[pos.y][pos.x]


func set_state(pos: Vector2i, new_state: int) -> void:
    cell_state_grid[pos.y][pos.x] = new_state
