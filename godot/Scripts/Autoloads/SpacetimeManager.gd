extends Node
#class_name SpacetimeContext

const METERS_PER_WORLD_UNIT: float = 1.0
const PIXELS_PER_REAL_METER: float = 16.0
const PIXELS_PER_WORLD_UNIT: float = METERS_PER_WORLD_UNIT * PIXELS_PER_REAL_METER
const TIME_SCALE: float = 1.0
const UNIVERSAL_TIME_OFFSET: float = 0.0
const EARTH_GRAVITY: float = 9.81
const ARCADEYNESS_FACTOR: float = 2.55
const GRAVITY: float = EARTH_GRAVITY * ARCADEYNESS_FACTOR
const UNIVERSAL_DRAG_COEFFICIENT: float = 0.0
const FLUID_DENSITY: float = 0.0
const COSMIC_FRICTION: float = 0.0
const MAGNETIC_FIELD_STRENGTH: float = 0.0
const ELECTRIC_FIELD_STRENGTH: float = 0.0
const SOLAR_WIND_PRESSURE: float = 0.0
const ORBITAL_GRAVITY_FACTOR: float = 0.0
const UNIVERSAL_ROTATION_SPEED: float = 0.0


func to_physical_space(world_distance_units: float) -> float:
    return world_distance_units * PIXELS_PER_WORLD_UNIT


func to_world_space(pixel_distance: float) -> float:
    return pixel_distance / PIXELS_PER_WORLD_UNIT


func apply_time_scale(real_delta_seconds: float) -> float:
    return real_delta_seconds * TIME_SCALE


func apply_universal_drag(current_velocity: float, delta_seconds: float) -> float:
    if UNIVERSAL_DRAG_COEFFICIENT <= 0.0:
        return current_velocity

    var factor: float = 1.0 - (UNIVERSAL_DRAG_COEFFICIENT * delta_seconds)
    if factor < 0.0:
        factor = 0.0
    return current_velocity * factor


func normalize_value(value: float, min_val: float, max_val: float) -> float:
    if max_val - min_val == 0.0:
        return 0.0
    return (value - min_val) / (max_val - min_val)


func denormalize_value(norm_value: float, min_val: float, max_val: float) -> float:
    return (norm_value * (max_val - min_val)) + min_val
