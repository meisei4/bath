extends Resource
class_name JumpData

@export var OVERRIDE_GRAVITY: float = 0.0
# If 0, uses SpacetimeContext.GRAVITY
# Units: px/s²

@export var INITIAL_JUMP_VELOCITY: float = 12.0
# Controls jump height: higher velocity → higher apex
# Units: px/s

@export var FORWARD_SPEED: float = 6.0
# Horizontal movement while airborne
# Units: world units/s

@export var SPRITE_SCALE_AT_MAX_ALTITUDE: float = 2.5
# Max sprite scale at jump apex
# Range: >1.0 for foreshortening illusion

@export var MAXIMUM_TILT_ANGLE_ACHIEVED_AT_IMMEDIATE_ASCENSION_AND_FINAL_DESCENT: float = 0.785398
# Max lean angle at takeoff/landing (radians)
# 0.785398 = 45°

@export var FOCAL_LENGTH: float = 1.0
# Smaller = more dramatic perspective squish during the jump
# Larger = flatter projection (less pseudo-3D)

#TODO: figure out where to control phase duration in the jump, and also look into easing functions because

# I have no idea how to integrate easing in an intuitive way that also is smart with the CPU <-> GPU state relationship
@export var ASCENT_PHASE: float = 0.0
@export var APEX_PHASE: float = 0.0
@export var DESCENT_PHASE: float = 0.0
#^^must always add up to 1.0
