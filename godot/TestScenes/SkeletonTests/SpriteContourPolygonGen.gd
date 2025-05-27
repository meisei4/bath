extends Node2D
class_name SpriteContourPolygonGen

var sprite_texture_path: String = "res://Assets/Sprites/Dolphin2.png"
var save_path: String = "res://TestScenes/SkeletonTests/SpriteContourPolygon.tscn"

var polygon_node: Polygon2D


func _ready() -> void:
    var texture: Texture2D = load(sprite_texture_path) as Texture2D
    polygon_node = Polygon2D.new()
    polygon_node.texture = texture
    #TODO: THESE ARE COPY PASTED FROM THE PYTHON CONTOURING ALGORITHM!!!
    var uvs_outward_shift: PackedVector2Array = [
        Vector2(0.4488, -0.0073),
        Vector2(0.3917, 0.0193),
        Vector2(0.3917, 0.0365),
        Vector2(0.3547, 0.0537),
        Vector2(0.3547, 0.1055),
        Vector2(0.2806, 0.1400),
        Vector2(0.2806, 0.1572),
        Vector2(0.2436, 0.1744),
        Vector2(0.2436, 0.2262),
        Vector2(0.2065, 0.2434),
        Vector2(0.2065, 0.3124),
        Vector2(0.0213, 0.3986),
        Vector2(0.0213, 0.4158),
        Vector2(-0.0157, 0.4331),
        Vector2(-0.0157, 0.5152),
        Vector2(0.0556, 0.5431),
        Vector2(0.1268, 0.5152),
        Vector2(0.1268, 0.4980),
        Vector2(0.1808, 0.4728),
        Vector2(0.1895, 0.4728),
        Vector2(0.2065, 0.4807),
        Vector2(0.2065, 0.6014),
        Vector2(0.2436, 0.6187),
        Vector2(0.2436, 0.6876),
        Vector2(0.2806, 0.7049),
        Vector2(0.2806, 0.7738),
        Vector2(0.3176, 0.7911),
        Vector2(0.3176, 0.8256),
        Vector2(0.3547, 0.8428),
        Vector2(0.3547, 0.8773),
        Vector2(0.3889, 0.8879),
        Vector2(0.3747, 0.8892),
        Vector2(0.3377, 0.8892),
        Vector2(0.2065, 0.9503),
        Vector2(0.2146, 0.9863),
        Vector2(0.3660, 0.9901),
        Vector2(0.4031, 0.9728),
        Vector2(0.5969, 0.9728),
        Vector2(0.6340, 0.9901),
        Vector2(0.7854, 0.9863),
        Vector2(0.7935, 0.9503),
        Vector2(0.6623, 0.8892),
        Vector2(0.6253, 0.8892),
        Vector2(0.6111, 0.8879),
        Vector2(0.6453, 0.8773),
        Vector2(0.6453, 0.8428),
        Vector2(0.6824, 0.8256),
        Vector2(0.6824, 0.7911),
        Vector2(0.7194, 0.7738),
        Vector2(0.7194, 0.7049),
        Vector2(0.7564, 0.6876),
        Vector2(0.7564, 0.6187),
        Vector2(0.7935, 0.6014),
        Vector2(0.7935, 0.4807),
        Vector2(0.8105, 0.4728),
        Vector2(0.8192, 0.4728),
        Vector2(0.8732, 0.4980),
        Vector2(0.8732, 0.5152),
        Vector2(0.9444, 0.5431),
        Vector2(1.0157, 0.5152),
        Vector2(1.0157, 0.4331),
        Vector2(0.9787, 0.4158),
        Vector2(0.9787, 0.3986),
        Vector2(0.7935, 0.3124),
        Vector2(0.7935, 0.2434),
        Vector2(0.7564, 0.2262),
        Vector2(0.7564, 0.1744),
        Vector2(0.7194, 0.1572),
        Vector2(0.7194, 0.1400),
        Vector2(0.6453, 0.1055),
        Vector2(0.6453, 0.0537),
        Vector2(0.6083, 0.0365),
        Vector2(0.6083, 0.0193),
        Vector2(0.5512, -0.0073),
    ]
    #TODO: ^^^^THESE ARE COPY PASTED FROM THE PYTHON CONTOURING ALGORITHM!!!

    var texture_width: float = texture.get_width()
    var texture_height: float = texture.get_height()

    var model_coords: Array[Vector2] = []
    for uv in uvs_outward_shift:
        var model_coord: Vector2 = uv * Vector2(texture_width, texture_height)
        model_coords.append(model_coord)

    polygon_node.polygon = model_coords
    polygon_node.uv = model_coords  #TODO: these arent UVs normalized eww, its the same as model apparently

    var packed_scene: PackedScene = PackedScene.new()
    packed_scene.pack(polygon_node)

    var save_result: int = ResourceSaver.save(packed_scene, save_path)
