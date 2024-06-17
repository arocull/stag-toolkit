shader_type spatial;
render_mode fog_disabled, unshaded, shadows_disabled, wireframe, blend_mul;

const vec3 col = vec3(0,0,0);

void fragment() {
	ALBEDO = col;
}
