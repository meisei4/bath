extends Node2D
class_name OccluderDebugDrawer

var occluder_polygon: Array[Vector2] = []

var line_width: float = 2.0
var line_color: Color = Color.RED

func _draw() -> void:
    if occluder_polygon.size() >= 2:
        for i in range(occluder_polygon.size() - 1):
            var start_point = occluder_polygon[i]
            var end_point = occluder_polygon[i + 1]
            draw_line(start_point, end_point, line_color, line_width)

        # Close the polygon by connecting the last point to the first
        draw_line(occluder_polygon[occluder_polygon.size() - 1], occluder_polygon[0], line_color, line_width)
