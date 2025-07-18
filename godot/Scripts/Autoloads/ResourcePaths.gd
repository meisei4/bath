extends Node
#class_name ResourcePaths

const MAIN: String = "res://Main.tscn"
const TEST_HARNESS: String = "res://Scenes/TestHarness.tscn"
const MECHANICS_TEST: String = "res://Scenes/Mechanics/Mechanics.tscn"
const DREKKER_SCENE: String = "res://Scenes/Shaders/Color/Drekker.tscn"
const GHOST_SHAPE: String = "res://Scenes/Shaders/Audio/GhostShape.tscn"
const GLACIER_SIMULATION: String = "res://Scenes/Entities/Glacier/GlacierSimulation.tscn"

const WATER_BODY: String = "res://assets/TileMaps/WaterBody.tscn"
const GLACIER_MAP: String = "res://assets/TileMaps/GlacierMap.tscn"

const PITCH_DIMENSION: String = "res://Scenes/Audio/PitchDimension.tscn"
const RHYTHM_DIMENSION: String = "res://Scenes/Audio/RhythmDimension.tscn"
const RHYTHM_ONSET_RECORDER: String = "res://Scenes/Audio/RhythmOnsetRecorder.tscn"

const WATER_SCENE: String = "res://Scenes/Shaders/Water/Water.tscn"
const WATER_PROJECTED_SCENE: String = "res://Scenes/Shaders/Water/WaterProjected.tscn"
const GLACIER_GEN_SCENE: String = "res://Scenes/Entities/Glacier/GlacierGen.tscn"
const CAPSULE_DUMMY_GEN: String = "res://Scenes/Entities/Characters/CapsuleDummyGen.tscn"
const CAPSULE_DUMMY_SCRIPT: String = "res://Scripts/Entities/Characters/CapsuleDummy.gd"
const FLAT_DUMMY_SCRIPT: String = "res://Scripts/Entities/Characters/FlatDummy.gd"
const CAPSULE_DUMMY: String = "res://Scenes/Entities/Characters/CapsuleDummy.tscn"
const FLAT_DUMMY: String = "res://Scenes/Entities/Characters/FlatDummy.tscn"

const ANIMATION_CONTROLLER: String = "res://Scenes/Mechanics/AnimationController.tscn"
const MECHANIC_CONTROLLER: String = "res://Scenes/Mechanics/MechanicController.tscn"

const STRAFE_MECHANIC: String = "res://Scenes/Mechanics/Strafe.tscn"
const CRUISING_MECHANIC: String = "res://Scenes/Mechanics/Cruising.tscn"
const JUMP_MECHANIC: String = "res://Scenes/Mechanics/Jump.tscn"
const DIVE_MECHANIC: String = "res://Scenes/Mechanics/Dive.tscn"
const SPIN_MECHANIC: String = "res://Scenes/Mechanics/Spin.tscn"

const JUMP_ANIMATION: String = "res://Scenes/Mechanics/JumpAnimation.tscn"
const DIVE_ANIMATION: String = "res://Scenes/Mechanics/DiveAnimation.tscn"
const SPIN_ANIMATION: String = "res://Scenes/Mechanics/SpinAnimation.tscn"

const WAVEFORM_VISUALIZER: String = "res://Scenes/Shaders/Audio/WaveformVisualizer.tscn"
const FFT_VISUALIZER: String = "res://Scenes/Shaders/Audio/FFTVisualizer.tscn"
const SOUND_ENVELOPE_SCENE: String = "res://Scenes/Shaders/Audio/SoundEnvelope.tscn"
const IOI_VISUALIZER: String = "res://Scenes/Shaders/Audio/IOIVisualizer.tscn"

const COLLISION_MASK_FRAGMENT: String = "res://Scenes/Shaders/Masks/CollisionMaskFragment.tscn"
const COLLISION_MASK_SCANLINE_POLYGONIZER: String = "res://Scenes/Shaders/Masks/CollisionMaskScanlinePolygonizer.tscn"
const RUSTY_COLLISION_MASK: String = "res://Scenes/Shaders/Masks/CollisionMaskIncrementalScanlinePolygonizer.tscn"
const PERSPECTIVE_TILT_MASK_FRAGMENT: String = "res://Scenes/Shaders/Masks/PerspectiveTiltMaskFragment.tscn"
const SHADOW_MASK_SCENE: String = "res://Scenes/Shaders/Masks/ShadowMask.tscn"

const ICE_SHEETS_SCENE: String = "res://Scenes/Shaders/IceSheets/IceSheetsRenderer.tscn"
const SNOWFALL_PARTICLES: String = "res://Scenes/Shaders/Particles/SnowfallParticles.tscn"

const HELLION: String = "res://assets/audio/hellion.wav"
const SNUFFY: String = "res://assets/audio/snuffy.wav"

# https://shadertoyunofficial.wordpress.com/2019/07/23/shadertoy-media-files
# https://www.shadertoy.com/media/a/29de534ed5e4a6a224d2dfffab240f2e19a9d95f5e39de8898e850efdb2a99de.mp3
const SHADERTOY_MUSIC_EXPERIMENT_WAV: String = "res://assets/audio/shadertoy_music_experiment.wav"
#ffmpeg -i shadertoy_music_experiment.wav -c:a libvorbis -qscale:a 0.1 -ar 12000 -ac 1 -compression_level 10 shadertoy_music_experiment_min_bitrate.ogg
const SHADERTOY_MUSIC_EXPERIMENT_OGG: String = "res://assets/audio/shadertoy_music_experiment_min_bitrate.ogg"
const FINGERBIB: String = "res://assets/audio/fingerbib.mid"
const DSDNMOY_SF2: String = "res://assets/audio/dsdnmoy.sf2"

