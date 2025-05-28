extends Node
class_name AudioPool

var pool_size: int
var bus: AudioBus.BUS
var loop_on_end: bool = false

var players: Array[AudioStreamPlayer] = []
var available: Array[AudioStreamPlayer] = []


func _ready() -> void:
    for i: int in pool_size:
        var p: AudioStreamPlayer = AudioStreamPlayer.new()
        p.bus = AudioBus.val(bus)
        p.finished.connect(_on_finished.bind(p))
        add_child(p)
        players.append(p)
        available.append(p)


func acquire() -> AudioStreamPlayer:
    if available.is_empty():
        push_warning("Pool exhausted on bus %s" % bus)
        return null
    return available.pop_back()


func play(resource: AudioStream, volume_db: float = 0.0) -> void:
    var p: AudioStreamPlayer = acquire()
    p.playback_type = AudioServer.PLAYBACK_TYPE_SAMPLE  #TODO this fixes web export playback
    if p:
        p.stream = resource
        p.volume_db = volume_db
        p.play()


func _on_finished(p: AudioStreamPlayer) -> void:
    if loop_on_end and p.stream:
        p.play()
    else:
        p.stop()
        p.stream = null
        available.append(p)
