extends Node
#class_name ResolutionManager

# Resolution hierarchy:
# 1. viewport size (internal render resolution)
# 2. window override size (OS window dimensions)
# 3. stretch mode (e.g. "canvas_items")
#
# To learn more, see:
# https://docs.godotengine.org/ja/4.x/classes/class_projectsettings.html

# Switching resolutions:
# - In the editor, copy the desired settings from your custom config file
#   (e.g., res://experimental_resolution_override.cfg)
#   into the [display] section of your godot.project file.
# - For web builds, ensure the same values are applied before export.

# TODO: update this singleton to actually parse the custom config file with per scene specificaitons of resolutions
# https://docs.godotengine.org/en/stable/classes/class_configfile.html.

# that way we can fully control the project settings procedurally and explicitly
# instead of messing with the project gui shit and its link with the godot.project file

var resolution: Vector2


func _ready() -> void:
    resolution.x = ProjectSettings.get_setting("display/window/size/viewport_width")
    resolution.y = ProjectSettings.get_setting("display/window/size/viewport_height")
    print("Game resolution initialized to ", resolution)