const CACHED_RHYTHM_DATA: String = "res://assets/audio/Cache/RhythmData.tres"
const CACHED_OGG: String = "res://assets/audio/Cache/cached_ogg.ogg"
const CACHED_WAV: String = "res://assets/audio/Cache/cached_wav.wav"

# https://shadertoyunofficial.wordpress.com/2019/07/23/shadertoy-media-files
const BAYER_PNG: String = "res://assets/textures/bayer.png"
const GRAY_NOISE_SMALL_PNG: String = "res://assets/textures/gray_noise_small.png"
const PEBBLES_PNG: String = "res://assets/textures/pebbles.png"
const ROCKS_JPG: String = "res://assets/textures/rocks.jpg"

const IOSEVKA_REGULAR_TTC: String = "res://assets/fonts/Iosevka-Regular.ttc"
const IOSEVKA_BOLD_TTC: String = "res://assets/fonts/Iosevka-Bold.ttc"

const DOLPHIN2_PNG: String = "res://assets/sprites/Dolphin2.png"
const IKIIKIIRUKA_PNG: String = "res://assets/sprites/Ikiikiiruka.png"
const BONE_PATTERN_PNG: String = "res://assets/sprites/bone_pattern.png"
const CAPSULE_PNG: String = "res://assets/sprites/capsule.png"
const IRUKA_PNG: String = "res://assets/sprites/iruka.png"

const ICEBERGS_JPG: String = "res://assets/textures/icebergs.jpg"
const MOON_WATER_PNG: String = "res://assets/textures/moon_water.png"
const WATER_PNG: String = "res://assets/tiles/water.png"

const WATER_TILESET: String = "res://assets/TileSets/Water.tres"
const GLACIER_TILESET: String = "res://assets/TileSets/GlacierTileset.tres"

const FFT_SHADER: String = "res://assets/shaders/gdshader/audio/fft.gdshader"
const IOI_SHADER: String = "res://assets/shaders/gdshader/audio/ioi.gdshader"
const WAVEFORM_SHADER: String = "res://assets/shaders/gdshader/audio/waveform.gdshader"
const MUSIC_BALL: String = "res://assets/shaders/gdshader/audio/music_ball.gdshader"
const GHOST: String = "res://assets/shaders/gdshader/audio/ghost.gdshader"
const IMAGE_SOUND_ENVELOPE: String = "res://assets/shaders/gdshader/audio/sound_envelope_wip/image_sound_envelope.gdshader"
const BUFFERA_SOUND_ENVELOPE: String = "res://assets/shaders/gdshader/audio/sound_envelope_wip/buffer_a_sound_envelope.gdshader"
const OPTIMIZED_ENVELOPE_BUFFER_A: String = "res://assets/shaders/gdshader/audio/sound_envelope_wip/optimized_envelope_buffer_a.gdshader"
const OPTIMIZED_ENVELOPE_BUFFER_B: String = "res://assets/shaders/gdshader/audio/sound_envelope_wip/optimized_envelope_buffer_b.gdshader"
const SOUND_ENVELOPE_UTILS: String = "res://assets/shaders/gdshader/audio/sound_envelope_wip/utils.gdshaderinc"
const FINITE_APPROX_RIPPLE: String = "res://assets/shaders/gdshader/water/finite_approx_ripple.gdshader"
const WATER_SHADER: String = "res://assets/shaders/gdshader/water/water.gdshader"
const WATER_PROJECTED_SHADER: String = "res://assets/shaders/gdshader/water/water_projected.gdshader"

const SUPERSAMPLING: String = "res://assets/shaders/gdshader/color/supersampling.gdshaderinc"
const DREKKER_EFFECT: String = "res://assets/shaders/gdshader/color/drekker_effect.gdshader"

const ICE_SHEETS_SHADER_FULL: String = "res://assets/shaders/gdshader/ice_sheets/icesheet_full.gdshader"
const ICE_SHEETS_SHADER: String = "res://assets/shaders/gdshader/ice_sheets/ice_sheets.gdshader"
const NOISE_INCLUDE: String = "res://assets/shaders/gdshader/ice_sheets/noise.gdshaderinc"
const PROJECTIONS_INCLUDE: String = "res://assets/shaders/gdshader/ice_sheets/projections.gdshaderinc"
const COLOR_INCLUDE: String = "res://assets/shaders/gdshader/ice_sheets/color.gdshaderinc"

const DIVE_SHADER: String = "res://assets/shaders/gdshader/mechanics/dive.gdshader"
const JUMP_TRIG_SHADER: String = "res://assets/shaders/gdshader/mechanics/jump_trig.gdshader"
const PERSPECTIVE_TILT_MASK_SHADER: String = "res://assets/shaders/gdshader/masks/perspective_tilt_mask.gdshader"
const ALL_SPRITE_MASK_SHADER: String = "res://assets/shaders/gdshader/masks/all_sprite_mask.gdshader"

const SNOW_PARTICLE_SHADER: String = "res://assets/shaders/gdshader/particles/snow_particle_shader.gdshader"
const UMBRAL_SHADER: String = "res://assets/shaders/gdshader/masks/umbral_zone.gdshader"
const DITHER_SHADER: String = "res://assets/shaders/gdshader/masks/dither_zone.gdshader"

const SCANLINE_SHADER: String = "res://assets/shaders/gdshader/masks/scanline.gdshader"
const FREE_ALPHA_CHANNEL: String = "res://assets/shaders/gdshader/masks/free_alpha_channel.gdshader"
const COLLISION_MASK_FRAGMENT_SHADER: String = "res://assets/shaders/gdshader/masks/collision_mask_fragment.gdshader"
