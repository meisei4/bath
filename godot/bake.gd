extends Node


func bake(root_node: Node = null) -> void:
    if root_node == null:
        root_node = get_tree().get_root()

    var bake_scenes_directory: String = "res://Baked/Scenes"
    var bake_resources_directory: String = "res://Baked/Resources"
    DirAccess.make_dir_recursive_absolute(bake_scenes_directory)
    DirAccess.make_dir_recursive_absolute(bake_resources_directory)
    var used_names: Dictionary[String, bool] = {}
    var node_stack: Array[Node] = []
    var ancestry_stack: Array[PackedStringArray] = []
    node_stack.append(root_node)
    ancestry_stack.append(PackedStringArray())
    while node_stack.size() > 0:
        var current_node: Node = node_stack.pop_back()
        var current_ancestry: PackedStringArray = ancestry_stack.pop_back()
        var base_name: String
        var script_reference: Script = current_node.get_script()
        if (
            script_reference != null
            and script_reference.resource_path != ""
            and ResourceLoader.exists(script_reference.resource_path)
        ):
            var script_class_name: String = script_reference.get_class()
            if script_class_name != "":
                base_name = script_class_name
            else:
                base_name = script_reference.resource_path.get_file().get_basename()
        else:
            base_name = current_node.get_class()

        var prefix: String = ""
        if current_ancestry.size() > 0:
            prefix = String("_").join(current_ancestry) + "_"

        var candidate_name: String = prefix + base_name
        var unique_name: String = candidate_name
        var name_counter: int = 1
        while used_names.has(unique_name):
            unique_name = "%s_%d" % [candidate_name, name_counter]
            name_counter += 1

        current_node.name = unique_name
        used_names[unique_name] = true

        for child_node in current_node.get_children():
            node_stack.append(child_node)
            var new_ancestry: PackedStringArray = current_ancestry.duplicate()
            new_ancestry.append(base_name)
            ancestry_stack.append(new_ancestry)

    for child_node in root_node.get_children():
        child_node.owner = root_node
        var ownership_queue: Array[Node] = []
        ownership_queue.append(child_node)
        while ownership_queue.size() > 0:
            var descendant_node: Node = ownership_queue.pop_back()
            descendant_node.owner = root_node
            for grandchild_node in descendant_node.get_children():
                ownership_queue.append(grandchild_node)

    var resource_scan_queue: Array[Node] = []
    resource_scan_queue.append(root_node)
    while resource_scan_queue.size() > 0:
        var scanned_node: Node = resource_scan_queue.pop_back()
        var property_list: Array[Dictionary] = scanned_node.get_property_list()

        for property_info in property_list:
            var property_name: String = property_info.name
            var property_value: Variant = scanned_node.get(property_name)
            if property_value is Resource:
                var resource_instance: Resource = property_value as Resource
                print(
                    (
                        "→ Node '%s' [%s] has Resource property '%s' (class: %s)"
                        % [
                            scanned_node.name,
                            scanned_node.get_class(),
                            property_name,
                            resource_instance.get_class()
                        ]
                    )
                )

        for property_info in property_list:
            if property_info.type == TYPE_OBJECT:
                var property_name: String = property_info.name
                var property_value: Variant = scanned_node.get(property_name)
                if property_value is Resource and (property_value as Resource).resource_path == "":
                    var runtime_resource: Resource = property_value as Resource
                    var resource_class_name: String = runtime_resource.get_class()
                    var base_file_path: String = (
                        bake_resources_directory
                        + "/"
                        + scanned_node.name
                        + "_"
                        + property_name
                        + "_"
                        + resource_class_name
                    )
                    var file_path: String = base_file_path + ".tres"
                    var suffix_counter: int = 1
                    while FileAccess.file_exists(file_path):
                        file_path = base_file_path + "_" + var_to_str(suffix_counter) + ".tres"
                        suffix_counter += 1
                    ResourceSaver.save(
                        runtime_resource,
                        file_path,
                        ResourceSaver.FLAG_BUNDLE_RESOURCES | ResourceSaver.FLAG_CHANGE_PATH
                    )
                    print("  • Saved runtime resource '%s' → %s" % [property_name, file_path])

        for child_node in scanned_node.get_children():
            resource_scan_queue.append(child_node)

    var packed_scene: PackedScene = PackedScene.new()
    packed_scene.pack(root_node)
    var output_path: String = bake_scenes_directory + "/" + root_node.name + ".tscn"
    ResourceSaver.save(packed_scene, output_path, ResourceSaver.FLAG_BUNDLE_RESOURCES)
    print("Baked full scene → " + output_path)


