extends Node
#class_name CollisionMaskTargetsManager

var iTime: float

var ice_sheets: IceSheets

signal ice_sheets_entered_scene(ice_sheets: IceSheets)

func register_ice_sheets(_ice_sheets: IceSheets) -> void:
    self.ice_sheets = _ice_sheets
    ice_sheets_entered_scene.emit(_ice_sheets)
