extends Node2D
class_name CharacterComponents

@export var active_sprite: Sprite2D = null
@export var active_collision_shape: CollisionShape2D = null
@export var active_animation: AnimationPlayer = null
@export var active_animation_sprite: AnimatedSprite2D = null
@export var active_speed: float
@export var active_mechanics: Array[Mechanic] = []


func _ready() -> void:
    self._initialize()


func _initialize() -> void:
    self._update_upgrade()


func _update_upgrade() -> void:
    if self.active_sprite:
        self.active_sprite.queue_free()  #Free the current sprite
    var character_data: CharacterComponentsResource = ComponentsManager.character_components[0]

    self.active_sprite = Sprite2D.new()
    self.active_sprite.set_texture(character_data.sprite_texture)
    add_child(self.active_sprite)

    self.active_collision_shape = CollisionShape2D.new()
    self.active_collision_shape.set_shape(character_data.collision_shape)

    self.active_speed = character_data.speed

    #TODO: this is for later studies related to REsource handling.
    #currently its fine to just keep Mechanic's as Nodes and not yet as Resources
    #Resources will allow for real time updating/configuring and runtime loading of configurations of mechanics
    #^^the cost to this is having to create a scene/.tscn for each mechanic or create a
    #extends "Resource" "MechanicData" class for every Mechanic and that would only be needed later
    # when the game perhaps has more dynamic mechanics loading and upgrading features
    for mechanic_scene in character_data.mechanics:
        var mechanic_node: Node = mechanic_scene.instantiate()
        if mechanic_node is Mechanic:
            mechanic_node.character = self
            add_child(mechanic_node)
            active_mechanics.append(mechanic_node)
