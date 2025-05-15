extends Node
class_name AudioPool

var pool_size: int
var bus: AudioBus.BUS
var loop_on_end: bool = false

var players: Array[AudioStreamPlayer] = []
var available: Array[AudioStreamPlayer] = []


func _ready():
    for i in pool_size:
        var p = AudioStreamPlayer.new()
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


func play(resource: AudioStream, volume_db: float = 0.0):
    var p = acquire()
    if p:
        p.stream = resource
        p.volume_db = volume_db
        p.play()


func _on_finished(p: AudioStreamPlayer):
    if loop_on_end and p.stream:
        p.play()
    else:
        p.stop()
        p.stream = null
        available.append(p)
