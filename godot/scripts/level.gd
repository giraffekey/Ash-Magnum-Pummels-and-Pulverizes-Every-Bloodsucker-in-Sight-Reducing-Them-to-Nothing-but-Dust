extends Node2D


var last_cursor_position


func _ready():
	pass

func _process(delta):
	if $Cursor.visible:
		if Input.is_action_just_pressed("left"):
			$Cursor.position.x -= 16
		if Input.is_action_just_pressed("right"):
			$Cursor.position.x += 16
		if Input.is_action_just_pressed("up"):
			$Cursor.position.y -= 16
		if Input.is_action_just_pressed("down"):
			$Cursor.position.y += 16
	else:
		if Input.is_action_just_pressed("left") \
		or Input.is_action_just_pressed("right") \
		or Input.is_action_just_pressed("up") \
		or Input.is_action_just_pressed("down"):
			$Cursor.visible = true
			if last_cursor_position == null:
				last_cursor_position = $Player.position + Vector2(0, 4)
			$Cursor.position = last_cursor_position
