extends Node2D


func _on_animation_started(anim_name):
	visible = true

func _on_animation_finished(anim_name):
	visible = false
