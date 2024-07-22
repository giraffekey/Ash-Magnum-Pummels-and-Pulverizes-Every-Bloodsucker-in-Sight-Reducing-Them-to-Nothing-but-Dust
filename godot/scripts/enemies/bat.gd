extends CharacterBody2D


enum Dir { UP, DOWN, LEFT, RIGHT }

const INVINCIBILITY = 0.5
const SPEED = 32
const SIGHT = 96

var health = 2
var is_hit = false
var invincible = false
var dir
var chasing = false
var chase_dir
var player
var invincibility_timer
var change_direction_timer


func _ready():
	invincibility_timer = Timer.new()
	add_child(invincibility_timer)
	invincibility_timer.timeout.connect(func(): invincible = false)

	change_direction_timer = Timer.new()
	add_child(change_direction_timer)
	change_direction_timer.timeout.connect(change_direction)
	change_direction()

	player = $"../Player"

func _process(_delta):
	chasing = can_see_player()

	if chasing:
		var x_dist = abs(position.x - player.position.x)
		var y_dist = abs(position.y - player.position.y)
		if player.position.x < position.x and x_dist < y_dist:
			chase_dir = Dir.LEFT
		elif player.position.x > position.x and x_dist < y_dist:
			chase_dir = Dir.RIGHT
		elif player.position.y < position.y and y_dist <= x_dist:
			chase_dir = Dir.UP
		elif player.position.y > position.y and y_dist <= x_dist:
			chase_dir = Dir.DOWN

func _physics_process(delta):
	var speed = SPEED
	if chasing:
		speed = SPEED * 2

	match dir:
		Dir.UP:
			velocity.y -= speed * 4 * delta
			if velocity.y < -speed:
				velocity.y = -speed

			if velocity.x > 0:
				velocity.x -= speed * 4 * delta
				if velocity.x < 0:
					velocity.x = 0
			elif velocity.x < 0:
				velocity.x += speed * 4 * delta
				if velocity.x > 0:
					velocity.x = 0

			$Attack/CollisionShape.position = Vector2(0, -4)
			$Sprite.flip_h = false
			$AnimationPlayer.play("back_walk")
		Dir.DOWN:
			velocity.y += speed * 4 * delta
			if velocity.y > speed:
				velocity.y = speed

			if velocity.x > 0:
				velocity.x -= speed * 4 * delta
				if velocity.x < 0:
					velocity.x = 0
			elif velocity.x < 0:
				velocity.x += speed * 4 * delta
				if velocity.x > 0:
					velocity.x = 0

			$Attack/CollisionShape.position = Vector2(0, 4)
			$Sprite.flip_h = false
			$AnimationPlayer.play("front_walk")
		Dir.LEFT:
			velocity.x -= speed * 4 * delta
			if velocity.x < -speed:
				velocity.x = -speed

			if velocity.y > 0:
				velocity.y -= speed * 4 * delta
				if velocity.y < 0:
					velocity.y = 0
			elif velocity.y < 0:
				velocity.y += speed * 4 * delta
				if velocity.y > 0:
					velocity.y = 0

			$Attack/CollisionShape.position = Vector2(-4, 0)
			$Sprite.flip_h = true
			$AnimationPlayer.play("side_walk")
		Dir.RIGHT:
			velocity.x += speed * 4 * delta
			if velocity.x > speed:
				velocity.x = speed

			if velocity.y > 0:
				velocity.y -= speed * 4 * delta
				if velocity.y < 0:
					velocity.y = 0
			elif velocity.y < 0:
				velocity.y += speed * 4 * delta
				if velocity.y > 0:
					velocity.y = 0

			$Attack/CollisionShape.position = Vector2(4, 0)
			$Sprite.flip_h = false
			$AnimationPlayer.play("side_walk")

	if not is_hit and move_and_collide(velocity * delta):
		change_direction()

func _on_attack_body_entered(body):
	var knockback
	match dir:
		Dir.UP:
			knockback = Vector2(0, -16)
		Dir.DOWN:
			knockback = Vector2(0, 16)
		Dir.LEFT:
			knockback = Vector2(-16, 0)
		Dir.RIGHT:
			knockback = Vector2(16, 0)
	body.hit(knockback)

func hit(knockback):
	if not is_hit and not invincible:
		is_hit = true
		health -= 1
		
		if test_move(transform, knockback):
			knockback_end()
		else:
			var tween = create_tween()
			tween.tween_property(self, "position", position + knockback, 0.1)
			tween.tween_callback(knockback_end)

		chasing = false
		if knockback.x < 0:
			dir = Dir.LEFT
		elif knockback.x > 0:
			dir = Dir.RIGHT
		elif knockback.y < 0:
			dir = Dir.UP
		elif knockback.y > 0:
			dir = Dir.DOWN

		invincible = true
		invincibility_timer.wait_time = INVINCIBILITY
		invincibility_timer.one_shot = true
		invincibility_timer.start()

func knockback_end():
	is_hit = false
	if health <= 0:
		queue_free()

func can_see_player():
	var within_range = position.distance_to(player.position) <= SIGHT
	match dir:
		Dir.UP:
			return within_range and player.position.y < position.y
		Dir.DOWN:
			return within_range and player.position.y > position.y
		Dir.LEFT:
			return within_range and player.position.x < position.x
		Dir.RIGHT:
			return within_range and player.position.x > position.x

func change_direction():
	var last_dir = dir
	while dir == last_dir:
		if chasing:
			var dirs
			match chase_dir:
				Dir.UP:
					dirs = [Dir.UP, Dir.LEFT, Dir.RIGHT]
				Dir.DOWN:
					dirs = [Dir.DOWN, Dir.LEFT, Dir.RIGHT]
				Dir.LEFT:
					dirs = [Dir.LEFT, Dir.UP, Dir.DOWN]
				Dir.RIGHT:
					dirs = [Dir.RIGHT, Dir.UP, Dir.DOWN]
			
			match randi_range(0, 3):
				0, 1:
					dir = dirs[0]
				2:
					dir = dirs[1]
				3:
					dir = dirs[2]
		else:
			dir = randi_range(0, 3)

	change_direction_timer.wait_time = randf_range(0.5, 2)
	change_direction_timer.one_shot = true
	change_direction_timer.start()
