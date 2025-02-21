// Remaps a value from a given range to a new range
float remap(in float value, in float in_min, in float in_max, in float out_min, in float out_max) {
	return fma(value - in_min, (out_max - out_min) / (in_max - in_min), out_min);
}
// Remaps a value from 0 to 1 to a new range
float remap_fast(in float value, in float out_minimum, in float out_maximum) {
	return fma(value, out_maximum - out_minimum, out_minimum);
}
