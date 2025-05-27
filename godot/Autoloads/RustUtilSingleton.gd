extends Node
#class_name RustUtilSingleton

var rust_util: RustUtil


func _ready() -> void:
    rust_util = RustUtil.new()
