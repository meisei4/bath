//TODO: THESE ARE MAYBE FUN TO FUCK WITH WITH AUDIO INPUT
#define NORMAL_SAMPLE_OFFSET 1e-4
#define NORMAL_Z_SCALE           1.0  // Multiplier for the Z component in the normal
#define AIR_REFRACTION_INDEX     1.0
#define WATER_REFRACTION_INDEX   1.08
#define REFRACTION_INDEX_RATIO (1.0 / 1.333) // from simple version
//#define REFRACTION_INDEX_RATIO   (AIR_REFRACTION_INDEX/ WATER_REFRACTION_INDEX) // For water ~0.75. Try (1.0 / 1.5)=~0.67 (glass), (1.0 / 2.4)=~0.42 (diamond), etc.
#define INCIDENT_DIRECTION       vec3(0.0, 0.0, -1.0) // "Camera is looking down" direction

//CAUSTIC CONSTANTS:
#define VIRTUAL_DS_RES_X 256.0 // Virtual display resolution width for DS resolution; recommended range: 128.0–1024.0.
#define VIRTUAL_DS_RES_Y 192.0// Virtual display resolution height for DS resolution; recommended range: 128.0–1024.0.

#define ZERO_POSITIONAL_OFFSET          vec2(0.0, 0.0)  // No offset
#define NOISE_DISP_MAP_1_INITIAL_OFFSET vec2(-0.1, 0.0) // Initial offset for noise map 1; small values ([-1.0, 1.0]) yield subtle movement.
#define NOISE_DISP_MAP_2_INITIAL_OFFSET vec2( 0.1, 0.0) // Initial offset for noise map 2; small values ([-1.0, 1.0]) yield subtle movement.

#define NOISE_DISP_MAP_1_SCROLL_VELOCITY vec2( 0.02,  0.02) // Scrolling velocity for noise map 1; recommended per component: [-0.1, 0.1] for natural motion.
#define NOISE_DISP_MAP_2_SCROLL_VELOCITY vec2(-0.02, -0.02) // Scrolling velocity for noise map 2; recommended per component: [-0.1, 0.1] for natural motion.

#define CAUSTICS_DISP_MAP_1_SCROLL_VELOCITY vec2(-0.1,  0.01) // Scrolling velocity for caustics map 1; horizontal: [-0.2, 0.2], vertical: [-0.1, 0.1].
#define CAUSTICS_DISP_MAP_2_SCROLL_VELOCITY vec2( 0.1,  0.01) // Scrolling velocity for caustics map 2; horizontal: [-0.2, 0.2], vertical: [-0.1, 0.1].

#define NOISE_DISP_MAP_DIMMING_FACTOR    0.33 // dims the noise texture -> maximum brightness becomes 1/3 where raw = 1.0 (would be white)
#define CAUSTICS_DISP_MAP_DIMMING_FACTOR 0.22 // dims the caustics texture -> maximum brightness becomes 2/9 where raw = 1.0 (would be white)
#define BACKGROUND_DISP_WARP_FACTOR      0.05 // simple warp factor for applying height and displacement map to background texture

#define BLURRY_ALPHA 0.4 // 40% opacity -> grey overtone (trough areas surrounding the inclining undulations)
#define NORMAL_ALPHA 0.0 // effectively the dark inclines in a wave undulation
#define FULL_ALPHA   4.0 // blast the fuck out of all 4 displacement maps 400% opacity -> white (catching the light at the undulation peaks)

#define NOISE_DISP_INDUCED_INTENSITY_THRESHOLD   0.30 // when noise displacement maps effect > 0.3, show water effect at normal alpha
#define ALL_DISP_MAP_INDUCED_INTENSITY_THRESHOLD 0.75 // when all_brightness > 0.75, show white full alpha
