// In-line Linear to RGB conversion
#define LINEAR_TO_RGB(r, g, b) vec3(pow(##r, 2.2), pow(##g, 2.2), pow(##b, 2.2))

// More accurate, perceptual color conversion.
// https://github.com/godotengine/godot/issues/70927#issuecomment-1371528580
vec3 linear_to_rgb(in vec3 col) {
	return mix(
		pow(
			(col.rgb + vec3(0.055)) * (1.0 / 1.055),
			vec3(2.4)
		),
		col.rgb * (1.0 / 12.92),
		lessThan(col.rgb, vec3(0.04045))
	);
}
