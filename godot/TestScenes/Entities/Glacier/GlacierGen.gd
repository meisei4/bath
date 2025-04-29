extends Node2D
class_name GlacierGen

var glacier_states_instance: GlacierCellState = GlacierCellState.new()

var glacier_surface: TileMapLayer


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
    for y: int in range(GlacierConstants.TOTAL_GRID_HEIGHT_IN_TILES):
        for x: int in range(GlacierConstants.TOTAL_GRID_WIDTH_IN_TILES):
            if y < GlacierConstants.GLACIER_HEIGHT_IN_TILES:
                glacier_surface.set_cell(
                    Vector2i(x, y),
                    GlacierConstants.SOURCE_ID,
                    Vector2i(0, GlacierCellState.STATE.INTACT)
                )
            else:
                glacier_surface.set_cell(
                    Vector2i(x, y),
                    GlacierConstants.SOURCE_ID,
                    Vector2i(0, GlacierCellState.STATE.NONE)
                )


func create_and_save_glacier_tile_set() -> TileSet:
    var glacier_states: Array = GlacierCellState.STATE.values()
    var glacier_tileset: TileSet = TileSet.new()
    glacier_tileset.set_tile_size(GlacierConstants.TILE_SIZE)

    var atlas_source: TileSetAtlasSource = TileSetAtlasSource.new()
    atlas_source.set_margins(GlacierConstants.ATLAS_MARGINS)
    atlas_source.set_separation(GlacierConstants.ATLAS_SEPARATION)
    atlas_source.set_use_texture_padding(true)

    var atlas_texture_width: int = GlacierConstants.TEXTURE_REGION_SIZE.x
    var atlas_texture_height: int = GlacierConstants.TEXTURE_REGION_SIZE.y * glacier_states.size()
    var atlas_texture: Image = Image.create_empty(
        atlas_texture_width, atlas_texture_height, false, Image.FORMAT_RGBA8
    )
    atlas_source.set_texture_region_size(GlacierConstants.TEXTURE_REGION_SIZE)
    for tile_index: int in range(glacier_states.size()):
        var state: GlacierCellState.STATE = glacier_states[tile_index]
        var color: Color = glacier_states_instance.get_color(state)
        var x_offset: int = GlacierConstants.MARGIN.x + GlacierConstants.ATLAS_MARGINS.x
        var y_offset: int = (
            GlacierConstants.TEXTURE_REGION_SIZE.y * tile_index
            + GlacierConstants.MARGIN.y
            + GlacierConstants.ATLAS_MARGINS.y
        )
        for i: int in range(GlacierConstants.IMAGE_TEXTURE_SIZE.x):
            for j: int in range(GlacierConstants.IMAGE_TEXTURE_SIZE.y):
                atlas_texture.set_pixel(x_offset + i, y_offset + j, color)

    var final_texture: ImageTexture = ImageTexture.create_from_image(atlas_texture)
    atlas_source.set_texture(final_texture)
    for tile_index: int in range(glacier_states.size()):
        var atlas_coords: Vector2i = Vector2i(0, tile_index)
        atlas_source.create_tile(atlas_coords, GlacierConstants.GRID_TILE_SIZE)

    glacier_tileset.add_source(atlas_source)
    glacier_tileset.set_source_id(0, GlacierConstants.SOURCE_ID)

    ResourceSaver.save(glacier_tileset, "res://Resources/TileSets/glacier_tileset.tres")
    return glacier_tileset
