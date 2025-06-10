extends Node
#class_name ResourcePaths

const MAIN: String = "res://Main.tscn"
const TEST_HARNESS: String = "res://Scenes/TestHarness.tscn"
const MECHANICS_TEST: String = "res://Scenes/Mechanics/MechanicsTest.tscn"
const DREKKER_SCENE: String = "res://Scenes/Shaders/Color/Drekker.tscn"
const GHOST_SHAPE: String = "res://Scenes/Shaders/Shape/GhostShape.tscn"

const WATER_BODY: String = "res://Resources/TileMaps/WaterBody.tscn"
const GLACIER_MAP: String = "res://Resources/TileMaps/GlacierMap.tscn"

const METRONOME_GEN: String = "res://Scenes/Audio/MetronomeGen.tscn"
const PITCH_DIMENSION: String = "res://Scenes/Audio/PitchDimension.tscn"
const RHYTHM_DIMENSION: String = "res://Scenes/Audio/RhythmDimension.tscn"
const RHYTHM_ONSET_RECORDER: String = "res://Scenes/Audio/RhythmOnsetRecorder.tscn"

const GLACIER_GEN_SCENE: String = "res://Scenes/Entities/Glacier/GlacierGen.tscn"
const GLACIER_SIMULATION: String = "res://Scenes/Entities/Glacier/GlacierSimulation.tscn"
const CAPSULE_DUMMY: String = "res://Scenes/Entities/Characters/CapsuleDummy.tscn"
const CAPSULE_DUMMY_GEN: String = "res://Scenes/Entities/Characters/CapsuleDummyGen.tscn"
const CAPSULE_DUMMY_SCRIPT: String = "res://Scripts/Entities/Characters/CapsuleDummy.gd"

const FFT_VISUALIZER: String = "res://Scenes/Shaders/Audio/FFTVisualizer.tscn"
const IOI_VISUALIZER: String = "res://Scenes/Shaders/Audio/IOIVisualizer.tscn"
const SOUND_ENVELOPE_SCENE: String = "res://Scenes/Shaders/Audio/SoundEnvelope.tscn"
const WAVEFORM_VISUALIZER: String = "res://Scenes/Shaders/Audio/WaveformVisualizer.tscn"

const COLLISION_MASK_FRAGMENT: String = "res://Scenes/Shaders/Collision/CollisionMaskFragment.tscn"
const COLLISION_MASK_SCANLINE_POLYGONIZER: String = "res://Scenes/Shaders/Collision/CollisionMaskScanlinePolygonizer.tscn"
const RUSTY_COLLISION_MASK: String = "res://Scenes/Shaders/Collision/RustyCollisionMask.tscn"

const ICE_SHEETS_SCENE: String = "res://Scenes/Shaders/IceSheets/IceSheets.tscn"
const PERSPECTIVE_TILT_MASK_FRAGMENT: String = "res://Scenes/Shaders/MechanicAnimations/PerspectiveTiltMaskFragment.tscn"
const SNOWFALL_PARTICLES: String = "res://Scenes/Shaders/Particles/SnowfallParticles.tscn"
const SHADOWS_SCENE: String = "res://Scenes/Shaders/Shadows/Shadows.tscn"
const WATER_SCENE: String = "res://Scenes/Shaders/Water/Water.tscn"
const WATER_PROJECTED_SCENE: String = "res://Scenes/Shaders/Water/WaterProjected.tscn"

const TWOAM_MIDI: String = "res://Resources/Audio/2am.mid"
const FINGERBIB: String = "res://Resources/Audio/Fingerbib.mid"
const METRONOME_CLICK: String = "res://Resources/Audio/metronome_click.wav"
const HELLION: String = "res://Resources/Audio/Hellion.wav"
const SNUFFY: String = "res://Resources/Audio/snuffy.wav"
const SHADERTOY_MUSIC_EXPERIMENT_WAV: String = "res://Resources/Audio/shadertoy_music_experiment.wav"
const DSDNM_SF2: String = "res://Resources/Audio/dsdnm.sf2"

const CACHED_RHYTHM_DATA: String = "res://Resources/Audio/Cache/rhythm_data.tres"
const CACHED_OGG: String = "res://Resources/Audio/Cache/cached_ogg.ogg"
const CACHED_WAV: String = "res://Resources/Audio/Cache/cached_wav.wav"

const IOSEVKA_REGULAR_TTC: String = "res://Resources/Fonts/Iosevka-Regular.ttc"
const IOSEVKA_BOLD_TTC: String = "res://Resources/Fonts/Iosevka-Bold.ttc"
const JETBRAINS_MONO_REGULAR_TTF: String = "res://Resources/Fonts/JetBrainsMono-Regular.ttf"
const JETBRAINS_MONO_BOLD_TTF: String = "res://Resources/Fonts/JetBrainsMono-Bold.ttf"

