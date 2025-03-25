shader_type spatial;
render_mode skip_vertex_transform, specular_disabled, cull_disabled, unshaded;

// TODO: rewrite this shader to be better

// #define DEBUG

const int NUM_POINTS_MAX = 400;
uniform int point_count = 100;
const int NUM_RADIALS = 4;
uniform vec3 points[NUM_POINTS_MAX];
uniform float radius = 0.05;
uniform float rope_length = 20.0;
uniform sampler2D basecolor: source_color, repeat_enable;
uniform sampler2D arm: hint_default_white, repeat_enable;

varying vec2 uv;
#ifdef DEBUG
varying flat int idx;
#endif

vec3 viewspace(in mat4 modelview, in vec3 pt) {
	return (modelview * vec4(pt, 1.0)).xyz;
}
// https://en.wikipedia.org/wiki/Rodrigues%27_rotation_formula
vec3 rotate(in vec3 v, in vec3 axis, in float theta) {
	return v * cos(theta) + cross(v, axis) * sin(theta) + axis * dot(axis, v) * (1.0 - cos(theta));
}

void vertex() {
	int sampleIdx = int(floor(UV.y * 1.9999 * float(point_count)));

	vec3 plane = normalize(points[point_count - 1] - points[0]);

	vec3 pos = points[sampleIdx];
	vec3 axis = normalize(mix(points[sampleIdx+1], plane, float(sampleIdx == 0 || sampleIdx == point_count - 1)) - pos);
	vec3 radial = cross(axis, vec3(0,1,0));
	radial = rotate(radial, axis, TAU * UV.x);

	VERTEX = viewspace(MODELVIEW_MATRIX, pos + radial * radius);
	NORMAL = normalize((MODELVIEW_MATRIX * vec4(radial, 0.0)).xyz);
	uv = vec2(UV.x, UV.y * (rope_length / radius) * 0.5);
	#ifdef DEBUG
	idx = sampleIdx;
	#endif
}
void fragment() {
	#ifdef DEBUG
	float prop = float(idx) / float(point_count);
	ALBEDO = vec3(prop, prop, prop);
	ROUGHNESS = 1.0;
	#else
	vec2 ar = texture(arm, uv).rg;
	ALBEDO = texture(basecolor, uv).rgb;
	AO = ar.r;
	// AO_LIGHT_AFFECT = 0.5;
	ROUGHNESS = ar.g;
	// SPECULAR = 0.0;
	#endif
}
