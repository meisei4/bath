extends Node
#class_name ResourcePaths

# --- SCENES ---
const MAIN: String = "res://Main.tscn"
const WATER_BODY: String = "res://Resources/TileMaps/WaterBody.tscn"
const GLACIER_MAP: String = "res://Resources/TileMaps/GlacierMap.tscn"

# TestScenes
const SPRITE_CONTOUR_POLYGON_GEN: String = "res://TestScenes/SkeletonTests/SpriteContourPolygonGen.tscn"
const SKELETON_TEST: String = "res://TestScenes/SkeletonTests/SkeletonTest.tscn"
const TEST_HARNESS: String = "res://TestScenes/TestHarness.tscn"
const TEST_OLD: String = "res://TestScenes/TestOld.tscn"
const MECHANICS_TEST: String = "res://TestScenes/Mechanics/MechanicsTest.tscn"

# Audio Test Scenes
const RHYTHM_DIMENSION: String = "res://TestScenes/Audio/RhythmDimension.tscn"
const AUDIO_ZONE: String = "res://TestScenes/Audio/AudioZoning/AudioZone.tscn"
const AUDIO_ZONE_GEN: String = "res://TestScenes/Audio/AudioZoning/AudioZoneGen.tscn"
const RHYTHM_ONSET_RECORDER: String = "res://TestScenes/Audio/RhythmOnsetRecorder.tscn"
const PITCH_DIMENSION: String = "res://TestScenes/Audio/PitchDimension.tscn"
const METRONOME_GEN: String = "res://TestScenes/Audio/MetronomeGen.tscn"
const MUSIC_TEST: String = "res://TestScenes/Audio/MusicTest.tscn"

# Shader Test Scenes
const GHOST_SHAPE: String = "res://TestScenes/Shaders/Shape/GhostShape.tscn"
const SHADOWS_SCENE: String = "res://TestScenes/Shaders/Shadows/Shadows.tscn"
const WATER_SCENE: String = "res://TestScenes/Shaders/Water/Water.tscn"
const WATER_PROJECTED_SCENE: String = "res://TestScenes/Shaders/Water/WaterProjected.tscn"
const DREKKER_SCENE: String = "res://TestScenes/Shaders/Color/Drekker.tscn"
const SNOWFALL_PARTICLES: String = "res://TestScenes/Shaders/Particles/SnowfallParticles.tscn"
const WAVEFORM_VISUALIZER: String = "res://TestScenes/Shaders/Audio/WaveformVisualizer.tscn"
const IOI_VISUALIZER: String = "res://TestScenes/Shaders/Audio/IOIVisualizer.tscn"
const SOUND_ENVELOPE_SCENE: String = "res://TestScenes/Shaders/Audio/SoundEnvelope.tscn"
const FFT_VISUALIZER: String = "res://TestScenes/Shaders/Audio/FFTVisualizer.tscn"
const ICE_SHEETS_SCENE: String = "res://TestScenes/Shaders/IceSheets/IceSheets.tscn"
const PERSPECTIVE_TILT_MASK_FRAGMENT: String = "res://TestScenes/Shaders/MechanicAnimations/PerspectiveTiltMaskFragment.tscn"
const COLLISION_MASK_SCANLINE_POLYGONIZER: String = "res://TestScenes/Shaders/Collision/CollisionMaskScanlinePolygonizer.tscn"
const RUSTY_COLLISION_MASK: String = "res://TestScenes/Shaders/Collision/RustyCollisionMask.tscn"
const COLLISION_MASK_FRAGMENT: String = "res://TestScenes/Shaders/Collision/CollisionMaskFragment.tscn"

# Entity Test Scenes
const GLACIER_SIMULATION: String = "res://TestScenes/Entities/Glacier/GlacierSimulation.tscn"
const GLACIER_GEN_SCENE: String = "res://TestScenes/Entities/Glacier/GlacierGen.tscn"
const CAPSULE_DUMMY: String = "res://TestScenes/Entities/Characters/CapsuleDummy.tscn"
const CAPSULE_DUMMY_GEN: String = "res://TestScenes/Entities/Characters/CapsuleDummyGen.tscn"


# --- AUDIO ---
const FINGERBIB: String = "res://Resources/Audio/Fingerbib.mid"
const TWOAM_MIDI: String = "res://Resources/Audio/2am.mid"

const METRONOME_CLICK: String = "res://Resources/Audio/metronome_click.wav"
const SHADERTOY_MUSIC_EXPERIMENT_WAV: String = "res://Resources/Audio/shadertoy_music_experiment.wav"
const HELLION: String = "res://Resources/Audio/Hellion.wav"
const SNUFFY: String = "res://Resources/Audio/snuffy.wav"
const SNUFFY_SYNTH_38_48_WAV: String = "res://Resources/Audio/snuffy_synth_38_48.wav"
const SNUFFY_ISOLATED_WAV: String = "res://Resources/Audio/snuffy__isolated.wav"
const SNUFFY_SYNTH_ISOLATED_WAV: String = "res://Resources/Audio/snuffy_synth_isolated.wav"
const SNUFFY_SYNTH_ISOLATED_GATED_WAV: String = "res://Resources/Audio/snuffy_synth_isolated_gated.wav"
const DSDNM_SF2: String = "res://Resources/Audio/dsdnm.sf2"

