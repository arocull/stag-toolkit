shader_type spatial;

#include "res://addons/stag_toolkit/utils/shader_includes/include_remap.gdshaderinc"
//#define DEBUG

#ifdef DEBUG
render_mode unshaded, cull_back, blend_mix;
#else
render_mode blend_mix, cull_back;
#endif

uniform sampler2D noise_generic: hint_default_white, filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2D noise_normal: hint_normal, filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2D gradient_rock: source_color, filter_linear;
uniform sampler2D gradient_dirt: source_color, filter_linear;
uniform sampler2D gradient_sand: source_color, filter_linear;

uniform vec2 uv_scale_rock = vec2(0.001,0.015);
uniform vec2 uv_scale_dirt = vec2(0.07,0.07);
uniform vec2 uv_scale_sand = vec2(0.5,0.5);
uniform float uv_scale_sandvar = 0.005;
uniform vec2 roughness_rock = vec2(0.8,0.92);
uniform vec2 roughness_dirt = vec2(0.7,0.8);
uniform vec2 roughness_sand = vec2(0.5,0.8);
uniform float sand_variance: hint_range(0.0, 1.0, 0.01) = 0.9;

const float specular = 0.2;

varying float mix_dirt;
varying float mix_sand;
varying vec2 uv_rock;
varying vec2 uv_dirt_flat;
varying vec2 uv_dirt_cylindrical;
varying vec2 uv_sand;

void vertex() {
	// Called for every vertex the material is visible on.
	float facing_up = dot(NORMAL * MODELVIEW_NORMAL_MATRIX, vec3(0,1,0) * MODELVIEW_NORMAL_MATRIX);

	mix_dirt = clamp(remap(facing_up, -0.1, 0.8, 0, 1), 0, 1);
	mix_sand = pow(clamp(remap(facing_up, 0.7, 1, 0, 1), 0, 1), 3);

	uv_rock = vec2(VERTEX.x + VERTEX.z, VERTEX.y) * uv_scale_rock;
	uv_dirt_flat = vec2(VERTEX.x, VERTEX.z) * uv_scale_dirt;
	uv_dirt_cylindrical = vec2(VERTEX.x + VERTEX.z, VERTEX.y) * uv_scale_dirt;
	uv_sand = vec2(VERTEX.x, VERTEX.z) * uv_scale_sand;
}

void fragment() {
	float rng_rock = texture(noise_generic, uv_rock).r;
	float rng_dirt = mix(
		texture(noise_generic, uv_dirt_cylindrical).r,
		texture(noise_generic, uv_dirt_flat).r,
		mix_dirt);
	float rng_sand = texture(noise_generic, uv_sand).r;
	float rng_sand2 = mix(rng_sand, texture(noise_generic, uv_sand * uv_scale_sandvar).r, sand_variance);

	vec3 col_rock = texture(gradient_rock, vec2(rng_rock, 0)).rgb;
	vec3 col_dirt = texture(gradient_dirt, vec2(rng_dirt, 0)).rgb;
	vec3 col_sand = texture(gradient_sand, vec2(rng_sand2, 0)).rgb;

	float r_rock = remap_fast(rng_rock,roughness_rock.x,roughness_rock.y);
	float r_dirt = remap_fast(rng_dirt,roughness_dirt.x,roughness_dirt.y);
	float r_sand = remap_fast(rng_sand,roughness_sand.x,roughness_sand.y);

	vec3 norm_rock = texture(noise_normal, uv_rock).rgb;
	vec3 norm_dirt = mix(mix(
		texture(noise_normal, uv_dirt_cylindrical).rgb,
		texture(noise_normal, uv_dirt_flat).rgb,
		mix_dirt), vec3(0.5,0.5,0.5), 0.5);
	vec3 norm_sand = texture(noise_normal, uv_sand).rgb;

	#ifdef DEBUG
	ALBEDO = vec3(rng_dirt); //vec3(mix(mix(rng_rock, rng_dirt, mix_dirt), rng_sand, mix_sand));
	#else
	// Called for every pixel the material is visible on.
	ALBEDO = mix(mix(col_rock, col_dirt, mix_dirt), col_sand, mix_sand);
	ROUGHNESS = mix(mix(r_rock, r_dirt, mix_dirt), r_sand, mix_sand);
	NORMAL_MAP = mix(mix(norm_rock, norm_dirt, mix_dirt), norm_sand, mix_sand);
	SPECULAR = specular;
	AO = COLOR.r;
	#endif
}
