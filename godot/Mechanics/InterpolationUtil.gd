extends Node


static func smooth_damp(
    current: float, target: float, velocity: float, smooth_time: float, delta: float
) -> Vector2:
    var omega: float = 2.0 / smooth_time
    var x: float = omega * delta
    var exp_factor: float = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x)
    var change: float = current - target
    var temp: float = (velocity + omega * change) * delta
    velocity = (velocity - omega * temp) * exp_factor
    var output: float = target + (change + temp) * exp_factor
    return Vector2(output, velocity)


static func depth_normal_pow(current_depth: float, max_depth: float) -> float:
    var depth: float = clamp(-current_depth / abs(max_depth), 0.0, 1.0)
    return pow(depth, 2.2)


static func depth_normal(current_depth: float, max_depth: float) -> float:
    return clamp(-current_depth / abs(max_depth), 0.0, 1.0)
