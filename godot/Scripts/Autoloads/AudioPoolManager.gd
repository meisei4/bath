extends Node
#class_name AudioPoolManager

const SFX_POOL_SIZE: int = 12
const MUSIC_POOL_SIZE: int = 5
const INPUT_POOL_SIZE: int = 1
var sfx_pool: AudioPool
var music_pool: AudioPool
var input_pool: AudioPool
const bus_volumes: Dictionary[int, float] = {
    AudioBus.MASTER: 0.0,
    AudioBus.SFX: 0.0,
    AudioBus.MUSIC: 0.0,
    AudioBus.INPUT: 0.0,
}


func _ready() -> void:
    _setup_buses([AudioBus.MASTER, AudioBus.SFX, AudioBus.MUSIC, AudioBus.INPUT])
    _set_bus_volumes()
    sfx_pool = AudioPool.new()
    sfx_pool.pool_size = SFX_POOL_SIZE
    sfx_pool.bus = AudioBus.SFX
    sfx_pool.loop_on_end = false
    add_child(sfx_pool)
    sfx_pool.owner = self

    music_pool = AudioPool.new()
    music_pool.pool_size = MUSIC_POOL_SIZE
    music_pool.bus = AudioBus.MUSIC
    music_pool.loop_on_end = true
    add_child(music_pool)
    music_pool.owner = self

    input_pool = AudioPool.new()
    input_pool.pool_size = INPUT_POOL_SIZE
    input_pool.bus = AudioBus.INPUT
    input_pool.loop_on_end = false
    add_child(input_pool)
    input_pool.owner = self


func _setup_buses(buses: Array[int]) -> void:
    var current_bus_count: int = AudioServer.get_bus_count()
    for i: int in range(current_bus_count, buses.size()):
        AudioServer.add_bus()
    for i: int in range(buses.size()):
        AudioServer.set_bus_name(i, AudioBus.val(buses[i]))


func _set_bus_volumes() -> void:
    for bus: int in bus_volumes.keys():
        AudioServer.set_bus_volume_db(AudioBus.get_bus_index(bus), bus_volumes[bus])


func play_sfx(sound_resource: AudioStream, volume_db: float = 0.0) -> void:
    sfx_pool.play(sound_resource, volume_db)


func play_music(music_resource: AudioStream, volume_db: float = 0.0) -> void:
    music_pool.play(music_resource, volume_db)


func play_input(input_resource: AudioStream, volume_db: float = 0.0) -> void:
    input_pool.play(input_resource, volume_db)
