extends Node
#TODO: autoloads cant be class named in file
#class_name AudioBus

enum BUS { MASTER = 0, SFX = 1, MUSIC = 2, INPUT = 3 }

@export var bus: BUS = BUS.MASTER


func val(_bus: BUS) -> StringName:
    match _bus:
        BUS.MASTER:
            return "Master"
        BUS.SFX:
            return "SFX"
        BUS.MUSIC:
            return "Music"
        BUS.INPUT:
            return "Input"
        _:
            return ""


func get_bus_index(_bus: BUS) -> int:
    var bus_name: StringName = AudioBus.val(_bus)
    var bus_idx: int = AudioServer.get_bus_index(bus_name)
    if bus_idx == -1:
        push_warning("Bus not found: " + bus_name)
    return bus_idx
