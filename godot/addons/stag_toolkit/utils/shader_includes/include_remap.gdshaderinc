// Remaps a value from a given range to a new range
float remap(in float value, in float in_min, in float in_max, in float out_min, in float out_max) {
	return out_min + (value - in_min) * ((out_max - out_min) / (in_max - in_min));
}
// Remaps a value from 0 to 1 to a new range
float remap_fast(in float value, in float minimum, in float maximum) {
	return minimum + value * (maximum - minimum);
}
