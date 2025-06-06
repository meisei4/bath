extends Resource
class_name GlacierCellState
#NONE (0)   <-->   INTACT (1)   -->   FRACTURED (2)   -->   ICEBERG (3)
enum STATE { NONE = 0, INTACT = 1, FRACTURED = 2, ICEBERG = 3 }

@export var state: STATE = STATE.NONE


func get_color(_state: STATE) -> Color:
    match _state:
        STATE.INTACT:
            return Color.WHITE
        STATE.FRACTURED:
            return Color.GRAY
        STATE.ICEBERG:
            return Color.ALICE_BLUE
        _:
            return Color.BLACK  # Default color for NONE or invalid state
