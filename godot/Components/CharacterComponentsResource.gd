extends Resource
class_name CharacterComponentsResource

#Cant export Node types in Resources, only NodePaths

@export var order: int
@export var sprite_texture: Texture2D
@export var collision_shape: RectangleShape2D
@export var speed: float
@export var animation_sprites: SpriteFrames  #TODO; figure out animations later
@export var mechanics: Array[PackedScene] = []
