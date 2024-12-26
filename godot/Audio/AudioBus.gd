extends Node

enum BUS { MASTER = 0, SFX = 1, MUSIC = 2 }

@export var bus: BUS = BUS.MASTER


#TODO: hahaha you idiot, nice try. It wont work, and probably shouldnt even work, GlacierCellState is forced)
func val(_bus: BUS) -> StringName:
    match _bus:
        BUS.MASTER:
            return "Master"
        BUS.SFX:
            return "SFX"
        BUS.MUSIC:
            return "Music"
        _:
            return ""
