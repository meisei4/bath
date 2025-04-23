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
        " X.....XX.....X ",
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

    for y: int in range(shape.size()):
        var line: String = shape[y]
        for x: int in range(sprite_width):
            var char: String = line[x]
            var pixel_color: Color = Color(0, 0, 0, 0)
            if char == "X" or char == "x":
                pixel_color = Color.BLACK
            elif char == ".":
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
    sprite.texture = ResourceLoader.load("res://Assets/Sprites/capsule.png")
    character_body.add_child(sprite)
    sprite.owner = character_body

    var collision: CollisionShape2D = CollisionShape2D.new()
    collision.name = "CollisionShape2D"
    var rect_shape: RectangleShape2D = RectangleShape2D.new()
    rect_shape.extents = Vector2(sprite_width, sprite_height)
    collision.shape = rect_shape
    collision.position = Vector2(0, 0)
    character_body.add_child(collision)
    collision.owner = character_body

    var capsule_script: Script = (
        ResourceLoader.load("res://godot/Test/Mechanics/CapsuleDummy.gd") as Script
    )
    character_body.set_script(capsule_script)

    var scene: PackedScene = PackedScene.new()
    scene.pack(character_body)
    ResourceSaver.save(scene, "res://godot/Test/Mechanics/CapsuleDummy.tscn")
