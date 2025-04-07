extends Node
#TODO: autoloads cant be class named in file
#class_name AudioManager

const SFX_POOL_SIZE: int = 12
const MUSIC_POOL_SIZE: int = 5

const bus_volumes: Dictionary[AudioBus.BUS, float] = {
    AudioBus.BUS.MASTER: 0.0, AudioBus.BUS.SFX: 0.0, AudioBus.BUS.MUSIC: 0.0
}

var sfx_pool: Array[AudioStreamPlayer] = []
var available_sfx_players: Array[AudioStreamPlayer] = []

var music_pool: Array[AudioStreamPlayer] = []
var available_music_players: Array[AudioStreamPlayer] = []


func _ready() -> void:
    _setup_buses([AudioBus.BUS.MASTER, AudioBus.BUS.SFX, AudioBus.BUS.MUSIC])
    _set_bus_volumes()
    _initialize_sfx_pool()
    _initialize_music_pool()


func _setup_buses(buses: Array[AudioBus.BUS]) -> void:
    var current_bus_count: int = AudioServer.get_bus_count()
    for i: int in range(current_bus_count, buses.size()):
        AudioServer.add_bus()
    for i: int in range(buses.size()):
        AudioServer.set_bus_name(i, AudioBus.val(buses[i]))


#TODO: this only sets the volumes of buses based on their Bus types,
#TODO: figure out if we want to be able to set individual buses in the object pool to certain volumes regardless of Bus type
func _set_bus_volumes() -> void:
    for bus: AudioBus.BUS in bus_volumes.keys():
        var bus_idx: int = AudioBus.get_bus_index(bus)
        AudioServer.set_bus_volume_db(bus_idx, bus_volumes[bus])


func _initialize_sfx_pool() -> void:
    for _i: int in range(SFX_POOL_SIZE):
        var sfx_player: AudioStreamPlayer = AudioStreamPlayer.new()
        sfx_player.bus = AudioBus.val(AudioBus.BUS.SFX)
        sfx_player.finished.connect(_on_sfx_finished.bind(sfx_player))
        add_child(sfx_player)
        sfx_pool.append(sfx_player)
        available_sfx_players.append(sfx_player)


func _initialize_music_pool() -> void:
    for _i: int in range(MUSIC_POOL_SIZE):
        var music_player: AudioStreamPlayer = AudioStreamPlayer.new()
        music_player.bus = AudioBus.val(AudioBus.BUS.MUSIC)
        music_player.finished.connect(_on_music_finished.bind(music_player))
        add_child(music_player)
        music_pool.append(music_player)
        available_music_players.append(music_player)


func acquire_sfx_player() -> AudioStreamPlayer:
    if available_sfx_players.is_empty():
        push_warning("No available SFX AudioStreamPlayers.")
        return null
    return available_sfx_players.pop_back()


func acquire_music_player() -> AudioStreamPlayer:
    if available_music_players.is_empty():
        push_warning("No available Music AudioStreamPlayers.")
        return null
    return available_music_players.pop_back()


func release_sfx_player(player: AudioStreamPlayer) -> void:
    player.stop()
    player.stream = null
    available_sfx_players.append(player)


func release_music_player(player: AudioStreamPlayer) -> void:
    player.stop()
    player.stream = null
    available_music_players.append(player)


func play_sfx(sound_resource: AudioStream, volume_db: float = 0.0) -> void:
    var player: AudioStreamPlayer = acquire_sfx_player()
    if player == null:
        print("Failed to play SFX: Pool exhausted.")
        push_warning("Failed to play SFX: Pool exhausted.")
        return
    player.bus = AudioBus.val(AudioBus.BUS.SFX)
    player.stream = sound_resource
    player.volume_db = volume_db
    player.play()


func stop_sfx(sfx_name: String) -> void:
    for player: AudioStreamPlayer in sfx_pool:
        if player.playing and player.stream is Resource and player.stream.resource_path == sfx_name:
            player.stop()
            release_sfx_player(player)
            break


func stop_all_sfx() -> void:
    for player: AudioStreamPlayer in sfx_pool:
        if player.playing:
            player.stop()
            release_sfx_player(player)


func play_music(music_resource: AudioStream, volume_db: float = 0.0) -> void:
    var player: AudioStreamPlayer = acquire_music_player()
    if player == null:
        push_warning("Failed to play Music: Pool exhausted.")
        return
    player.stream = music_resource
    player.volume_db = volume_db
    player.play()


func stop_music() -> void:
    for player: AudioStreamPlayer in music_pool:
        if player.playing:
            player.stop()
            release_music_player(player)


func is_music_playing() -> bool:
    for player: AudioStreamPlayer in music_pool:
        if player.playing:
            return true
    return false


func _on_sfx_finished(player: AudioStreamPlayer) -> void:
    release_sfx_player(player)


func _on_music_finished(player: AudioStreamPlayer) -> void:
    if player.stream:
        player.play()
    else:
        release_music_player(player)
