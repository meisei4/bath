extends Node

var character_components: Array[CharacterComponentsResource] = []


func _ready() -> void:
    load_upgrades()


func load_upgrades() -> void:
    var character_components_dir: String = "res://Resources/CharacterComponents/"
    var dir: DirAccess = DirAccess.open(character_components_dir)
    dir.list_dir_begin()
    var file_name: String = dir.get_next()
    while file_name != "":
        var resource_path: String = character_components_dir + file_name
        if file_name.ends_with(".tres"):
            #TODO: figure out better resource error checking??
            var resource: CharacterComponentsResource = ResourceLoader.load(resource_path)
            character_components.append(resource)
        file_name = dir.get_next()
    dir.list_dir_end()
    character_components.sort_custom(
        func(x: CharacterComponentsResource, y: CharacterComponentsResource) -> bool:
            return x.order < y.order
    )
