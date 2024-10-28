#ifdef OVERDRAW
render_mode blend_add, shadows_disabled, cull_back, depth_test_disabled, fog_disabled;
uniform float alpha : hint_range(0.0, 1.0, 0.01) = 0.15;
uniform vec3 color: source_color = vec3(1,1,1);
#else
render_mode fog_disabled, unshaded, shadows_disabled;
#endif
#ifdef DEBUG_WIREFRAME
render_mode wireframe, blend_mul;
#endif

void vertex() {}

void fragment() {
	#ifdef OVERDRAW
	ALPHA = alpha;
	ALBEDO = color;
	ROUGHNESS = 1.0;
	#endif
	#ifdef DEBUG_COLOR
	ALBEDO = COLOR.rgb;
	#endif
	#ifdef DEBUG_COLORALPHA
	ALBEDO = vec3(COLOR.a);
	#endif
	#ifdef DEBUG_UV
	ALBEDO = vec3(fract(UV), 0);
	#endif
	#ifdef DEBUG_UV2
	ALBEDO = vec3(fract(UV2), 0);
	#endif
	#ifdef DEBUG_WIREFRAME
	ALBEDO = vec3(0.0);
	#endif
}
