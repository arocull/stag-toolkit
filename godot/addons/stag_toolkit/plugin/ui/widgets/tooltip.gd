extends Label
class_name StagUITooltip

## Root node to bind tooltip events to.
## Will recursively bind with nodes in the tree.
@export_node_path("Control") var root: NodePath = "."

## If true, automatically binds events on startup.
@export var auto_bind: bool = true

## Whether to bind tooltip display to hover events.
@export var bind_hover_events: bool = true
## Whether to bind tooltip display to focus events.
@export var bind_focus_events: bool = true

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	if auto_bind:
		bind_events(get_node_or_null(root))

func bind_events(node: Node):
	if not is_instance_valid(node):
		return

	# If this is a control node with tooltip text
	if node is Control and not node.tooltip_text.is_empty():
		if node is StagUIFormItem: # Form-specific signals for cleanliness
			if bind_focus_events:
				node.interactive_focused.connect(display_tooltip.bind(node))
			if bind_hover_events:
				node.interactive_hovered.connect(display_tooltip.bind(node))
		else:
			if bind_focus_events:
				node.focus_entered.connect(display_tooltip.bind(node))
			if bind_hover_events:
				node.mouse_entered.connect(display_tooltip.bind(node))

	for child in node.get_children(false):
		bind_events(child)

func display_tooltip(node: Control):
	text = node.tooltip_text
