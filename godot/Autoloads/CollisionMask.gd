extends StaticBody2D

var iResolution: Vector2
var rendering_device: RenderingDevice
var compute_shader_file: RDShaderFile = load(
    "res://Resources/Shaders/Compute/glacier_collision_mask.glsl"
)
var compute_shader_spirv: RDShaderSPIRV
var compute_shader_rid: RID
var compute_pipeline_rid: RID

const COLLISION_MASK_SSBO_UNIFORM_BINDING: int = 0
var collision_mask_uniform: RDUniform
var collision_mask_texture_format: RDTextureFormat
var collision_mask_view: RDTextureView
var collision_mask_texture_view_rid: RID
var collision_mask_texture: Texture2DRD

var collision_mask_polygons: Array[CollisionPolygon2D]

var push_constants: PackedByteArray
const PUSH_CONSTANTS_BYTE_BLOCK_SIZE: int = 16
const PUSH_CONSTANTS_BYTE_ALIGNMENT_0: int = 0
const PUSH_CONSTANTS_BYTE_ALIGNMENT_4: int = 4
const PUSH_CONSTANTS_BYTE_ALIGNMENT_8: int = 8
const PUSH_CONSTANTS_BYTE_ALIGNMENT_12: int = 12

const WORKGROUP_TILE_PIXELS_X: int = 2
const WORKGROUP_TILE_PIXELS_Y: int = 2

var iTime: float

var debug_spr: Sprite2D


func _ready() -> void:
    _init_rendering_device()
    _init_compute_shader_pipeline()
    _init_collision_mask_texture()
    RenderingServer.frame_pre_draw.connect(_dispatch_compute)
    RenderingServer.frame_post_draw.connect(_on_first_frame)


var _polygons_built := false


func _on_first_frame() -> void:
    #TODO this draws some collision shapes!! for the first frame only
    if _polygons_built:
        return
    #_polygons_built = true
    _debug_print_ascii()
    _generate_collision_polygons()


func _init_rendering_device() -> void:
    iResolution = Resolution.resolution
    rendering_device = RenderingServer.get_rendering_device()


func _init_compute_shader_pipeline() -> void:
    compute_shader_spirv = compute_shader_file.get_spirv()
    compute_shader_rid = rendering_device.shader_create_from_spirv(compute_shader_spirv)
    compute_pipeline_rid = rendering_device.compute_pipeline_create(compute_shader_rid)


func _init_collision_mask_texture() -> void:
    collision_mask_texture_format = RDTextureFormat.new()
    collision_mask_texture_format.texture_type = RenderingDevice.TEXTURE_TYPE_2D
    collision_mask_texture_format.format = RenderingDevice.DATA_FORMAT_R8G8B8A8_UNORM
    collision_mask_texture_format.width = iResolution.x as int
    collision_mask_texture_format.height = iResolution.y as int
    collision_mask_texture_format.depth = 1
    collision_mask_texture_format.array_layers = 1
    collision_mask_texture_format.mipmaps = 1
    collision_mask_texture_format.usage_bits = (
        RenderingDevice.TEXTURE_USAGE_STORAGE_BIT
        | RenderingDevice.TEXTURE_USAGE_CAN_UPDATE_BIT
        | RenderingDevice.TEXTURE_USAGE_SAMPLING_BIT
        | RenderingDevice.TEXTURE_USAGE_CAN_COPY_FROM_BIT
    )
    collision_mask_view = RDTextureView.new()
    collision_mask_texture_view_rid = rendering_device.texture_create(
        collision_mask_texture_format, collision_mask_view
    )
    collision_mask_texture = Texture2DRD.new()
    collision_mask_texture.set_texture_rd_rid(collision_mask_texture_view_rid)

    collision_mask_uniform = RDUniform.new()
    collision_mask_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_IMAGE
    collision_mask_uniform.binding = COLLISION_MASK_SSBO_UNIFORM_BINDING
    collision_mask_uniform.add_id(collision_mask_texture_view_rid)


