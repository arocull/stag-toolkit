render_mode fog_disabled, unshaded, shadows_disabled;

void fragment() {
	#ifdef DEBUG_COLOR
	ALBEDO = COLOR.rgb;
	#endif
	#ifdef DEBUG_UV
	ALBEDO = vec3(fract(UV), 0);
	#endif
	#ifdef DEBUG_UV2
	ALBEDO = vec3(fract(UV2), 0);
	#endif
}
