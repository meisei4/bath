extends Node2D
class_name CharacterComponents

@export var active_sprite: Sprite2D = null
@export var active_collision_shape: CollisionShape2D = null
@export var active_animation: AnimationPlayer = null
@export var active_animation_sprite: AnimatedSprite2D = null
@export var active_speed: float

func _ready() -> void:
    self._initialize()

func _initialize() -> void:
    self._update_upgrade()

func _update_upgrade() -> void:
    if self.active_sprite:
        self.active_plane_sprite.queue_free()  #Free the current sprite
    var character_data: CharacterData = ComponentsManager.character_components[0]

    self.active_sprite = Sprite2D.new()
    self.active_sprite.set_texture(character_data.sprite_texture)
    add_child(self.active_sprite)

    self.active_collision_shape = CollisionShape2D.new()
    self.active_collision_shape.set_shape(character_data.collision_shape)

    self.active_speed = character_data.speed