func _dispatch_compute() -> void:
    var cl = rendering_device.compute_list_begin()
    rendering_device.compute_list_bind_compute_pipeline(cl, compute_pipeline_rid)
    var uset = rendering_device.uniform_set_create([collision_mask_uniform], compute_shader_rid, 0)
    rendering_device.compute_list_bind_uniform_set(cl, uset, 0)
    push_constants = PackedByteArray()
    push_constants.resize(PUSH_CONSTANTS_BYTE_BLOCK_SIZE)
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_0, iResolution.x)
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_4, iResolution.y)
    #TODO: this iTime global singleton value gets hack updated in the GlacierFlow test scene for now
    push_constants.encode_float(PUSH_CONSTANTS_BYTE_ALIGNMENT_8, iTime)
    push_constants.encode_u32(PUSH_CONSTANTS_BYTE_ALIGNMENT_12, 0)
    rendering_device.compute_list_set_push_constant(cl, push_constants, push_constants.size())
    var groups_x = int(ceil(iResolution.x / float(WORKGROUP_TILE_PIXELS_X)))
    var groups_y = int(ceil(iResolution.y / float(WORKGROUP_TILE_PIXELS_Y)))
    rendering_device.compute_list_dispatch(cl, groups_x, groups_y, 1)
    rendering_device.compute_list_end()


func _debug_print_ascii(tile_width: int = 4, tile_height: int = 8) -> void:
    var width: int = int(iResolution.x)
    var height: int = int(iResolution.y)
    var raw_data: PackedByteArray = rendering_device.texture_get_data(
        collision_mask_texture_view_rid, 0
    )
    var count_nonzero := 0
    for i in range(0, raw_data.size(), 4):
        if raw_data.decode_u8(i) != 0:
            count_nonzero += 1
    print(" non-zero mask pixels:", count_nonzero)
    var compact_mask: PackedByteArray = PackedByteArray()
    compact_mask.resize(width * height)
    for y in range(height):
        for x in range(width):
            var byte_index = ((y * width) + x) * 4
            compact_mask[y * width + x] = 1 if raw_data.decode_u8(byte_index) != 0 else 0

    var cols: int = int(ceil(width / float(tile_width)))
    var rows: int = int(ceil(height / float(tile_height)))
    print("ASCII square:", cols, "×", rows, "(tile_w=", tile_width, ", tile_h=", tile_height, ")")
    for row in range(rows):
        var sample_y_pos = clamp(row * tile_height + tile_height / 2, 0, height - 1)
        var source_y = height - 1 - sample_y_pos  # vertical flip TODO: GLSL and godot differ hahahahah???
        var line_text = ""
        for col in range(cols):
            var sample_x_pos = clamp(col * tile_width + tile_width / 2, 0, width - 1)
            line_text += "#" if compact_mask[source_y * width + sample_x_pos] == 1 else "."
        print(" ", line_text)


const TILE_SIZE_PIXELS: int = 2


