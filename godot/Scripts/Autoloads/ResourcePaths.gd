extends Node
#class_name ResourcePaths

const MAIN: String = "res://Main.tscn"
const TEST_HARNESS: String = "res://Scenes/TestHarness.tscn"
const MECHANICS_TEST: String = "res://Scenes/Mechanics/Mechanics.tscn"
const DREKKER_SCENE: String = "res://Scenes/Shaders/Color/Drekker.tscn"
const GHOST_SHAPE: String = "res://Scenes/Shaders/Shape/GhostShape.tscn"
const GLACIER_SIMULATION: String = "res://Scenes/Entities/Glacier/GlacierSimulation.tscn"

const WATER_BODY: String = "res://Resources/TileMaps/WaterBody.tscn"
const GLACIER_MAP: String = "res://Resources/TileMaps/GlacierMap.tscn"

const METRONOME_GEN: String = "res://Scenes/Audio/MetronomeGen.tscn"
const PITCH_DIMENSION: String = "res://Scenes/Audio/PitchDimension.tscn"
const RHYTHM_DIMENSION: String = "res://Scenes/Audio/RhythmDimension.tscn"
const RHYTHM_ONSET_RECORDER: String = "res://Scenes/Audio/RhythmOnsetRecorder.tscn"

const WATER_SCENE: String = "res://Scenes/Shaders/Water/Water.tscn"
const WATER_PROJECTED_SCENE: String = "res://Scenes/Shaders/Water/WaterProjected.tscn"
const GLACIER_GEN_SCENE: String = "res://Scenes/Entities/Glacier/GlacierGen.tscn"
const CAPSULE_DUMMY_GEN: String = "res://Scenes/Entities/Characters/CapsuleDummyGen.tscn"
const CAPSULE_DUMMY_SCRIPT: String = "res://Scripts/Entities/Characters/CapsuleDummy.gd"
const CAPSULE_DUMMY: String = "res://Scenes/Entities/Characters/CapsuleDummy.tscn"

const LATERAL_MOVEMENT_MECHANIC: String = "res://Scenes/Mechanics/LateralMovement.tscn"
const JUMP_MECHANIC: String = "res://Scenes/Mechanics/Jump.tscn"
const SWIM_MECHANIC: String = "res://Scenes/Mechanics/Swim.tscn"

const WAVEFORM_VISUALIZER: String = "res://Scenes/Shaders/Audio/WaveformVisualizer.tscn"
const FFT_VISUALIZER: String = "res://Scenes/Shaders/Audio/FFTVisualizer.tscn"
const SOUND_ENVELOPE_SCENE: String = "res://Scenes/Shaders/Audio/SoundEnvelope.tscn"
const IOI_VISUALIZER: String = "res://Scenes/Shaders/Audio/IOIVisualizer.tscn"

const COLLISION_MASK_FRAGMENT: String = "res://Scenes/Shaders/Collision/CollisionMaskFragment.tscn"
const COLLISION_MASK_SCANLINE_POLYGONIZER: String = "res://Scenes/Shaders/Collision/CollisionMaskScanlinePolygonizer.tscn"
const RUSTY_COLLISION_MASK: String = "res://Scenes/Shaders/Collision/CollisionMaskIncrementalScanlinePolygonizer.tscn"

const ICE_SHEETS_SCENE: String = "res://Scenes/Shaders/IceSheets/IceSheets.tscn"
const PERSPECTIVE_TILT_MASK_FRAGMENT: String = "res://Scenes/Shaders/MechanicsAnimations/PerspectiveTiltMaskFragment.tscn"
const SNOWFALL_PARTICLES: String = "res://Scenes/Shaders/Particles/SnowfallParticles.tscn"
const SHADOWS_SCENE: String = "res://Scenes/Shaders/Shadows/Shadows.tscn"

const HELLION: String = "res://Resources/audio/hellion.wav"
const SNUFFY: String = "res://Resources/audio/snuffy.wav"
const METRONOME_CLICK: String = "res://Resources/audio/metronome_click.wav"

# https://shadertoyunofficial.wordpress.com/2019/07/23/shadertoy-media-files
# https://www.shadertoy.com/media/a/29de534ed5e4a6a224d2dfffab240f2e19a9d95f5e39de8898e850efdb2a99de.mp3
const SHADERTOY_MUSIC_EXPERIMENT_WAV: String = "res://Resources/audio/shadertoy_music_experiment.wav"
#ffmpeg -i shadertoy_music_experiment.wav -c:a libvorbis -qscale:a 0.1 -ar 12000 -ac 1 -compression_level 10 shadertoy_music_experiment_min_bitrate.ogg
const SHADERTOY_MUSIC_EXPERIMENT_OGG: String = "res://Resources/audio/shadertoy_music_experiment_min_bitrate.ogg"
const FINGERBIB: String = "res://Resources/audio/fingerbib.mid"
const DSDNMOY_SF2: String = "res://Resources/audio/dsdnmoy.sf2"

