extends Node2D
class_name GlacierGen

const GLACIER_WIDTH: int = 16
const TOTAL_GRID_HEIGHT: int = 24

const SOURCE_ID: int = 234

var glacier_states_instance: GlacierCellState = GlacierCellState.new()

var glacier_surface: TileMapLayer

const IMAGE_TEXTURE_SIZE: Vector2i = Vector2i(16, 16)
const TEXTURE_REGION_SIZE: Vector2i = Vector2i(16, 16)
const MARGIN: Vector2i = (TEXTURE_REGION_SIZE - IMAGE_TEXTURE_SIZE) / 2
const ATLAS_MARGINS: Vector2i = Vector2i(0, 0)
const TILE_SIZE: Vector2i = IMAGE_TEXTURE_SIZE
const GRID_TILE_SIZE: Vector2i = Vector2i(1, 1)
const ATLAS_SEPARATION: Vector2i = Vector2i(0, 0)

func _ready() -> void:
    initialize_glacier_surface()

func initialize_glacier_surface() -> void:
    glacier_surface = TileMapLayer.new()
    var glacier_tileset: TileSet = create_and_save_glacier_tile_set()
    glacier_surface.set_tile_set(glacier_tileset)

    fill_initial_states()

    var glacier_scene: PackedScene = PackedScene.new()
    glacier_scene.pack(glacier_surface)
    ResourceSaver.save(glacier_scene, "res://Resources/TileMaps/GlacierMap.tscn")

func fill_initial_states() -> void:
    var glacier_height: int = 8

    for y: int in range(TOTAL_GRID_HEIGHT):
        for x: int in range(GLACIER_WIDTH):
            if y < glacier_height:
                glacier_surface.set_cell(
                    Vector2i(x, y), SOURCE_ID, Vector2i(0, GlacierCellState.STATE.INTACT)
                )
            else:
                glacier_surface.set_cell(
                    Vector2i(x, y), SOURCE_ID, Vector2i(0, GlacierCellState.STATE.NONE)
                )


func create_and_save_glacier_tile_set() -> TileSet:
    var glacier_states: Array = GlacierCellState.STATE.values()
    var glacier_tileset: TileSet = TileSet.new()
    glacier_tileset.set_tile_size(TILE_SIZE)

    var atlas_source: TileSetAtlasSource = TileSetAtlasSource.new()
    atlas_source.set_margins(ATLAS_MARGINS)
    atlas_source.set_separation(ATLAS_SEPARATION)
    atlas_source.set_use_texture_padding(true)

    var atlas_texture_width: int = TEXTURE_REGION_SIZE.x
    var atlas_texture_height: int = TEXTURE_REGION_SIZE.y * glacier_states.size()
    var atlas_texture: Image = Image.create_empty(
        atlas_texture_width, atlas_texture_height, false, Image.FORMAT_RGBA8
    )
    atlas_source.set_texture_region_size(TEXTURE_REGION_SIZE)
    for tile_index: int in range(glacier_states.size()):
        var state: GlacierCellState.STATE = glacier_states[tile_index]
        var color: Color = glacier_states_instance.get_color(state)
        var x_offset: int = MARGIN.x + ATLAS_MARGINS.x
        var y_offset: int = TEXTURE_REGION_SIZE.y * tile_index + MARGIN.y + ATLAS_MARGINS.y
        for i: int in range(IMAGE_TEXTURE_SIZE.x):
            for j: int in range(IMAGE_TEXTURE_SIZE.y):
                atlas_texture.set_pixel(x_offset + i, y_offset + j, color)

    var final_texture: ImageTexture = ImageTexture.create_from_image(atlas_texture)
    atlas_source.set_texture(final_texture)
    for tile_index: int in range(glacier_states.size()):
        var atlas_coords: Vector2i = Vector2i(0, tile_index)
        atlas_source.create_tile(atlas_coords, GRID_TILE_SIZE)

    glacier_tileset.add_source(atlas_source)
    glacier_tileset.set_source_id(0, SOURCE_ID)

    ResourceSaver.save(glacier_tileset, "res://Resources/TileSets/glacier_tileset.tres")
    return glacier_tileset
