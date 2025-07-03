pub struct ResourcePaths;

impl ResourcePaths {
    pub const MAIN: &'static str = "res://Main.tscn";
    pub const TEST_HARNESS: &'static str = "res://Scenes/TestHarness.tscn";
    pub const MECHANICS_TEST: &'static str = "res://Scenes/Mechanics/Mechanics.tscn";
    pub const DREKKER_SCENE: &'static str = "res://Scenes/Shaders/Color/Drekker.tscn";
    pub const GHOST_SHAPE: &'static str = "res://Scenes/Shaders/Audio/GhostShape.tscn";
    pub const GLACIER_SIMULATION: &'static str = "res://Scenes/Entities/Glacier/GlacierSimulation.tscn";

    pub const WATER_BODY: &'static str = "res://Resources/TileMaps/WaterBody.tscn";
    pub const GLACIER_MAP: &'static str = "res://Resources/TileMaps/GlacierMap.tscn";

    pub const METRONOME_GEN: &'static str = "res://Scenes/Audio/MetronomeGen.tscn";
    pub const PITCH_DIMENSION: &'static str = "res://Scenes/Audio/PitchDimension.tscn";
    pub const RHYTHM_DIMENSION: &'static str = "res://Scenes/Audio/RhythmDimension.tscn";
    pub const RHYTHM_ONSET_RECORDER: &'static str = "res://Scenes/Audio/RhythmOnsetRecorder.tscn";

    pub const WATER_SCENE: &'static str = "res://Scenes/Shaders/Water/Water.tscn";
    pub const WATER_PROJECTED_SCENE: &'static str = "res://Scenes/Shaders/Water/WaterProjected.tscn";
    pub const GLACIER_GEN_SCENE: &'static str = "res://Scenes/Entities/Glacier/GlacierGen.tscn";
    pub const CAPSULE_DUMMY_GEN: &'static str = "res://Scenes/Entities/Characters/CapsuleDummyGen.tscn";
    pub const CAPSULE_DUMMY_SCRIPT: &'static str = "res://Scripts/Entities/Characters/CapsuleDummy.gd";
    pub const CAPSULE_DUMMY: &'static str = "res://Scenes/Entities/Characters/CapsuleDummy.tscn";

    pub const STRAFE_MECHANIC: &'static str = "res://Scenes/Mechanics/Strafe.tscn";
    pub const JUMP_MECHANIC: &'static str = "res://Scenes/Mechanics/Jump.tscn";
    pub const DIVE_MECHANIC: &'static str = "res://Scenes/Mechanics/Dive.tscn";
    pub const SPIN_MECHANIC: &'static str = "res://Scenes/Mechanics/Spin.tscn";

    pub const JUMP_ANIMATION: &'static str = "res://Scenes/Mechanics/JumpAnimation.tscn";
    pub const DIVE_ANIMATION: &'static str = "res://Scenes/Mechanics/DiveAnimation.tscn";
    pub const SPIN_ANIMATION: &'static str = "res://Scenes/Mechanics/SpinAnimation.tscn";

    pub const WAVEFORM_VISUALIZER: &'static str = "res://Scenes/Shaders/Audio/WaveformVisualizer.tscn";
    pub const FFT_VISUALIZER: &'static str = "res://Scenes/Shaders/Audio/FFTVisualizer.tscn";
    pub const SOUND_ENVELOPE_SCENE: &'static str = "res://Scenes/Shaders/Audio/SoundEnvelope.tscn";
    pub const IOI_VISUALIZER: &'static str = "res://Scenes/Shaders/Audio/IOIVisualizer.tscn";

    pub const COLLISION_MASK_FRAGMENT: &'static str = "res://Scenes/Shaders/Masks/CollisionMaskFragment.tscn";
    pub const COLLISION_MASK_SCANLINE_POLYGONIZER: &'static str =
        "res://Scenes/Shaders/Masks/CollisionMaskScanlinePolygonizer.tscn";
    pub const RUSTY_COLLISION_MASK: &'static str =
        "res://Scenes/Shaders/Masks/CollisionMaskIncrementalScanlinePolygonizer.tscn";
    pub const PERSPECTIVE_TILT_MASK_FRAGMENT: &'static str =
        "res://Scenes/Shaders/Masks/PerspectiveTiltMaskFragment.tscn";
    pub const SHADOW_MASK_SCENE: &'static str = "res://Scenes/Shaders/Masks/ShadowMask.tscn";

    pub const ICE_SHEETS_SCENE: &'static str = "res://Scenes/Shaders/IceSheets/IceSheets.tscn";
    pub const SNOWFALL_PARTICLES: &'static str = "res://Scenes/Shaders/Particles/SnowfallParticles.tscn";

    pub const HELLION: &'static str = "res://Resources/audio/hellion.wav";
    pub const SNUFFY: &'static str = "res://Resources/audio/snuffy.wav";
    pub const METRONOME_CLICK: &'static str = "res://Resources/audio/metronome_click.wav";

    pub const SHADERTOY_MUSIC_EXPERIMENT_WAV: &'static str = "res://Resources/audio/shadertoy_music_experiment.wav";
    pub const SHADERTOY_MUSIC_EXPERIMENT_OGG: &'static str =
        "res://Resources/audio/shadertoy_music_experiment_min_bitrate.ogg";
    pub const FINGERBIB: &'static str = "res://Resources/audio/fingerbib.mid";
    pub const DSDNMOY_SF2: &'static str = "res://Resources/audio/dsdnmoy.sf2";

    pub const CACHED_RHYTHM_DATA: &'static str = "res://Resources/audio/Cache/RhythmData.tres";
    pub const CACHED_OGG: &'static str = "res://Resources/audio/Cache/cached_ogg.ogg";
    pub const CACHED_WAV: &'static str = "res://Resources/audio/Cache/cached_wav.wav";

