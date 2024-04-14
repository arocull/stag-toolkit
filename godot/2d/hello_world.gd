extends Node2D

@onready var player: Player = $Player
@onready var island_builder: IslandBuilder = $IslandBuilder

func _on_player_speed_increased() -> void:
	print("Speed increased")
	island_builder.generate()


func _on_timer_timeout() -> void:
	player.increase_speed(100.0)
