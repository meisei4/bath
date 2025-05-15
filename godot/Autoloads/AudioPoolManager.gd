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

    # create and configure each pooled player
    sfx_pool = AudioPool.new()
    sfx_pool.pool_size = SFX_POOL_SIZE
    sfx_pool.bus = AudioBus.BUS.SFX
    sfx_pool.loop_on_end = false
    add_child(sfx_pool)

    music_pool = AudioPool.new()
    music_pool.pool_size = MUSIC_POOL_SIZE
    music_pool.bus = AudioBus.BUS.MUSIC
    music_pool.loop_on_end = true  # automatically loop if you want continuous music
    add_child(music_pool)

    input_pool = AudioPool.new()
    input_pool.pool_size = INPUT_POOL_SIZE
    input_pool.bus = AudioBus.BUS.INPUT
    input_pool.loop_on_end = false
    add_child(input_pool)


func _setup_buses(buses: Array[AudioBus.BUS]) -> void:
    var current_bus_count: int = AudioServer.get_bus_count()
    for i in range(current_bus_count, buses.size()):
        AudioServer.add_bus()
    for i in range(buses.size()):
        AudioServer.set_bus_name(i, AudioBus.val(buses[i]))


func _set_bus_volumes() -> void:
    for bus in bus_volumes.keys():
        AudioServer.set_bus_volume_db(AudioBus.get_bus_index(bus), bus_volumes[bus])


func play_sfx(sound_resource: AudioStream, volume_db: float = 0.0) -> void:
    sfx_pool.play(sound_resource, volume_db)


func stop_all_sfx() -> void:
    sfx_pool.stop_all()  # you can add a stop_all() inside AudioPool similarly


func play_music(music_resource: AudioStream, volume_db: float = 0.0) -> void:
    music_pool.play(music_resource, volume_db)


func stop_music() -> void:
    music_pool.stop_all()  # same as above


func is_music_playing() -> bool:
    return music_pool.is_any_playing()


func play_input(input_resource: AudioStream, volume_db: float = 0.0) -> void:
    input_pool.play(input_resource, volume_db)
