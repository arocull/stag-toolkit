// Returns a world-space Tangent Bitangent Normal matrix
// https://gamedev.stackexchange.com/questions/178606/how-to-calculate-normal-from-normal-map-in-world-space-opengl
mat3 world_normal_matrix(
	in mat3 model_normal_matrix, in vec3 model_tangent,
	in vec3 model_binormal, in vec3 model_normal
) {
	return mat3(
		normalize(model_normal_matrix * model_tangent),
		normalize(model_normal_matrix * model_binormal),
		normalize(model_normal_matrix * model_normal)
	);
}

// Applies a normal map to the given Tangent Bitangent Normal matrix
vec3 parse_normal_map(in vec3 normal_map_value, in mat3 tbn) {
	return normalize(tbn * normalize(fma(normal_map_value, vec3(2.0), vec3(-1.0))));
}

// For learning more about normal blending methods, see: https://blog.selfshadow.com/publications/blending-in-detail/
