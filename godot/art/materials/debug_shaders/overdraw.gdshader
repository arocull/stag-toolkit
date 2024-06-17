shader_type spatial;
render_mode blend_add, shadows_disabled, cull_back, depth_test_disabled;

uniform float alpha : hint_range(0.0, 1.0, 0.1) = 0.1;
uniform vec3 color: source_color = vec3(1,1,1);

void vertex() {
	// Called for every vertex the material is visible on.
}

void fragment() {
	ALPHA = alpha;
	ALBEDO = color;
	ROUGHNESS = 1.0;
}
