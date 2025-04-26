extends Node

# This autoload provides a single source of truth for our “game” resolution.
# In Project Settings → Display → Window:
#   • viewport_width  = 256      ← internal render resolution
#   • viewport_height = 384
#   • window_width_override  = 1024  ← actual OS window size (4×256)
#   • window_height_override = 1536  ← actual OS window size (4×384)
#   • stretch/mode = "canvas_items"
#
# Godot will render everything at 256×384 internally, then automatically
# stretch all CanvasItems by 4× to fill the 1024×1536 window.
# By reading from ProjectSettings here, both our fragment shaders and
# compute shaders can use the exact same resolution, avoiding any mismatch.

var resolution: Vector2


func _ready() -> void:
    var w: int = ProjectSettings.get_setting("display/window/size/viewport_width")
    var h: int = ProjectSettings.get_setting("display/window/size/viewport_height")
    resolution = Vector2(w, h)
    print("Game resolution initialized to ", resolution)  # Expect (256, 384)
