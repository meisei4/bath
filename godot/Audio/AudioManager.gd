extends Node

const SFX_POOL_SIZE: int = 10

var sfxPool: Array[AudioStreamPlayer] = []
var availableSfxPlayers: Array[AudioStreamPlayer] = []

var musicPlayer: AudioStreamPlayer = null

func _ready() -> void:
    setupBuses(["Master", "SFX", "Music"])
    setBusVolumes({"Master": 0.0, "SFX": -3.0, "Music": -6.0})
    initializeSfxPool()
    initializeMusicPlayer()

func setupBuses(busNames: Array[String]) -> void:
    var currentBusCount: int = AudioServer.get_bus_count()
    var totalBuses: int = busNames.size()

    for index in range(currentBusCount, totalBuses):
        AudioServer.add_bus()

    for index in range(totalBuses):
        AudioServer.set_bus_name(index, busNames[index])

func setBusVolumes(volumesDb: Dictionary) -> void:
    for busName in volumesDb.keys():
        var busIndex: int = AudioServer.get_bus_index(busName)
        if busIndex != -1:
            AudioServer.set_bus_volume_db(busIndex, volumesDb[busName])
        else:
            push_warning("Bus not found: " + busName)

# --- SFX Management ---
func initializeSfxPool() -> void:
    for _i in range(SFX_POOL_SIZE):
        var sfxPlayer: AudioStreamPlayer = AudioStreamPlayer.new()
        sfxPlayer.bus = "SFX"
        sfxPlayer.finished.connect(_onSfxFinished.bind(sfxPlayer))
        add_child(sfxPlayer)
        sfxPool.append(sfxPlayer)
        availableSfxPlayers.append(sfxPlayer)

func playSfx(soundResource: Resource, volumeDb: float, busName: String) -> void:
    var player: AudioStreamPlayer = getAvailableSfxPlayer()
    if player == null:
        push_warning("No available SFX AudioStreamPlayers.")
        return
    routeSoundToBus(player, busName)
    player.stream = soundResource
    player.volume_db = volumeDb
    player.play()

func getAvailableSfxPlayer() -> AudioStreamPlayer:
    if availableSfxPlayers.is_empty():
        return null
    return availableSfxPlayers.pop_back()

func _onSfxFinished(player: AudioStreamPlayer) -> void:
    player.stop()
    player.stream = null
    availableSfxPlayers.append(player)

func stopSfx(sfxName: String) -> void:
    for player in sfxPool:
        if player.playing and player.stream.resource_path == sfxName:
            player.stop()
            availableSfxPlayers.append(player)
            break

func stopAllSfx() -> void:
    for player in sfxPool:
        if player.playing:
            player.stop()
            availableSfxPlayers.append(player)

func initializeMusicPlayer() -> void:
    musicPlayer = AudioStreamPlayer.new()
    musicPlayer.bus = "Music"
    add_child(musicPlayer)
    musicPlayer.finished.connect(_onMusicFinished)

func playMusic(musicResource: Resource, volumeDb: float) -> void:
    if musicPlayer.playing:
        musicPlayer.stop()
    musicPlayer.stream = musicResource
    musicPlayer.volume_db = volumeDb
    musicPlayer.play()

func stopMusic() -> void:
    if musicPlayer.playing:
        musicPlayer.stop()
        musicPlayer.stream = null

func isMusicPlaying() -> bool:
    return musicPlayer.playing

func _onMusicFinished() -> void:
    musicPlayer.play() #LOOP?
    pass

func addEffect(busName: String, effect: AudioEffect) -> void:
    var busIndex: int = AudioServer.get_bus_index(busName)
    if busIndex != -1:
        AudioServer.add_bus_effect(busIndex, effect)
    else:
        push_warning("Bus not found: " + busName)

func removeEffect(busName: String, effectType: String) -> void:
    var busIndex: int = AudioServer.get_bus_index(busName)
    if busIndex == -1:
        push_warning("Bus not found: " + busName)
        return
    var effectCount: int = AudioServer.get_bus_effect_count(busIndex)
    for index in range(effectCount):
        var currentEffect: AudioEffect = AudioServer.get_bus_effect(busIndex, index)
        if currentEffect.get_class() == effectType:
            AudioServer.remove_bus_effect(busIndex, index)
            break

func clearEffects(busName: String) -> void:
    var busIndex: int = AudioServer.get_bus_index(busName)
    if busIndex == -1:
        push_warning("Bus not found: " + busName)
        return
    while AudioServer.get_bus_effect_count(busIndex) > 0:
        AudioServer.remove_bus_effect(busIndex, 0)

func routeSoundToBus(player: AudioStreamPlayer, busName: String) -> void:
    var busIndex: int = AudioServer.get_bus_index(busName)
    if busIndex == -1:
        push_warning("Cannot route sound to non-existent bus: " + busName)
    else:
        player.bus = busName

func linearToDb(linear: float) -> float:
    if linear <= 0.0:
        return -80.0
    return 20.0 * log(linear)
