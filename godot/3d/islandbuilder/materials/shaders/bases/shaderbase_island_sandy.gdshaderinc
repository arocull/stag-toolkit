#include "res://addons/stag_toolkit/utils/shader_includes/include_remap.gdshaderinc"
render_mode cull_back;
//#define QUALITY_HIGH

// Noise texture and corresponding normal map
uniform sampler2D noise_generic: hint_default_white, filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2D noise_normal: hint_normal, filter_linear_mipmap_anisotropic, repeat_enable;
// Color Gradients
uniform sampler2D gradient_rock: source_color, filter_linear;
uniform sampler2D gradient_dirt: source_color, filter_linear;
uniform sampler2D gradient_sand: source_color, filter_linear;

const vec2 uv_scale_rock = vec2(0.001,0.02);
const vec2 uv_scale_dirt = vec2(0.07,0.07);
const vec2 uv_scale_sand = vec2(0.5,0.5);
const float uv_scale_sandvar = 0.005;
const vec2 roughness_rock = vec2(0.8,0.92); // Linear remap for roughness
const vec2 roughness_dirt = vec2(0.7,0.8); // Linear remap
const vec2 roughness_sand = vec2(0.5,0.8); // Linear remap
const float sand_variance = 0.45;
const float specular = 0.2; // Base specular value

// Varying values automatically interpolate between vertex and fragment passes
varying vec2 uv_rock;
varying vec2 uv_dirt_flat;
varying vec2 uv_sand;
#ifdef QUALITY_HIGH
varying vec2 uv_dirt_cylindrical;
#endif

void vertex() {
	// Apply UV scalings and store for fragment pass
	uv_rock = UV * uv_scale_rock;
	uv_dirt_flat = UV2 * uv_scale_dirt;
	uv_sand = UV2 * uv_scale_sand;
	#ifdef QUALITY_HIGH
	uv_dirt_cylindrical = UV * uv_scale_dirt;
	#endif
}

void fragment() {
	// Sample noise values for each material using given UVs
	vec4 rng1 = texture(noise_generic, uv_rock).rgba;
	vec4 rng2 = texture(noise_generic, uv_sand).rgba;

	float rng_rock = rng1.r;
	float rng_dirt = mix(rng2.r, rng2.g, COLOR.g);

	#ifdef QUALITY_HIGH
	vec4 rng3 = texture(noise_generic, uv_sand * uv_scale_sandvar);
	vec2 rng_sand = vec2(rng2.b, mix(rng2.b, rng3.b, sand_variance));
	#else
	vec2 rng_sand = vec2(rng2.b, mix(rng2.b, rng2.a + rng2.r, (sand_variance - COLOR.a) * sand_variance));
	#endif

	// Determine coloration for each material based on gradient
	vec3 col_rock = texture(gradient_rock, vec2(rng_rock, 0)).rgb;
	vec3 col_dirt = texture(gradient_dirt, vec2(rng_dirt, 0)).rgb;
	vec3 col_sand = texture(gradient_sand, vec2(rng_sand.g, 0)).rgb;
	// Determine roughness for each material based on gradient
	float r_rock = remap_fast(rng_rock,roughness_rock.x,roughness_rock.y);
	float r_dirt = remap_fast(rng_dirt,roughness_dirt.x,roughness_dirt.y);
	float r_sand = remap_fast(rng_sand.x,roughness_sand.x,roughness_sand.y);

	// Use pre-baked tri-planar UVs for rock and sand, and blend both UVs for dirt mapping
	vec3 norm_rock = texture(noise_normal, uv_rock).rgb;
	vec3 norm_sand = texture(noise_normal, uv_sand).rgb;
	#ifdef QUALITY_HIGH
	vec3 norm_dirt = mix(mix(
		texture(noise_normal, uv_dirt_cylindrical).rgb,
		texture(noise_normal, uv_dirt_flat).rgb,
		COLOR.g), vec3(0.5,0.5,1.0), 0.25);
	#else
	vec3 norm_dirt = mix(texture(noise_normal, uv_dirt_flat).rgb, vec3(0.5,0.5,1.0), 0.25);
	#endif

	// Called for every pixel the material is visible on.
	ALBEDO = mix(mix(col_rock, col_dirt, COLOR.g), col_sand, COLOR.b);
	ROUGHNESS = mix(mix(r_rock, r_dirt, COLOR.g), r_sand, COLOR.b);
	NORMAL_MAP = mix(mix(norm_rock, norm_dirt, COLOR.g), norm_sand, COLOR.b);
	SPECULAR = specular;
	//#ifdef HIGH_QUALITY
	AO = COLOR.r; // Ambient Occlusion is pre-baked to red channel
	//#endif
}
