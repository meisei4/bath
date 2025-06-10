extends Node2D
#class_name SceneManager

#TODO: try the following urls, and or any other scenes
# in order to change scenes you need to reload the webpage with the new #/ url

# http://localhost:8060/tmp_js_export.html#/TestScenes/Shaders/Shape/GhostShape
# http://localhost:8060/tmp_js_export.html#/TestScenes/TestHarness


func _ready() -> void:
    if OS.get_name() == "Web" or OS.has_feature("wasm32") or OS.has_feature("web"):
        #TODO: this is assuming http://localhost:8060/tmp_js_export.html#/ is always in front?
        #substring(2) to cut off the "#/" prefix
        var scene_path: String = JavaScriptBridge.eval("location.hash.substring(2)")
        var resource_path: String = "res://" + scene_path + ".tscn"
        _load_scene(resource_path)
    else:
        print("not running web mode, you're fine")


func _load_scene(resource_path: String) -> void:
    if not ResourceLoader.exists(resource_path):
        push_error(
            (
                " CUSTOM ERROR!! Scene not found: %s, could be empty path upon intialization, or scene path is wrong"
                % resource_path
            )
        )
    else:
        #get_tree().change_scene_to_file(resource_path)
        var scene_resource: PackedScene = load(resource_path) as PackedScene
        get_tree().change_scene_to_packed(scene_resource)
        #TODO: this isnt working, but its kind of stupid to have a main scene, just build thw eb build targetting a different main scene everytime
        ResolutionManager._apply_resolution()