func _generate_collision_polygons() -> void:
    for child in get_children():
        if child is StaticBody2D:
            remove_child(child)
            child.queue_free()
    collision_mask_polygons = []  # assign to the class variable, not `var`

    var width: int = int(iResolution.x)
    var height: int = int(iResolution.y)
    var raw_data: PackedByteArray = rendering_device.texture_get_data(
        collision_mask_texture_view_rid, 0
    )
    var pixel_mask_array: PackedByteArray = PackedByteArray()
    pixel_mask_array.resize(width * height)
    var solid_pixel_count: int = 0
    for y in range(height):
        for x in range(width):
            var byte_index = ((y * width) + x) * 4
            var value = raw_data.decode_u8(byte_index)
            if value != 0:
                pixel_mask_array[y * width + x] = 1
                solid_pixel_count += 1
            else:
                pixel_mask_array[y * width + x] = 0
    var tile_columns: int = int((width + TILE_SIZE_PIXELS - 1) / TILE_SIZE_PIXELS)
    var tile_rows: int = int((height + TILE_SIZE_PIXELS - 1) / TILE_SIZE_PIXELS)
    var tile_solid_array: PackedByteArray = PackedByteArray()
    tile_solid_array.resize(tile_columns * tile_rows)
    for ty in range(tile_rows):
        for tx in range(tile_columns):
            var any_solid: bool = false
            var start_y = ty * TILE_SIZE_PIXELS
            var end_y = min((ty + 1) * TILE_SIZE_PIXELS, height)
            var start_x = tx * TILE_SIZE_PIXELS
            var end_x = min((tx + 1) * TILE_SIZE_PIXELS, width)
            for py in range(start_y, end_y):
                for px in range(start_x, end_x):
                    if pixel_mask_array[py * width + px] == 1:
                        any_solid = true
                        break
                if any_solid:
                    break
            tile_solid_array[ty * tile_columns + tx] = 1 if any_solid else 0

    var visited_array: PackedByteArray = PackedByteArray()
    visited_array.resize(tile_columns * tile_rows)
    var regions_list: Array = []
    var directions = [Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]
    for ty in range(tile_rows):
        for tx in range(tile_columns):
            var index = ty * tile_columns + tx
            if tile_solid_array[index] == 1 and visited_array[index] == 0:
                visited_array[index] = 1
                var region_queue: Array = [Vector2i(tx, ty)]
                var region_tile_list: Array = []
                while region_queue.size() > 0:
                    var cell: Vector2i = region_queue.pop_back()
                    region_tile_list.append(cell)
                    for dir in directions:
                        var nx = cell.x + dir.x
                        var ny = cell.y + dir.y
                        if nx >= 0 and ny >= 0 and nx < tile_columns and ny < tile_rows:
                            var nidx = ny * tile_columns + nx
                            if tile_solid_array[nidx] == 1 and visited_array[nidx] == 0:
                                visited_array[nidx] = 1
                                region_queue.append(Vector2i(nx, ny))
                regions_list.append(region_tile_list)

    var region_boundaries: Array = []
    for region_tiles in regions_list:
        var boundary_tiles: Array = []
        for tile in region_tiles:
            for direction in directions:
                var neighbor_x: int = tile.x + direction.x
                var neighbor_y: int = tile.y + direction.y
                var in_bounds: bool = (
                    neighbor_x >= 0
                    and neighbor_y >= 0
                    and neighbor_x < tile_columns
                    and neighbor_y < tile_rows
                )
                var neighbor_is_solid: bool = false
                if in_bounds:
                    var neighbor_index: int = neighbor_y * tile_columns + neighbor_x
                    neighbor_is_solid = (tile_solid_array[neighbor_index] == 1)
                if not neighbor_is_solid:
                    boundary_tiles.append(tile)
                    break
        region_boundaries.append(boundary_tiles)

    var collision_mask_polygons: Array = []
    for boundary_tiles in region_boundaries:
        var center_points: Array = []
        for tile in boundary_tiles:
            var center_x = tile.x * TILE_SIZE_PIXELS + TILE_SIZE_PIXELS * 0.5
            # invert y so that tile.y=0 (bottom row) maps to y=height
            var center_y = (
                (iResolution.y as int) - (tile.y * TILE_SIZE_PIXELS + TILE_SIZE_PIXELS * 0.5)
            )
            center_points.append(Vector2(center_x, center_y))
        var hull_points: Array = _compute_convex_hull(center_points)
        collision_mask_polygons.append(hull_points)
        var body = StaticBody2D.new()
        add_child(body)
        var poly = CollisionPolygon2D.new()
        poly.polygon = hull_points
        body.add_child(poly)
        collision_mask_polygons.append(poly)
    print("Spawned", collision_mask_polygons.size(), "collision bodies.")


func _compute_convex_hull(tile_center_points: Array) -> Array:
    var point_count: int = tile_center_points.size()
    if point_count < 3:
        return tile_center_points.duplicate()
    var leftmost_index: int = 0
    for i in range(1, point_count):
        var p: Vector2 = tile_center_points[i]
        var q: Vector2 = tile_center_points[leftmost_index]
        if p.x < q.x:
            leftmost_index = i
        elif p.x == q.x:
            if p.y < q.y:
                leftmost_index = i
    var hull_points: Array = []
    var current_index: int = leftmost_index
    while true:
        hull_points.append(tile_center_points[current_index])
        var next_index: int = (current_index + 1) % point_count
        for j in range(point_count):
            if (
                _orientation(
                    tile_center_points[current_index],
                    tile_center_points[next_index],
                    tile_center_points[j]
                )
                == -1
            ):
                next_index = j
        current_index = next_index
        if current_index == leftmost_index:
            break
    return hull_points


func _orientation(a: Vector2, b: Vector2, c: Vector2) -> int:
    var cross_product: float = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    if cross_product > 0.0:
        return -1
    elif cross_product < 0.0:
        return 1
    else:
        return 0
