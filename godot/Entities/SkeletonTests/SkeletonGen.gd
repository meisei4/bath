extends Node2D
class_name BoneRigGenerator

# --- Constants for Texture Creation ---
const PATTERN_WIDTH: int = 5
const PATTERN_HEIGHT: int = 7

const ROOT_BONE_NAME: String = "root_bone"
const CHILD_BONE_NAME: String = "child_bone"

# Colors
const WHITE_COLOR: Color = Color.WHITE        # Opaque white for 'o'
const GREY_COLOR: Color = Color.DIM_GRAY     # Grey for '*' outline

# Path to save the generated texture (ensure the directory exists)
const TEXTURE_SAVE_PATH: String = "res://Assets/Sprites/generated_bone_sprite.png"

const BONE_LENGTH: float = 32.0  # Length of each bone in pixels

func _ready() -> void:
    var texture = generate_bone_texture()

    var sprite = Sprite2D.new()
    sprite.texture = texture
    sprite.position = Vector2.ZERO
    sprite.scale = Vector2(10, 10)  # Scale up for better visibility if needed
    add_child(sprite)

    # Step 3: Create the Skeleton2D
    var skeleton = Skeleton2D.new()
    skeleton.name = "BoneSkeleton"
    skeleton.position = Vector2.ZERO
    add_child(skeleton)

    # Step 4: Create Bone2D Nodes
    var root_bone = Bone2D.new()
    root_bone.position = Vector2.ZERO  # Origin
    root_bone.length = BONE_LENGTH
    skeleton.add_child(root_bone)

    var child_bone = Bone2D.new()
    child_bone.position = Vector2(BONE_LENGTH, 0)  # Positioned at end of root bone
    child_bone.length = BONE_LENGTH
    root_bone.add_child(child_bone)

    # Step 5: Create Polygon2D Node and Assign to Skeleton
    var polygon = Polygon2D.new()
    polygon.polygon = generate_polygon_points()
    polygon.color = WHITE_COLOR
    polygon.position = Vector2.ZERO
    add_child(polygon)

    # Assign the Skeleton to the Polygon
    polygon.skeleton = skeleton

    # Step 6: Create a Skin2D and Assign Bones

    # Retrieve Bone2D nodes
    var root_bone_node = skeleton.get_node(ROOT_BONE_NAME) as Bone2D
    var child_bone_node = skeleton.get_node(CHILD_BONE_NAME) as Bone2D

    if root_bone_node and child_bone_node:
        skeleton.add_bone(root_bone_node)
        skeleton.add_bone(child_bone_node)
    else:
        push_error("Bone nodes not found in the skeleton.")
        return

    # Step 7: Save the Texture as a PNG File (Optional)
    var image = texture.get_data()
    image.flip_y()  # Flip Y-axis if necessary
    var save_result = image.save_png(TEXTURE_SAVE_PATH)
    if save_result != OK:
        push_error("Failed to save the generated texture to " + TEXTURE_SAVE_PATH)
    else:
        print("Texture saved successfully at " + TEXTURE_SAVE_PATH)

    # Step 8: Set Up a Simple Animation (Optional)
    setup_animation(skeleton, root_bone)

    # Inform the User
    print("Bone/Skeleton rigging setup completed successfully.")

# --- Function to Generate the Bone Texture ---
func generate_bone_texture() -> ImageTexture:
    # Define the bone pixel pattern
    var pattern = [
        "*****",
        "*o*o*",
        "**o**",
        "**o**",
        "**o**",
        "*o*o*",
        "*****"
    ]

    # Define constants for pattern size and colors
    const PATTERN_WIDTH = 5
    const PATTERN_HEIGHT = 7
    const WHITE_COLOR = Color(1, 1, 1, 1)
    const GREY_COLOR = Color(0.5, 0.5, 0.5, 1)

    # Create a new Image with exact pattern size
    var image = Image.new()
    image.create(PATTERN_WIDTH, PATTERN_HEIGHT, false, Image.FORMAT_RGBA8)
    image.fill(Color(0, 0, 0, 0))  # Transparent background

    # Iterate through the pattern and set pixels
    for y in range(PATTERN_HEIGHT):
        var row = pattern[y]
        for x in range(PATTERN_WIDTH):
            var char = row[x]
            var pixel_color = null
            match char:
                'o':
                    pixel_color = WHITE_COLOR
                '*':
                    pixel_color = GREY_COLOR
                _:
                    pixel_color = null
            if pixel_color != null:
                image.set_pixel(x, y, pixel_color)

    # Create an ImageTexture from the Image
    var texture = ImageTexture.new()
    texture.create_from_image(image)

    return texture

# --- Function to Generate Polygon Points ---
func generate_polygon_points() -> PackedVector2Array:
    # Define a simple polygon that matches the bone's shape
    # Adjust points as needed for more accurate deformation
    var points = PackedVector2Array([
        Vector2(-2, -3),
        Vector2(2, -3),
        Vector2(2, 3),
        Vector2(-2, 3)
    ])
    return points

# --- Function to Set Up a Simple Animation ---
func setup_animation(skeleton: Skeleton2D, root_bone: Bone2D) -> void:
    var animation_player = AnimationPlayer.new()
    animation_player.name = "AnimationPlayer"
    add_child(animation_player)

    var animation = Animation.new()
    animation.length = 2.0  # Duration in seconds
    animation.loop = true

    # Add a rotation track for the RootBone
    var track_idx = animation.add_track(Animation.TYPE_VALUE)
    animation.track_set_path(track_idx, "BoneSkeleton:" + ROOT_BONE_NAME + ":rotation")
    animation.track_insert_key(track_idx, 0.0, 0.0)                    # Start at 0 radians
    animation.track_insert_key(track_idx, 1.0, deg2rad(45))          # Rotate to 45 degrees
    animation.track_insert_key(track_idx, 2.0, deg2rad(-45))         # Rotate to -45 degrees

    # Assign the animation to the AnimationPlayer
    animation_player.add_animation("BoneSwing", animation)
    animation_player.play("BoneSwing")

# --- Utility Functions ---
func deg2rad(degrees: float) -> float:
    return degrees * (PI / 180.0)

func rad2deg(radians: float) -> float:
    return radians * (180.0 / PI)
