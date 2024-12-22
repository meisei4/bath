extends CharacterBody2D
class_name Obstacle

@export var path_speed: float = 0.0  # Speed along the path

func _ready() -> void:
    self._initialize()
    self._setup_signals()

func _setup_signals() -> void:
    pass

func _initialize() -> void:
    #self.health.set_current_health(1)
    var collision_shape: CollisionShape2D = self.upgrade_component.active_collision_shape
    self.add_child(collision_shape)

    self.path_speed = self.upgrade_component.active_speed  #TODO: this seems sloppy, not sure where to best control speed yet