const DOLPHIN2_PNG: String = "res://Assets/Sprites/Dolphin2.png"
const IKIIKIIRUKA_PNG: String = "res://Assets/Sprites/Ikiikiiruka.png"
const BONE_PATTERN_PNG: String = "res://Assets/Sprites/bone_pattern.png"
const CAPSULE_PNG: String = "res://Assets/Sprites/capsule.png"
const IRUKA_PNG: String = "res://Assets/Sprites/iruka.png"

const BAYER_PNG: String = "res://Assets/Textures/bayer.png"
const GRAY_NOISE_SMALL_PNG: String = "res://Assets/Textures/gray_noise_small.png"
const ICEBERGS_JPG: String = "res://Assets/Textures/icebergs.jpg"
const MOON_WATER_PNG: String = "res://Assets/Textures/moon_water.png"
const PEBBLES_PNG: String = "res://Assets/Textures/pebbles.png"
const ROCKS_JPG: String = "res://Assets/Textures/rocks.jpg"

const WATER_PNG: String = "res://Assets/Tiles/water.png"

const WATER_TILESET: String = "res://Resources/TileSets/water.tres"
const GLACIER_TILESET: String = "res://Resources/TileSets/glacier_tileset.tres"

const FREE_ALPHA_CHANNEL: String = "res://Resources/Shaders/free_alpha_channel.gdshader"
const INCLUDES: String = "res://Resources/Shaders/includes.gdshaderinc"
const SIMPLE_FEEDBACK_BUFFER: String = "res://Resources/Shaders/simple_feedback_buffer.gdshader"
const VIRTUAL_GRID_SNAPPING: String = "res://Resources/Shaders/virtual_grid_snapping.gdshader"

const FFT_SHADER: String = "res://Resources/Shaders/Audio/fft.gdshader"
const IOI_SHADER: String = "res://Resources/Shaders/Audio/ioi.gdshader"
const WAVEFORM_SHADER: String = "res://Resources/Shaders/Audio/waveform.gdshader"
const MUSIC_BALL: String = "res://Resources/Shaders/Audio/music_ball.gdshader"
const IMAGE_SOUND_ENVELOPE: String = "res://Resources/Shaders/Audio/SoundEnvelopeWIP/Image_sound_envelope.gdshader"
const BUFFERA_SOUND_ENVELOPE: String = "res://Resources/Shaders/Audio/SoundEnvelopeWIP/BufferA_sound_envelope.gdshader"
const OPTIMIZED_ENVELOPE_BUFFER_A: String = "res://Resources/Shaders/Audio/SoundEnvelopeWIP/optimized_envelope_buffer_a.gdshader"
const OPTIMIZED_ENVELOPE_BUFFER_B: String = "res://Resources/Shaders/Audio/SoundEnvelopeWIP/optimized_envelope_buffer_b.gdshader"
const SOUND_ENVELOPE_UTILS: String = "res://Resources/Shaders/Audio/SoundEnvelopeWIP/utils.gdshaderinc"

const SUPERSAMPLING: String = "res://Resources/Shaders/Color/supersampling.gdshaderinc"
const DREKKER_EFFECT: String = "res://Resources/Shaders/Color/drekker_effect.gdshader"

const ICE_SHEETS_SHADER: String = "res://Resources/Shaders/IceSheets/ice_sheets.gdshader"
const NOISE_INCLUDE: String = "res://Resources/Shaders/IceSheets/noise.gdshaderinc"
const PROJECTIONS_INCLUDE: String = "res://Resources/Shaders/IceSheets/projections.gdshaderinc"
const COLOR_INCLUDE: String = "res://Resources/Shaders/IceSheets/color.gdshaderinc"

const SWIM_SHADER: String = "res://Resources/Shaders/MechanicAnimations/swim.gdshader"
const JUMP_TRIG_SHADER: String = "res://Resources/Shaders/MechanicAnimations/jump_trig.gdshader"
const PERSPECTIVE_TILT_MASK_SHADER: String = "res://Resources/Shaders/MechanicAnimations/perspective_tilt_mask.gdshader"
const ALL_SPRITE_MASK_SHADER: String = "res://Resources/Shaders/MechanicAnimations/all_sprite_mask.gdshader"

const SNOW_PARTICLE_SHADER: String = "res://Resources/Shaders/Particles/snow_particle_shader.gdshader"

const SCANLINE_SHADER: String = "res://Resources/Shaders/Collision/scanline.gdshader"
const COLLISION_MASK_FRAGMENT_SHADER: String = "res://Resources/Shaders/Collision/collision_mask_fragment.gdshader"
