@tool
extends StagUIFormItem

# Default value at startup.
@export var _value: bool = false:
	set(newVal):
		_value = newVal
		set_value(newVal)

@export_group("Toggle", "toggle_")
@export var toggle_on_text: String = "Enabled":
	set(newVal):
		toggle_on_text = newVal
		_tweak_toggle_text(_value)
@export var toggle_off_text: String = "Disabled":
	set(newVal):
		toggle_off_text = newVal
		_tweak_toggle_text(_value)

func set_value(value: Variant):
	%interact.button_pressed = value
	_tweak_toggle_text(value)

func _tweak_toggle_text(value: bool):
	if value:
		%interact.text = toggle_on_text
	else:
		%interact.text = toggle_off_text