    pub const BAYER_PNG: &'static str = "res://Resources/textures/bayer.png";
    pub const GRAY_NOISE_SMALL_PNG: &'static str = "res://Resources/textures/gray_noise_small.png";
    pub const PEBBLES_PNG: &'static str = "res://Resources/textures/pebbles.png";
    pub const ROCKS_JPG: &'static str = "res://Resources/textures/rocks.jpg";

    pub const IOSEVKA_REGULAR_TTC: &'static str = "res://Resources/fonts/Iosevka-Regular.ttc";
    pub const IOSEVKA_BOLD_TTC: &'static str = "res://Resources/fonts/Iosevka-Bold.ttc";

    pub const DOLPHIN2_PNG: &'static str = "res://Resources/sprites/Dolphin2.png";
    pub const IKIIKIIRUKA_PNG: &'static str = "res://Resources/sprites/Ikiikiiruka.png";
    pub const BONE_PATTERN_PNG: &'static str = "res://Resources/sprites/bone_pattern.png";
    pub const CAPSULE_PNG: &'static str = "res://Resources/sprites/capsule.png";
    pub const IRUKA_PNG: &'static str = "res://Resources/sprites/iruka.png";

    pub const ICEBERGS_JPG: &'static str = "res://Resources/textures/icebergs.jpg";
    pub const MOON_WATER_PNG: &'static str = "res://Resources/textures/moon_water.png";
    pub const WATER_PNG: &'static str = "res://Resources/tiles/water.png";

    pub const WATER_TILESET: &'static str = "res://Resources/TileSets/Water.tres";
    pub const GLACIER_TILESET: &'static str = "res://Resources/TileSets/GlacierTileset.tres";

    pub const FFT_SHADER: &'static str = "res://Resources/shaders/audio/fft.gdshader";
    pub const IOI_SHADER: &'static str = "res://Resources/shaders/audio/ioi.gdshader";
    pub const WAVEFORM_SHADER: &'static str = "res://Resources/shaders/audio/waveform.gdshader";
    pub const MUSIC_BALL: &'static str = "res://Resources/shaders/audio/music_ball.gdshader";
    pub const IMAGE_SOUND_ENVELOPE: &'static str =
        "res://Resources/shaders/audio/sound_envelope_wip/image_sound_envelope.gdshader";
    pub const BUFFERA_SOUND_ENVELOPE: &'static str =
        "res://Resources/shaders/audio/sound_envelope_wip/buffer_a_sound_envelope.gdshader";
    pub const OPTIMIZED_ENVELOPE_BUFFER_A: &'static str =
        "res://Resources/shaders/audio/sound_envelope_wip/optimized_envelope_buffer_a.gdshader";
    pub const OPTIMIZED_ENVELOPE_BUFFER_B: &'static str =
        "res://Resources/shaders/audio/sound_envelope_wip/optimized_envelope_buffer_b.gdshader";
    pub const SOUND_ENVELOPE_UTILS: &'static str = "res://Resources/shaders/audio/sound_envelope_wip/utils.gdshaderinc";
    pub const FINITE_APPROX_RIPPLE: &'static str = "res://Resources/shaders/water/finite_approx_ripple.gdshader";
    pub const WATER_SHADER: &'static str = "res://Resources/shaders/water/water.gdshader";
    pub const WATER_PROJECTED_SHADER: &'static str = "res://Resources/shaders/water/water_projected.gdshader";

    pub const SUPERSAMPLING: &'static str = "res://Resources/shaders/color/supersampling.gdshaderinc";
    pub const DREKKER_EFFECT: &'static str = "res://Resources/shaders/color/drekker_effect.gdshader";

    pub const ICE_SHEETS_SHADER: &'static str = "res://Resources/shaders/ice_sheets/ice_sheets.gdshader";
    pub const NOISE_INCLUDE: &'static str = "res://Resources/shaders/ice_sheets/noise.gdshaderinc";
    pub const PROJECTIONS_INCLUDE: &'static str = "res://Resources/shaders/ice_sheets/projections.gdshaderinc";
    pub const COLOR_INCLUDE: &'static str = "res://Resources/shaders/ice_sheets/color.gdshaderinc";
    pub const ICE_SHEETS_SHADER_FULL: &'static str = "res://Resources/shaders/ice_sheets/icesheet_full.gdshader";

    pub const DIVE_SHADER: &'static str = "res://Resources/shaders/mechanics/dive.gdshader";
    pub const JUMP_TRIG_SHADER: &'static str = "res://Resources/shaders/mechanics/jump_trig.gdshader";
    pub const PERSPECTIVE_TILT_MASK_SHADER: &'static str =
        "res://Resources/shaders/masks/perspective_tilt_mask.gdshader";
    pub const ALL_SPRITE_MASK_SHADER: &'static str = "res://Resources/shaders/masks/all_sprite_mask.gdshader";

    pub const SNOW_PARTICLE_SHADER: &'static str = "res://Resources/shaders/particles/snow_particle_shader.gdshader";
    pub const UMBRAL_SHADER: &'static str = "res://Resources/shaders/masks/umbral_zone.gdshader";
    pub const DITHER_SHADER: &'static str = "res://Resources/shaders/masks/dither_zone.gdshader";

    pub const SCANLINE_SHADER: &'static str = "res://Resources/shaders/masks/scanline.gdshader";
    pub const FREE_ALPHA_CHANNEL: &'static str = "res://Resources/shaders/masks/free_alpha_channel.gdshader";
    pub const COLLISION_MASK_FRAGMENT_SHADER: &'static str =
        "res://Resources/shaders/masks/collision_mask_fragment.gdshader";
}
