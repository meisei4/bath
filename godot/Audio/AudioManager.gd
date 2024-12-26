extends Node

const SFX_POOL_SIZE: int = 20
const MUSIC_POOL_SIZE: int = 5

var sfx_pool: Array[AudioStreamPlayer] = []
var available_sfx_players: Array[AudioStreamPlayer] = []

var music_pool: Array[AudioStreamPlayer] = []
var available_music_players: Array[AudioStreamPlayer] = []

# TODO: godot enums suck, like they seriously suck, or else im the dumbest person on earth
#var MASTER: String = AudioBus.val(AudioBus.BUS.MASTER)
#var SFX: String = AudioBus.val(AudioBus.BUS.SFX)
#var MUSIC: String = AudioBus.val(AudioBus.BUS.MUSIC)
const MASTER: String = "Master"
const SFX: String = "SFX"
const MUSIC: String = "Music"


func _ready() -> void:
    _setup_buses([MASTER, SFX, MUSIC])
    _set_bus_volumes()
    _initialize_pools()


func _setup_buses(bus_names: Array[String]) -> void:
    var current_bus_count: int = AudioServer.get_bus_count()
    for i: int in range(current_bus_count, bus_names.size()):
        AudioServer.add_bus()
    for i: int in range(bus_names.size()):
        AudioServer.set_bus_name(i, bus_names[i])


func _set_bus_volumes() -> void:
    var bus_volumes: Dictionary = {MASTER: 0.0, SFX: -3.0, MUSIC: -6.0}
    for bus_name in bus_volumes.keys():
        var bus_idx: int = AudioServer.get_bus_index(bus_name)
        if bus_idx != -1:
            AudioServer.set_bus_volume_db(bus_idx, bus_volumes[bus_name])
        else:
            push_warning("Bus not found: " + bus_name)


func _initialize_pools() -> void:
    _initialize_sfx_pool()
    _initialize_music_pool()


func _initialize_sfx_pool() -> void:
    for _i: int in range(SFX_POOL_SIZE):
        var sfx_player: AudioStreamPlayer = AudioStreamPlayer.new()
        sfx_player.bus = SFX
        sfx_player.finished.connect(_on_sfx_finished.bind(sfx_player))
        add_child(sfx_player)
        sfx_pool.append(sfx_player)
        available_sfx_players.append(sfx_player)


func _initialize_music_pool() -> void:
    for _i: int in range(MUSIC_POOL_SIZE):
        var music_player: AudioStreamPlayer = AudioStreamPlayer.new()
        music_player.bus = MUSIC
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


func play_sfx(sound_resource: AudioStream, volume_db: float = 0.0, bus_name: String = SFX) -> void:
    var player: AudioStreamPlayer = acquire_sfx_player()
    if player == null:
        push_warning("Failed to play SFX: Pool exhausted.")
        return
    _route_sound_to_bus(player, bus_name)
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


func _route_sound_to_bus(player: AudioStreamPlayer, bus_name: String) -> void:
    var bus_index: int = AudioServer.get_bus_index(bus_name)
    if bus_index == -1:
        push_warning("Cannot route sound to non-existent bus: " + bus_name)
    else:
        player.bus = bus_name


func _on_sfx_finished(player: AudioStreamPlayer) -> void:
    release_sfx_player(player)


func _on_music_finished(player: AudioStreamPlayer) -> void:
    if player.stream:
        player.play()
    else:
        release_music_player(player)