# Cached Audio
const CACHED_OGG: String = "res://Resources/Audio/Cache/cached_ogg.ogg"
const CACHED_WAV: String = "res://Resources/Audio/Cache/cached_wav.wav"
const CACHED_RHYTHM_DATA: String = "res://Resources/Audio/Cache/rhythm_data.tres"


# --- FONTS ---
const IOSEVKA_REGULAR_TTC: String = "res://Resources/Fonts/Iosevka-Regular.ttc"
const IOSEVKA_BOLD_TTC: String = "res://Resources/Fonts/Iosevka-Bold.ttc"
const JETBRAINS_MONO_REGULAR_TTF: String = "res://Resources/Fonts/JetBrainsMono-Regular.ttf"
const JETBRAINS_MONO_BOLD_TTF: String = "res://Resources/Fonts/JetBrainsMono-Bold.ttf"


# --- CHARACTER COMPONENTS ---
const CHARACTER_COMPONENTS_DEFAULT: String = "res://Resources/CharacterComponents/default.tres"


# --- TILESETS ---
const WATER_TILESET: String = "res://Resources/TileSets/water.tres"
const GLACIER_TILESET: String = "res://Resources/TileSets/glacier_tileset.tres"


# --- SHADERS ---
const GHOST: String = "res://Resources/Shaders/Shape/ghost.gdshader"
const FREE_ALPHA_CHANNEL: String = "res://Resources/Shaders/free_alpha_channel.gdshader"
const VIRTUAL_GRID_SNAPPING: String = "res://Resources/Shaders/virtual_grid_snapping.gdshader"
const SIMPLE_FEEDBACK_BUFFER: String = "res://Resources/Shaders/simple_feedback_buffer.gdshader"

# Color shaders
const SUPERSAMPLING: String = "res://Resources/Shaders/Color/supersampling.gdshaderinc"
const DREKKER_EFFECT: String = "res://Resources/Shaders/Color/drekker_effect.gdshader"

# Particle shaders
const SNOW_PARTICLE_SHADER: String = "res://Resources/Shaders/Particles/snow_particle_shader.gdshader"

# Water shaders
const WATER_SHADER: String = "res://Resources/Shaders/Water/water.gdshader"
const WATER_PROJECTED_SHADER: String = "res://Resources/Shaders/Water/water_projected.gdshader"
const FINITE_APPROX_RIPPLE: String = "res://Resources/Shaders/Water/finite_approx_ripple.gdshader"
const CONSTANTS_INCLUDE: String = "res://Resources/Shaders/Water/constants.gdshaderinc"

# Audio shaders
const FFT_SHADER: String = "res://Resources/Shaders/Audio/fft.gdshader"
const IOI_SHADER: String = "res://Resources/Shaders/Audio/ioi.gdshader"
const WAVEFORM_SHADER: String = "res://Resources/Shaders/Audio/waveform.gdshader"
const IMAGE_SOUND_ENVELOPE: String = "res://Resources/Shaders/Audio/SoundEnvelopeWIP/Image_sound_envelope.gdshader"
const BUFFERA_SOUND_ENVELOPE: String = "res://Resources/Shaders/Audio/SoundEnvelopeWIP/BufferA_sound_envelope.gdshader"
const OPTIMIZED_ENVELOPE_BUFFER_A: String = "res://Resources/Shaders/Audio/SoundEnvelopeWIP/optimized_envelope_buffer_a.gdshader"
const OPTIMIZED_ENVELOPE_BUFFER_B: String = "res://Resources/Shaders/Audio/SoundEnvelopeWIP/optimized_envelope_buffer_b.gdshader"
const SOUND_ENVELOPE_UTILS: String = "res://Resources/Shaders/Audio/SoundEnvelopeWIP/utils.gdshaderinc"
const MUSIC_BALL: String = "res://Resources/Shaders/Audio/music_ball.gdshader"

# IceSheets shaders
const ICE_SHEETS_SHADER: String = "res://Resources/Shaders/IceSheets/ice_sheets.gdshader"
const NOISE_INCLUDE: String = "res://Resources/Shaders/IceSheets/noise.gdshaderinc"
const PROJECTIONS_INCLUDE: String = "res://Resources/Shaders/IceSheets/projections.gdshaderinc"
const COLOR_INCLUDE: String = "res://Resources/Shaders/IceSheets/color.gdshaderinc"

# Mechanic animation shaders
const SWIM_SHADER: String = "res://Resources/Shaders/MechanicAnimations/swim.gdshader"
const JUMP_TRIG_SHADER: String = "res://Resources/Shaders/MechanicAnimations/jump_trig.gdshader"
const PERSPECTIVE_TILT_MASK_SHADER: String = "res://Resources/Shaders/MechanicAnimations/perspective_tilt_mask.gdshader"
const ALL_SPRITE_MASK_SHADER: String = "res://Resources/Shaders/MechanicAnimations/all_sprite_mask.gdshader"

# Collision shaders
const SCANLINE_SHADER: String = "res://Resources/Shaders/Collision/scanline.gdshader"
const COLLISION_MASK_FRAGMENT_SHADER: String = "res://Resources/Shaders/Collision/collision_mask_fragment.gdshader"
