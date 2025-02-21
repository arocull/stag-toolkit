// Returns the linear depth to the given coordinate.
float get_linear_depth(in float depth, in vec2 screenUV, in mat4 invProjectionMatrix) {
	// ndc = vec3(screenUV * 2.0 - 1.0, depth)
	vec4 view = invProjectionMatrix * vec4(fma(screenUV, vec2(2.0), vec2(-1.0)), depth, 1.0);
	view.xyz /= view.w;
	return -view.z;
}

// In-line normalized screen UV coordinate.
#define SCREEN_UV_NORMALIZED (SCREEN_UV - vec2(0.5,0.5)) * VIEWPORT_SIZE / max(VIEWPORT_SIZE.x, VIEWPORT_SIZE.y)
