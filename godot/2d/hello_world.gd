extends Node2D

@onready var player: Player = $Player

func _on_player_speed_increased() -> void:
	print("Speed increased")


func _on_timer_timeout() -> void:
	player.increase_speed(100.0)
