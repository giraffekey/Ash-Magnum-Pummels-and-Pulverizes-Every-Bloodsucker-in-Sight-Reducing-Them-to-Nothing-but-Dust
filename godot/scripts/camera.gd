extends Camera2D


func _ready():
	position = $"../CursorLayer/Cursor".position

func _process(delta):
	position = $"../CursorLayer/Cursor".position
