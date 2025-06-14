extends Node
class_name Mechanic

enum TYPE { SWIM = 0, LATERAL_MOVEMENT = 1, JUMP = 2 }

signal state_completed(completed_state: MechanicController.STATE)

var type: TYPE

var mechanic_controller: MechanicController
