@tool
extends Control
class_name StagUIFormItem
## Base class for general-purpose nodes for stuff like settings and other things.

## Emitted when the interactive node is hovered while enabled.
signal interactive_hovered()
## Emitted when the interactive node is no longer hovered, or unhobered while enabled.
signal interactive_unhovered()
## Emitted when the interactive node is focused.
signal interactive_focused()
## Emitted when the interactive part of the form is unfocused.
signal interactive_unfocused()
@warning_ignore_start("unused_signal")
## Emitted when the interactive node is pressed while enabled.
signal interactive_pressed()
## Emitted when the interactive node is pressed while disabled.
signal interactive_pressed_disabled()
## Emitted when the interactive node value changed.
signal value_changed(value)
@warning_ignore_restore("unused_signal")

## Text to display alongside the interactive node.
## Set to an empty string to hide the label.
@export var label: String = "Label":
	set(newVal):
		label = newVal
		_tweak_label()
## Whether this form item can be modified or interacted with.
@export var disabled: bool = false:
	set(newVal):
		disabled = newVal
		_tweak_disabled_state()

@export_group("Label", "label_")
## Define how text is laid horizontally.
@export var label_horizontal_alignment: HorizontalAlignment = HorizontalAlignment.HORIZONTAL_ALIGNMENT_LEFT:
	set(newVal):
		label_horizontal_alignment = newVal
		%label.horizontal_alignment = newVal
## Define how text is laid vertically.
@export var label_vertical_alignment: VerticalAlignment = VerticalAlignment.VERTICAL_ALIGNMENT_CENTER:
	set(newVal):
		label_vertical_alignment = newVal
		%label.vertical_alignment = newVal

@export_group("Icon", "icon_")
## Optional icon texture to include alongside the interactive node.
## Clear the asset to hide the icon.
@export var icon_texture: Texture2D = null:
	set(newVal):
		icon_texture = newVal
		_tweak_icon()
## Define how icon size is handled.
@export var icon_expand_mode: TextureRect.ExpandMode = TextureRect.ExpandMode.EXPAND_KEEP_SIZE:
	set(newVal):
		icon_expand_mode = newVal
		_tweak_icon()

# Interactable node of the UI (i.e. a toggle button, dropdown, text input, slider, etc)
var _interact: Control
var _is_hovered: bool = false
var _is_valid: bool = true
var _validation_hint: String = ""

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	_interact = get_node_or_null("%interact") # Try to find a unique node first
	if not is_instance_valid(_interact):
		# Non-unique path (allows users to customize directly in scene)
		_interact.get_node_or_null("interact")

	# Bind hover events to interact node
	if is_instance_valid(_interact):
		_interact.focus_entered.connect(interactive_focused.emit)
		_interact.focus_exited.connect(interactive_unfocused.emit)
		_interact.mouse_entered.connect(_hovered)
		_interact.mouse_exited.connect(_unhovered)

## Generic read function for all form items.
func get_value() -> Variant:
	return null
## Generic set function for all form items.
## Note that this bypasses sanitization and validation steps.
## See also [member StagUIFormItem.set_value_sanitized].
func set_value(value: Variant):
	pass
## Sanitizes the value and marking it as valid, before calling [member StagUIFormItem.set_value].
func set_value_sanitized(value: Variant):
	var v = _sanitize(value)
	_is_valid = _validate(v)
	set_value(v)
## Sanitizes the value before passing it along.
## Override this as needed.
func _sanitize(value: Variant) -> Variant:
	return value
## Returns whether the passed value is an acceptable input.
## Optionally set _validation_hint as needed.
## Override this as needed.
func _validate(value: Variant) -> bool:
	return true
## Returns whether the current input is valid.
func valid() -> bool:
	return _is_valid

func _tweak_label():
	%label.text = label
	%label.visible = not label.is_empty()
func _tweak_icon():
	%icon.texture = icon_texture
	%icon.expand_mode = icon_expand_mode
	%icon.visible = is_instance_valid(icon_texture)
func _tweak_disabled_state():
	if _is_hovered:
		if disabled: # Cursor was hovering item, and we've now stopped hovering over it
			interactive_unhovered.emit()
		else: # Cursor was hovering item, and we've now regained focus of it
			interactive_hovered.emit()
	if "disabled" in _interact:
		_interact.disabled = disabled

func _get_configuration_warnings() -> PackedStringArray:
	var warnings: PackedStringArray = PackedStringArray()
	if not is_instance_valid(_interact):
		warnings.append("Needs Control child with name '%interact'")
	return warnings

func _hovered():
	_is_hovered = true
	if not disabled:
		interactive_hovered.emit()

func _unhovered():
	_is_hovered = false
	if not disabled: # We already played the unhover event upon disabling
		interactive_unhovered.emit()
