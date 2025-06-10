extends Node
#class_name AudioPoolManager

const SFX_POOL_SIZE: int = 12
const MUSIC_POOL_SIZE: int = 5
const INPUT_POOL_SIZE: int = 1

const bus_volumes: Dictionary[AudioBus.BUS, float] = {
    AudioBus.BUS.MASTER: 0.0,
    AudioBus.BUS.SFX: 0.0,
    AudioBus.BUS.MUSIC: 0.0,
    AudioBus.BUS.INPUT: 0.0,
}

var sfx_pool: AudioPool
var music_pool: AudioPool
var input_pool: AudioPool


func _ready() -> void:
    _setup_buses([AudioBus.BUS.MASTER, AudioBus.BUS.SFX, AudioBus.BUS.MUSIC, AudioBus.BUS.INPUT])
    _set_bus_volumes()
    sfx_pool = AudioPool.new()
    sfx_pool.pool_size = SFX_POOL_SIZE
    sfx_pool.bus = AudioBus.BUS.SFX
    sfx_pool.loop_on_end = false
    add_child(sfx_pool)

    music_pool = AudioPool.new()
    music_pool.pool_size = MUSIC_POOL_SIZE
    music_pool.bus = AudioBus.BUS.MUSIC
    music_pool.loop_on_end = true
    add_child(music_pool)

    input_pool = AudioPool.new()
    input_pool.pool_size = INPUT_POOL_SIZE
    input_pool.bus = AudioBus.BUS.INPUT
    input_pool.loop_on_end = false
    add_child(input_pool)


func _setup_buses(buses: Array[AudioBus.BUS]) -> void:
    var current_bus_count: int = AudioServer.get_bus_count()
    for i: int in range(current_bus_count, buses.size()):
        AudioServer.add_bus()
    for i: int in range(buses.size()):
        AudioServer.set_bus_name(i, AudioBus.val(buses[i]))


func _set_bus_volumes() -> void:
    for bus: AudioBus.BUS in bus_volumes.keys():
        AudioServer.set_bus_volume_db(AudioBus.get_bus_index(bus), bus_volumes[bus])


func play_sfx(sound_resource: AudioStream, volume_db: float = 0.0) -> void:
    sfx_pool.play(sound_resource, volume_db)


func play_music(music_resource: AudioStream, volume_db: float = 0.0) -> void:
    music_pool.play(music_resource, volume_db)


func play_input(input_resource: AudioStream, volume_db: float = 0.0) -> void:
    input_pool.play(input_resource, volume_db)