const CACHED_RHYTHM_DATA: String = "res://Resources/audio/Cache/RhythmData.tres"
const CACHED_OGG: String = "res://Resources/audio/Cache/cached_ogg.ogg"
const CACHED_WAV: String = "res://Resources/audio/Cache/cached_wav.wav"

# https://shadertoyunofficial.wordpress.com/2019/07/23/shadertoy-media-files
const BAYER_PNG: String = "res://Resources/textures/bayer.png"
const GRAY_NOISE_SMALL_PNG: String = "res://Resources/textures/gray_noise_small.png"
const PEBBLES_PNG: String = "res://Resources/textures/pebbles.png"
const ROCKS_JPG: String = "res://Resources/textures/rocks.jpg"

const IOSEVKA_REGULAR_TTC: String = "res://Resources/fonts/Iosevka-Regular.ttc"
const IOSEVKA_BOLD_TTC: String = "res://Resources/fonts/Iosevka-Bold.ttc"

const DOLPHIN2_PNG: String = "res://Resources/sprites/Dolphin2.png"
const IKIIKIIRUKA_PNG: String = "res://Resources/sprites/Ikiikiiruka.png"
const BONE_PATTERN_PNG: String = "res://Resources/sprites/bone_pattern.png"
const CAPSULE_PNG: String = "res://Resources/sprites/capsule.png"
const IRUKA_PNG: String = "res://Resources/sprites/iruka.png"

const ICEBERGS_JPG: String = "res://Resources/textures/icebergs.jpg"
const MOON_WATER_PNG: String = "res://Resources/textures/moon_water.png"
const WATER_PNG: String = "res://Resources/tiles/water.png"

const WATER_TILESET: String = "res://Resources/TileSets/Water.tres"
const GLACIER_TILESET: String = "res://Resources/TileSets/GlacierTileset.tres"

const FFT_SHADER: String = "res://Resources/shaders/audio/fft.gdshader"
const IOI_SHADER: String = "res://Resources/shaders/audio/ioi.gdshader"
const WAVEFORM_SHADER: String = "res://Resources/shaders/audio/waveform.gdshader"
const MUSIC_BALL: String = "res://Resources/shaders/audio/music_ball.gdshader"
const IMAGE_SOUND_ENVELOPE: String = "res://Resources/shaders/audio/sound_envelope_wip/image_sound_envelope.gdshader"
const BUFFERA_SOUND_ENVELOPE: String = "res://Resources/shaders/audio/sound_envelope_wip/buffer_a_sound_envelope.gdshader"
const OPTIMIZED_ENVELOPE_BUFFER_A: String = "res://Resources/shaders/audio/sound_envelope_wip/optimized_envelope_buffer_a.gdshader"
const OPTIMIZED_ENVELOPE_BUFFER_B: String = "res://Resources/shaders/audio/sound_envelope_wip/optimized_envelope_buffer_b.gdshader"
const SOUND_ENVELOPE_UTILS: String = "res://Resources/shaders/audio/sound_envelope_wip/utils.gdshaderinc"
const FINITE_APPROX_RIPPLE: String = "res://Resources/shaders/water/finite_approx_ripple.gdshader"
const WATER_SHADER: String = "res://Resources/shaders/water/water.gdshader"
const WATER_PROJECTED_SHADER: String = "res://Resources/shaders/water/water_projected.gdshader"

const SUPERSAMPLING: String = "res://Resources/shaders/color/supersampling.gdshaderinc"
const DREKKER_EFFECT: String = "res://Resources/shaders/color/drekker_effect.gdshader"

const ICE_SHEETS_SHADER: String = "res://Resources/shaders/ice_sheets/ice_sheets.gdshader"
const NOISE_INCLUDE: String = "res://Resources/shaders/ice_sheets/noise.gdshaderinc"
const PROJECTIONS_INCLUDE: String = "res://Resources/shaders/ice_sheets/projections.gdshaderinc"
const COLOR_INCLUDE: String = "res://Resources/shaders/ice_sheets/color.gdshaderinc"

const SWIM_SHADER: String = "res://Resources/shaders/mechanics_animations/swim.gdshader"
const JUMP_TRIG_SHADER: String = "res://Resources/shaders/mechanics_animations/jump_trig.gdshader"
const PERSPECTIVE_TILT_MASK_SHADER: String = "res://Resources/shaders/mechanics_animations/perspective_tilt_mask.gdshader"
const ALL_SPRITE_MASK_SHADER: String = "res://Resources/shaders/mechanics_animations/all_sprite_mask.gdshader"

const SNOW_PARTICLE_SHADER: String = "res://Resources/shaders/particles/snow_particle_shader.gdshader"
const UMBRAL_SHADER: String = "res://Resources/shaders/shadows/umbral_zone.gdshader"
const DITHER_SHADER: String = "res://Resources/shaders/shadows/dither_zone.gdshader"

const SCANLINE_SHADER: String = "res://Resources/shaders/collision/scanline.gdshader"
const FREE_ALPHA_CHANNEL: String = "res://Resources/shaders/collision/free_alpha_channel.gdshader"
const COLLISION_MASK_FRAGMENT_SHADER: String = "res://Resources/shaders/collision/collision_mask_fragment.gdshader"
