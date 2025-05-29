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

var cfg: ConfigFile

var resolution: Vector2

const CFG_PATH: String = "res://experimental_resolution_override.cfg"
const DEFAULT_SECTION: String = "display_default"
const EXPERIMENTAL_SECTION: String = "display_experimental"
var DEFAULT_VIEWPORT_WIDTH: int = ProjectSettings.get_setting("display/window/size/viewport_width")
var DEFAULT_VIEWPORT_HEIGHT: int = ProjectSettings.get_setting("display/window/size/viewport_height")
var DEFAULT_WINDOW_OVERRIDE_WIDTH: int = ProjectSettings.get_setting("display/window/size/window_width_override")
var DEFAULT_WINDOW_OVERRIDE_HEIGHT: int = ProjectSettings.get_setting("display/window/size/window_height_override")

func _ready() -> void:
    cfg = ConfigFile.new()
    var err: int = cfg.load(CFG_PATH)
    resolution.x = DEFAULT_VIEWPORT_WIDTH
    resolution.y = DEFAULT_VIEWPORT_HEIGHT
    _apply_resolution()


func _apply_resolution() -> void:
    var section: String = DEFAULT_SECTION
    var scene_root: Node = get_tree().get_current_scene()
    if scene_root and scene_root.is_in_group(EXPERIMENTAL_SECTION):
        section = EXPERIMENTAL_SECTION

    var viewport_width: int = cfg.get_value(section, "window/size/viewport_width", DEFAULT_VIEWPORT_WIDTH)
    var viewport_height: int = cfg.get_value(section, "window/size/viewport_height", DEFAULT_VIEWPORT_HEIGHT)
    var window_width_override: int = cfg.get_value(section, "window/size/window_width_override", DEFAULT_WINDOW_OVERRIDE_WIDTH)
    var window_height_override: int = cfg.get_value(section, "window/size/window_height_override", DEFAULT_WINDOW_OVERRIDE_HEIGHT)
    resolution = Vector2(viewport_width, viewport_height)
    print("ResolutionManager: applying viewport %sx%s and window %sx%s for '%s'" % [viewport_width, viewport_height, window_width_override, window_height_override, section])
    DisplayServer.window_set_size(Vector2(window_width_override, window_height_override))
    var window: Window = get_tree().root
    window.set_content_scale_size(Vector2i(viewport_width, viewport_height))
    window.set_content_scale_mode(Window.ContentScaleMode.CONTENT_SCALE_MODE_VIEWPORT)
    window.set_content_scale_aspect(Window.ContentScaleAspect.CONTENT_SCALE_ASPECT_KEEP)
    window.set_content_scale_stretch(Window.ContentScaleStretch.CONTENT_SCALE_STRETCH_INTEGER)
    window.set_content_scale_factor(1.0)


func get_resolution() -> Vector2:
    return resolution


func get_window_override_size() -> Vector2:
    return Vector2(
        int(cfg.get_value(resolution_section(), "window/size/window_width_override", DEFAULT_WINDOW_OVERRIDE_WIDTH)),
        int(cfg.get_value(resolution_section(), "window/size/window_height_override", DEFAULT_WINDOW_OVERRIDE_HEIGHT))
    )


func resolution_section() -> String:
    var scene_root: Node = get_tree().get_current_scene()
    return EXPERIMENTAL_SECTION if scene_root and scene_root.is_in_group(EXPERIMENTAL_SECTION) else DEFAULT_SECTION
