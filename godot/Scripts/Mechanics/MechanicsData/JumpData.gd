extends Resource
class_name JumpData

@export var OVERRIDE_GRAVITY: float = 0.0
# If 0, uses SpacetimeContext.GRAVITY
# Units: px/s

@export var INITIAL_VERTICAL_POSITION: float = 0.0

@export var INITIAL_JUMP_VELOCITY: float = 8.0
# Controls jump height: higher velocity -> higher apex
# Units: px/s

@export var FORWARD_VELOCITY: float = 12.0
# Horizontal movement while airborne
# Units: world units/s

#TODO: figure out where to control phase duration in the jump, and also look into easing functions because
