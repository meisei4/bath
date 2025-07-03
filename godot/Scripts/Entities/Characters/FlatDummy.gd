extends CharacterBody2D
class_name FlatDummy

enum STATE { DIVE, DIVE_ASCEND, JUMP, IDLE, SPIN }
signal state_changed(new_state: STATE)

@export var sprite: Sprite2D
@export var collision_shape: CollisionShape2D
@export var mut_ref_velocity: MutRefVelocity = MutRefVelocity.new()

@export var mechanic_scenes: Array[PackedScene] = [
    preload(ResourcePaths.STRAFE_MECHANIC),
    preload(ResourcePaths.CRUISING_MECHANIC),
    preload(ResourcePaths.JUMP_MECHANIC),
    preload(ResourcePaths.DIVE_MECHANIC),
    preload(ResourcePaths.SPIN_MECHANIC),
]

@export var animation_scenes: Array[PackedScene] = [
    preload(ResourcePaths.JUMP_ANIMATION),
    preload(ResourcePaths.DIVE_ANIMATION),
    preload(ResourcePaths.SPIN_ANIMATION),
]

@export var jump: Jump
@export var dive: Dive
@export var spin: Spin
@export var all_mechanics: Array[Node] = []

@export var jump_animation: JumpAnimation
@export var dive_animation: DiveAnimation
@export var spin_animation: SpinAnimation

var current_state: STATE = STATE.DIVE
var queued_state: STATE = STATE.IDLE


# -------------------------------------------------------------
# LIFE-CYCLE
# -------------------------------------------------------------
func _ready() -> void:
    var new_nodes_added: bool = false

    _resolve_exported_children()  # sprite / collision_shape
    new_nodes_added = _ensure_mechanics() or new_nodes_added
    new_nodes_added = _ensure_animations() or new_nodes_added

    if MaskManager.perspective_tilt_mask_fragment:
        if not MaskManager.sprite_to_mask_index.has(sprite):
            var mask_index: int = (
                MaskManager.perspective_tilt_mask_fragment.register_sprite_texture(sprite.texture)
            )
            MaskManager.sprite_to_mask_index[sprite] = mask_index

    if new_nodes_added:
        var packed_scene: PackedScene = PackedScene.new()
        packed_scene.pack(self)
        ResourceSaver.save(packed_scene, ResourcePaths.FLAT_DUMMY)

    state_changed.emit(current_state)


# -------------------------------------------------------------
#  CHILD-RESOLUTION HELPERS
# -------------------------------------------------------------
func _resolve_exported_children() -> void:
    if not collision_shape:
        for child_node: Node in get_children():
            if child_node is CollisionShape2D:
                collision_shape = child_node
                break
    if not sprite:
        for child_node: Node in get_children():
            if child_node is Sprite2D:
                sprite = child_node
                break


func _ensure_mechanics() -> bool:
    # Find already-placed mechanics
    var existing_mechanics: Array[Node] = []
    for child_node: Node in get_children():
        if child_node is Jump or child_node is Dive or child_node is Spin:
            existing_mechanics.append(child_node)

    # Wire and register existing mechanics
    for mech: Node in existing_mechanics:
        if mech.has_variable("mut_ref_velocity"):
            mech.mut_ref_velocity = mut_ref_velocity
        _wire_mechanic_signals(mech)
        if mech is Jump:
            jump = mech
        elif mech is Dive:
            dive = mech
        elif mech is Spin:
            spin = mech
    all_mechanics = existing_mechanics.duplicate()

    # Instantiate any mechanics that are still missing
    var added: bool = false
    for scene: PackedScene in mechanic_scenes:
        var prototype: Node = scene.instantiate()
        var missing: bool = true
        for mech_in_tree: Node in existing_mechanics:
            if mech_in_tree.get_class() == prototype.get_class():
                missing = false
                break
        if missing:
            var mech: Node = prototype
            mech.mut_ref_velocity = mut_ref_velocity
            add_child(mech)
            mech.owner = self
            _wire_mechanic_signals(mech)
            all_mechanics.append(mech)
            if mech is Jump:
                jump = mech
            elif mech is Dive:
                dive = mech
            elif mech is Spin:
                spin = mech
            added = true
    return added


# Helper closure to finish wiring
func _register_animation(anim: Node) -> void:
    anim.sprite = sprite
    if (
        anim is JumpAnimation
        and jump
        and not jump.animate_mechanic.is_connected(anim.process_animation_data)
    ):
        jump.animate_mechanic.connect(anim.process_animation_data)
        jump_animation = anim
    elif (
        anim is DiveAnimation
        and dive
        and not dive.animate_mechanic.is_connected(anim.process_animation_data)
    ):
        dive.animate_mechanic.connect(anim.process_animation_data)
        dive_animation = anim
    elif (
        anim is SpinAnimation
        and spin
        and not spin.animate_mechanic.is_connected(anim.process_animation_data)
    ):
        spin.animate_mechanic.connect(anim.process_animation_data)
        spin_animation = anim


func _ensure_animations() -> bool:
    # Gather animations already in the tree
    var existing_animations: Array[Node] = []
    for child_node: Node in get_children():
        if (
            child_node is JumpAnimation
            or child_node is DiveAnimation
            or child_node is SpinAnimation
        ):
            existing_animations.append(child_node)
    var added: bool = false
    # Wire existing animations
    for anim: Node in existing_animations:
        _register_animation(anim)

    # Instantiate any missing animations
    for scene: PackedScene in animation_scenes:
        var prototype: Node = scene.instantiate()
        var missing: bool = true
        for anim_in_tree: Node in existing_animations:
            if anim_in_tree.get_class() == prototype.get_class():
                missing = false
                break
        if missing:
            var anim: Node = prototype
            add_child(anim)
            anim.owner = self
            _register_animation(anim)
            added = true
    return added


func _wire_mechanic_signals(mechanic: Node) -> void:
    if mechanic.has_method("on_state_changed"):
        if not state_changed.is_connected(mechanic.on_state_changed):
            state_changed.connect(mechanic.on_state_changed)
    if mechanic.has_signal("state_completed"):
        if not mechanic.state_completed.is_connected(_on_state_completed):
            mechanic.state_completed.connect(_on_state_completed)


func _physics_process(delta: float) -> void:
    velocity = mut_ref_velocity.val
    collision_shape.disabled = _collision_shape_disabled()
    move_and_slide()
    mut_ref_velocity.val = velocity

    if Input.is_action_pressed("jump") and current_state == STATE.DIVE:
        queued_state = STATE.JUMP
        _update_state(STATE.DIVE_ASCEND)

    if Input.is_action_pressed("F") and current_state == STATE.JUMP:
        _update_state(STATE.SPIN)


# -------------------------------------------------------------
#  STATE MANAGEMENT
# -------------------------------------------------------------
func _update_state(next_state: STATE) -> void:
    if current_state != next_state:
        current_state = next_state
        state_changed.emit(next_state)


func _on_state_completed(completed_state: STATE) -> void:
    if completed_state != current_state:
        return
    if queued_state != current_state and queued_state != STATE.IDLE:
        var next_state: STATE = queued_state
        queued_state = STATE.IDLE
        _update_state(next_state)
    elif current_state == STATE.JUMP:
        _update_state(STATE.DIVE)
    elif current_state == STATE.SPIN:
        _update_state(STATE.JUMP)


func _collision_shape_disabled() -> bool:
    return (
        (not jump or jump.current_phase != Jump.JumpPhase.GROUNDED)
        or (not dive or dive.current_phase != Dive.DivePhase.LEVEL)
    )
