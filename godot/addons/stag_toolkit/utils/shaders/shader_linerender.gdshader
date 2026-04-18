shader_type spatial;

//render_mode unshaded;

#include "res://addons/stag_toolkit/utils/shader_includes/include_linerender.gdshaderinc"

uniform float line_radius = 0.05;

void vertex() {
	//VERTEX = VERTEX + cross((MODELVIEW_MATRIX * vec4(NORMAL, 0.0)).xyz, CAMERA_DIRECTION_VIEW) * line_radius;
	VERTEX.x += float(VERTEX_ID);
	VERTEX.y += float(VERTEX_ID * 4);
}

void fragment() {
	// Called for every pixel the material is visible on.
	ALBEDO = COLOR.rgb;
	ALPHA = COLOR.a;
}

//void light() {
//	// Called for every pixel for every light affecting the material.
//	// Uncomment to replace the default light processing function with this one.
//}
