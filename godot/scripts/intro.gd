extends Node2D


func _ready():
	Dialogic.start("intro")
	Dialogic.signal_event.connect(_on_signal_event)
	Dialogic.timeline_ended.connect(_on_timeline_ended)

func _process(delta):
	$AnimationPlayer.play("spin")

func _on_signal_event(arg: String):
	if arg == "show_sprite":
		$Sprite.visible = true

func _on_timeline_ended():
	get_tree().change_scene_to_file("res://scenes/levels/1-entrance-hall.tscn")
