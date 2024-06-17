vec2 rotate2D(in vec2 vector, in float theta) {
	float c = cos(theta);
	float s = sin(theta);
	return vec2(
		c * vector.x - s * vector.y,
		s * vector.x + c * vector.y
	);
}
