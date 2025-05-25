extends Node2D
class_name Main

const TEST_SCENES_DIRECTORY: String = "res://TestScenes"

const CLI_SCENE_FLAG: String = "--scene"
const URL_PARAM_SCENE_KEY: String = "scene"

const FEATURE_WEB: String    = "web"
const FEATURE_WINDOWS: String = "windows"
const FEATURE_LINUX: String   = "linux"
const FEATURE_MACOS: String   = "macos"
const FEATURE_ARM: String   = "arm"

var HARDCODED_TEST_SCENES: PackedStringArray = PackedStringArray([
    "res://TestScenes/Audio/ManualRhythmOnsetRecorder.tscn",
    "res://TestScenes/Audio/PitchDimension.tscn",
    "res://TestScenes/Mechanics/MechanicsTest.tscn",
    "res://TestScenes/Shaders/Compute/CollisionMask.tscn",
    "res://TestScenes/Shaders/Compute/PerspectiveTiltMask.tscn",
    "res://TestScenes/Shaders/Glacier/GlacierFlow.tscn",
    "res://TestScenes/Shaders/Shadows/ShadowsTest.tscn",
    "res://TestScenes/TestHarness.tscn"
])

enum Platform {
    PLATFORM_WEB,
    PLATFORM_ARM,
    PLATFORM_WINDOWS,
    PLATFORM_LINUX,
    PLATFORM_MACOS,
    PLATFORM_UNKNOWN
}

func _ready() -> void:
    var scenes_to_load: PackedStringArray = _determine_scenes_to_load()
    for scene_path: String in scenes_to_load:
        _load_and_add_scene(scene_path)


func _load_and_add_scene(scene_path: String) -> void:
    var packed: PackedScene = load(scene_path) as PackedScene
    var inst: Node = packed.instantiate()
    add_child(inst)


func _determine_scenes_to_load() -> PackedStringArray:
    match _get_platform():
        Platform.PLATFORM_WEB:
            return _scenes_from_url()
        _:
            return _scenes_from_cli()


func _scenes_from_url() -> PackedStringArray:
    var full_url: String = JavaScriptBridge.eval("window.location.href") as String
    var key: String = _extract_url_parameter(full_url, URL_PARAM_SCENE_KEY)
    if key != "":
        var path: String = _find_matching_scene(key)
        if path != "":
            return PackedStringArray([ path ])
    return HARDCODED_TEST_SCENES.duplicate() as PackedStringArray


func _scenes_from_cli() -> PackedStringArray:
    var args: PackedStringArray = OS.get_cmdline_args()
    var idx: int = args.find(CLI_SCENE_FLAG)
    if idx >= 0 and idx + 1 < args.size():
        return PackedStringArray([ args[idx + 1] ])
    return HARDCODED_TEST_SCENES.duplicate() as PackedStringArray


func _get_platform() -> int:
    if OS.has_feature(FEATURE_WEB):
        return Platform.PLATFORM_WEB
    elif OS.has_feature(FEATURE_ARM):
        return Platform.PLATFORM_ARM
    elif OS.has_feature(FEATURE_WINDOWS):
        return Platform.PLATFORM_WINDOWS
    elif OS.has_feature(FEATURE_LINUX):
        return Platform.PLATFORM_LINUX
    elif OS.has_feature(FEATURE_MACOS):
        return Platform.PLATFORM_MACOS
    return Platform.PLATFORM_UNKNOWN


func _extract_url_parameter(url: String, param: String) -> String:
    var qidx: int = url.find("?")
    if qidx < 0:
        return ""
    var query: String = url.substr(qidx + 1, url.length() - qidx - 1)
    for pair: String in query.split("&"):
        var kv: Array[String] = pair.split("=")
        if kv.size() == 2 and kv[0] == param:
            return kv[1]
    return ""


func _find_matching_scene(key: String) -> String:
    for path: String in HARDCODED_TEST_SCENES:
        if path.ends_with("%s.tscn" % key) or path == key:
            return path
    return ""
