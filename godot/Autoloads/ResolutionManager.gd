extends Node
#class_name ResolutionManager

#TODO: MAKE SURE THAT YOU HAVE OVERRIDE SETTINGS ON OR OFF DEPENDING ON SCENE CONTEXT!!!
#----------------------see file res://experimental_resolution_override.cfg----------------
#TODO: renderer settings do not update post godot runtime
# the only things that can be changed after godot runtime is the OS level variables, the godot render settings must be
#overridden with a separate config file to be ran at runtime:
# SEE: https://docs.godotengine.org/ja/4.x/classes/class_projectsettings.html#class-projectsettings-property-application-config-project-settings-override
#TODO: lol, this will come to bite me with porting to custom devices/raspi

# This autoload provides a single source of truth for our “game” resolution.
# In Project Settings → Display → Window:
#   • viewport_width  = 256      ← internal render resolution
#   • viewport_height = 384
#   • window_width_override  = 1024  ← actual OS window size (OVERRIDE_SCALE×256)
#   • window_height_override = 1536  ← actual OS window size (×384)
#   • stretch/mode = "canvas_items"
#
# Godot will render everything at 256×384 internally, then automatically
# stretch all CanvasItems by 4× to fill the 1024×1536 window.
# By reading from ProjectSettings here, both our fragment shaders and
# compute shaders can use the exact same resolution, avoiding any mismatch.
#TODO: BELOW ARE FAKE NUMBERS FOR RESOLUTION REFERENCES, THEY DO NOTHING, JUST HERE TO REMIND DEVELOPER OF res://experimental_resolution_override.cfg
#const BATH_WIDTH: int = 256
#const BATH_HEIGHT: int = 384
#const OVERRIDE_SCALE: int = 3
#const bath_size: Vector2i = Vector2i(BATH_WIDTH, BATH_HEIGHT)

#const EXPERIMENTAL_WIDTH: int = 855
#const EXPERIMENTAL_HEIGHT: int = 480
#const experimental_size: Vector2i = Vector2i(EXPERIMENTAL_WIDTH, EXPERIMENTAL_HEIGHT)

var resolution: Vector2


func _ready() -> void:
    var w: int = ProjectSettings.get_setting("display/window/size/viewport_width")
    var h: int = ProjectSettings.get_setting("display/window/size/viewport_height")
    resolution = Vector2(w, h)
    print("Game resolution initialized to ", resolution)

#TODO: Im sure i could hack together a fucked up scene that sets up the override project settings somehow lmao
#https://docs.godotengine.org/en/stable/classes/class_configfile.html
