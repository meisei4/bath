extends Node
class_name CapsuleDummyGen


func _ready() -> void:
    var sprite_width: int = 16
    var sprite_height: int = 24
    var image: Image = Image.create(sprite_width, sprite_height, false, Image.FORMAT_RGBA8)

    var shape: Array[String] = [
        "      XXXX      ",
        "    X......X    ",
        "  X..........X  ",
        " X.....WW.....X ",
        "X..............X",
        "X..............X",
        "X..............X",
        "X..............X",
        "X..............X",
        "X..............X",
        "X..............X",
        "X..............X",
        "X..............X",
        "X..............X",
        "X..............X",
        "X..............X",
        "X..............X",
        "X..............X",
        "X..............X",
        "X..............X",
        " X............X ",
        "  X..........X  ",
        "    X......X    ",
        "      XXXX      "
    ]
    var opaque_pixels: PackedVector2Array = PackedVector2Array()
    for y: int in range(shape.size()):
        var line: String = shape[y]
        for x: int in range(sprite_width):
            var character: String = line[x]
            var pixel_color: Color = Color(0, 0, 0, 0)
            if character == "X":
                pixel_color = Color.BLACK
                var center_x: float = x - sprite_width * 0.5 + 0.5
                var center_y: float = y - sprite_height * 0.5 + 0.5
                opaque_pixels.append(Vector2(center_x, center_y))
            elif character == "W":
                pixel_color = Color.DARK_GRAY
            elif character == ".":
                pixel_color = Color.WHITE

            image.set_pixel(x, y, pixel_color)

    image.save_png("res://Assets/Sprites/capsule.png")
    var texture: ImageTexture = ImageTexture.create_from_image(image)

    var character_body: CharacterBody2D = CharacterBody2D.new()
    character_body.name = "CharacterBody2D"

    var sprite: Sprite2D = Sprite2D.new()
    sprite.set_texture_filter(CanvasItem.TextureFilter.TEXTURE_FILTER_NEAREST)  #TODO: This is the mipmap thing, not in the image itself but the sprite
    sprite.name = "Sprite2D"
    sprite.texture = texture
    sprite.centered = true

    #TODO: this next part is a race condition because the image wont be saved in time, so you have to run this twice lol
    sprite.texture = load("res://Assets/Sprites/capsule.png")
    character_body.add_child(sprite)
    sprite.owner = character_body

    var convex_hull: PackedVector2Array = Geometry2D.convex_hull(opaque_pixels)
    var collision: CollisionShape2D = CollisionShape2D.new()
    collision.name = "CollisionShape2D"
    var convex_polygon: ConvexPolygonShape2D = ConvexPolygonShape2D.new()
    convex_polygon.points = convex_hull
    collision.shape = convex_polygon
    collision.position = Vector2.ZERO
    character_body.add_child(collision)
    collision.owner = character_body

    var capsule_script: Script = load("res://Entities/Characters/CapsuleDummy.gd") as Script
    character_body.set_script(capsule_script)

    var scene: PackedScene = PackedScene.new()
    scene.pack(character_body)
    ResourceSaver.save(scene, "res://TestScenes/Entities/Characters/CapsuleDummy.tscn")
