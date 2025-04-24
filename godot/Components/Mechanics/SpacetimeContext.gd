extends Node
class_name SpacetimeContext

#TODO: idk, just trying to have fun, none of this actually does anything lol

# The idea here is to define the relationship between:
#  1) Real-world meters/seconds,
#  2) "World units" that your game might use internally,
#  3) The number of pixels used to display 1 meter or 1 world unit on-screen.
#
# Additional constants can be used to scale any "real" physics values
# to create more arcadey or stylized behavior.

# The ratio of real meters to your "world units."
#    If you want 1 world unit == 1 real meter, set this to 1.0.
#    If you want 1 world unit == 0.5 real meters, set it to 0.5, etc.
const METERS_PER_WORLD_UNIT: float = 1.0

# The ratio of real meters to on-screen pixels.
#    For example, 1 real meter might correspond to 16 pixels on screen.
#    This is arbitrary—adjust as needed.
const PIXELS_PER_REAL_METER: float = 16.0

# If you want your "world units" to be the main measure instead of meters,
# you can use this combined factor for convenience:
#  (how many pixels in 1 world unit) = METERS_PER_WORLD_UNIT * PIXELS_PER_REAL_METER
const PIXELS_PER_WORLD_UNIT: float = METERS_PER_WORLD_UNIT * PIXELS_PER_REAL_METER

# For controlling time flow. If TIME_SCALE = 1.0,
# then 1 second of real time = 1 second of in-game time.
# If TIME_SCALE = 2.0, then the game runs at "double speed."
const TIME_SCALE: float = 0.5

# A universal time offset (e.g., for network synchronization or for offsetting
# a game clock from real time). Usually 0 for local single-player contexts.
const UNIVERSAL_TIME_OFFSET: float = 0.0

# The real-world gravitational acceleration near Earth’s surface is ~9.81 m/s^2.
# We can define an "arcadeyness factor" to exaggerate or diminish that gravity.
#
# Example: ARCADEYNESS_FACTOR = 2.55  =>   effective gravity = 9.81 * 2.55 ≈ 25.0
#
# You can do the same for drag, friction, etc.

const EARTH_GRAVITY: float = 9.81  # m/s^2 in real world
const ARCADEYNESS_FACTOR: float = 2.55  # exaggeration factor

const GRAVITY: float = EARTH_GRAVITY * ARCADEYNESS_FACTOR

const UNIVERSAL_DRAG_COEFFICIENT: float = 0.0
const FLUID_DENSITY: float = 0.0
const COSMIC_FRICTION: float = 0.0
const MAGNETIC_FIELD_STRENGTH: float = 0.0
const ELECTRIC_FIELD_STRENGTH: float = 0.0
const SOLAR_WIND_PRESSURE: float = 0.0
const ORBITAL_GRAVITY_FACTOR: float = 0.0
const UNIVERSAL_ROTATION_SPEED: float = 0.0

const QUANTUM_INSTABILITY_PROBABILITY: float = 0.0
const WAVEFUNCTION_COLLAPSE_FACTOR: float = 0.0
const PLANCK_LENGTH_SCALE: float = 0.0000000000000000000000000000000000000000001


static func to_physical_space(world_distance_units: float) -> float:
    """
    Converts a distance in "world units" into on-screen pixels.
    (If you prefer to think in meters -> pixels, do that conversion
     outside or define separate function for meters -> pixels.)
    """
    return world_distance_units * PIXELS_PER_WORLD_UNIT


static func to_world_space(pixel_distance: float) -> float:
    """
    Converts a distance in on-screen pixels back to "world units."
    """
    return pixel_distance / PIXELS_PER_WORLD_UNIT


static func apply_time_scale(real_delta_seconds: float) -> float:
    """
    Scales real-world delta time into game time, based on TIME_SCALE.
    """
    return real_delta_seconds * TIME_SCALE


static func apply_universal_drag(current_velocity: float, delta_seconds: float) -> float:
    """
    Applies a linear drag effect to the velocity over the given time step.
    If UNIVERSAL_DRAG_COEFFICIENT is 0.0, no drag is applied.
    """
    if UNIVERSAL_DRAG_COEFFICIENT <= 0.0:
        return current_velocity

    var factor: float = 1.0 - (UNIVERSAL_DRAG_COEFFICIENT * delta_seconds)
    if factor < 0.0:
        factor = 0.0
    return current_velocity * factor


static func random_quantum_tunnel_check() -> bool:
    """
    Returns 'true' with probability QUANTUM_INSTABILITY_PROBABILITY.
    Could be used to flip velocities, cause random teleports, etc.
    """
    if QUANTUM_INSTABILITY_PROBABILITY <= 0.0:
        return false
    return randf() < QUANTUM_INSTABILITY_PROBABILITY


static func normalize_value(value: float, min_val: float, max_val: float) -> float:
    """
    Maps 'value' from the range [min_val, max_val] to [0, 1].
    """
    if max_val - min_val == 0.0:
        return 0.0
    return (value - min_val) / (max_val - min_val)


static func denormalize_value(norm_value: float, min_val: float, max_val: float) -> float:
    """
    Maps 'norm_value' from the range [0, 1] to [min_val, max_val].
    """
    return (norm_value * (max_val - min_val)) + min_val
