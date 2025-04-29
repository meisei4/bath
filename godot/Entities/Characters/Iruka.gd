extends CharacterBody2D
class_name Iruka

@onready var character_components: CharacterComponents = $CharacterComponents


func _init() -> void:
    self.set_physics_process(false)
    self.set_process(false)


func _ready() -> void:
    self.add_to_group("player")
    self._initialize()
    self._setup_signals()
    self.set_physics_process(true)
    self.set_process(true)


func _setup_signals() -> void:
    pass


func _initialize() -> void:
    self.z_index = -1
    var collision_shape: CollisionShape2D = self.character_components.active_collision_shape
    self.add_child(collision_shape)


func _process(_delta: float) -> void:
    self._handle_movement()
    self._handle_actions()


func _handle_movement() -> void:
    var input_vector: Vector2 = Input.get_vector("left", "right", "up", "down")
    var speed: float = self.character_components.active_speed
    self.velocity = input_vector * speed

    self.move_and_slide()  #velocity is handled in here

    var collision_count: int = self.get_slide_collision_count()
    for i: int in range(collision_count):
        var collision: KinematicCollision2D = self.get_slide_collision(i)


func _handle_actions() -> void:
    #if Input.is_action_just_pressed("jump"):
    pass
    #if Input.is_action_just_pressed("dive"):
    #    pass