################

var procedural_script_found: bool = false


func bake_better() -> void:
    print("Starting bake for: %s" % name)

    var cloned_root: Node = clone_and_bake_node(self)

    set_owner_recursively(cloned_root, cloned_root)

    var packed_scene: PackedScene = PackedScene.new()
    var pack_status: int = packed_scene.pack(cloned_root)
    if pack_status != OK:
        push_error("Pack failed: %d" % pack_status)
        return

    var output_folder: String = "res://Baked/Scenes"
    DirAccess.make_dir_recursive_absolute(output_folder)

    var scene_file_path: String = "%s/%s.tscn" % [output_folder, cloned_root.name]
    var save_flags: int = ResourceSaver.FLAG_NONE
    if procedural_script_found:
        save_flags = ResourceSaver.FLAG_BUNDLE_RESOURCES

    var save_status: int = ResourceSaver.save(packed_scene, scene_file_path, save_flags)
    if save_status != OK:
        push_error("Save failed: %d" % save_status)
    else:
        print("Scene saved to %s" % scene_file_path)


func clone_and_bake_node(original_node: Node) -> Node:
    var new_node: Node = original_node.duplicate(Node.DUPLICATE_USE_INSTANTIATION)
    bake_script_if_needed(new_node)

    for i in new_node.get_child_count():
        var child: Node = new_node.get_child(i)
        var baked_child: Node = clone_and_bake_node(child)
        new_node.remove_child(child)
        new_node.add_child(baked_child)
        baked_child.name = child.name

    return new_node


func bake_script_if_needed(node: Node) -> void:
    var script_resource: GDScript = node.get_script() as GDScript
    if script_resource and script_resource.has_source_code():
        var source_code: String = script_resource.get_source_code()

        if source_code.contains("class_name") and source_code.contains("Baked"):
            print("Skipping already baked script on node: %s" % node.name)
            return

        if source_code.contains("add_child("):
            procedural_script_found = true
            print("Baking procedural script on node: %s" % node.name)

            var source_lines: PackedStringArray = source_code.split("\n")
            var output_lines: PackedStringArray = []
            var flag_inserted: bool = false
            var in_ready: bool = false
            var ready_indent: String = ""

            for line in source_lines:
                if not flag_inserted and line.begins_with("class_name "):
                    output_lines.append("@export var baked_skip_ready: bool = true")
                    flag_inserted = true
                    continue

                if line.strip_edges().begins_with("func _ready"):
                    output_lines.append(line)
                    in_ready = true
                    ready_indent = line.substr(0, line.find("func"))
                    continue

                if in_ready:
                    if not line.begins_with(ready_indent + "    "):
                        in_ready = false
                    elif line.contains("add_child("):
                        output_lines.append("%s    if not baked_skip_ready:" % ready_indent)
                        output_lines.append(line)
                        continue

                output_lines.append(line)

            var new_source: String = String("\n").join(output_lines)
            var baked_script: GDScript = GDScript.new()
            baked_script.set_source_code(new_source)
            var reload_status: int = baked_script.reload(true)
            if reload_status == OK:
                node.set_script(baked_script)
                print("Assigned baked script to node: %s" % node.name)
            else:
                push_error("Script reload failed on node: %s" % node.name)


func set_owner_recursively(current_node: Node, owner_node: Node) -> void:
    for child_node in current_node.get_children():
        child_node.owner = owner_node
        set_owner_recursively(child_node, owner_node)
