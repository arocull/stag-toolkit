render_mode shadows_disabled, unshaded, cull_disabled, depth_prepass_alpha;

#include "res://addons/stag_toolkit/utils/shader_includes/include_colorspace.gdshaderinc"

uniform float discard_threshold : hint_range(0.0, 1.0, 0.001) = 0.5;

void vertex() {

}

void fragment() {
	ALBEDO = linear_to_rgb(COLOR.rgb);

	#ifdef MASK
	float mask = COLOR.a * clamp(1.0 - 2.0 * length(UV - vec2(0.5)), 0.0, 1.0);
	#else
	float mask = COLOR.a;
	#endif

	#ifdef DISCARD
	if (mask < discard_threshold) {
		discard;
	}
	#else
	ALPHA = mask;
	#endif
}
