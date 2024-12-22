extends Node

const POOL_SIZE: int = 10
var audio_pool: Array[AudioStreamPlayer] = []
var available_stream_players: Array[AudioStreamPlayer] = []

func _ready() -> void:
    setup_buses(["Master", "SFX", "Music"])
    set_bus_volumes({"Master": 0.0, "SFX": -3.0, "Music": -6.0})
    for i: int in range(POOL_SIZE):
        var player: AudioStreamPlayer = AudioStreamPlayer.new()
        player.bus = "SFX"
        player.finished.connect(self._on_audio_finished.bind(player))
        add_child(player)
        audio_pool.append(player)
        available_stream_players.append(player)

func setup_buses(bus_names: Array[String]) -> void:
    var current_bus_count: int = AudioServer.get_bus_count()
    for i: int in range(current_bus_count, bus_names.size()):
        AudioServer.add_bus()
    for i: int in range(bus_names.size()):
        AudioServer.set_bus_name(i, bus_names[i])

func set_bus_volumes(volumes_db: Dictionary) -> void:
    for bus_name: String in volumes_db.keys():
        var bus_idx: int = AudioServer.get_bus_index(bus_name)
        AudioServer.set_bus_volume_db(bus_idx, volumes_db[bus_name])

func play_sound(sound_res: Resource, volume: float = 0.6, bus_name: String = "SFX") -> void:
    if available_stream_players.is_empty():
        push_warning("No free AudioStreamPlayers.")
        return
    var player: AudioStreamPlayer = available_stream_players.pop_back()
    route_sound_to_bus(player, bus_name)
    player.stream = sound_res
    player.volume_db = linear_to_db(volume)
    player.play()

func route_sound_to_bus(player: AudioStreamPlayer, bus_name: String) -> void:
    var bus_idx: int = AudioServer.get_bus_index(bus_name)
    if bus_idx == -1:
        push_warning("not a bus, cant route.")
    else:
        player.bus = bus_name

func _on_audio_finished(player: AudioStreamPlayer) -> void:
    player.stop()
    player.stream = null
    available_stream_players.append(player)

func stop_sound(sfx_name: String) -> void:
    for player: AudioStreamPlayer in audio_pool:
        if player.playing and player.stream.resource_path == sfx_name:
            player.stop()
            available_stream_players.append(player)
            return


func stop_all_sounds() -> void:
    for player: AudioStreamPlayer in audio_pool:
        if player.playing:
            player.stop()
            available_stream_players.append(player)

func linear_to_db(linear: float) -> float:
    return -80.0 if linear <= 0.0 else 20.0 * log(linear)
