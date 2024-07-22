extends CharacterBody2D


enum Dir { UP, DOWN, LEFT, RIGHT }

const INVINCIBILITY = 0.5
const SPEED = 48
const ATTACK_COOLDOWN = 0.1

var health = 3
var is_hit = false
var invincible = false
var dir
var moving = false
var attacking = false
var can_attack = true
var invincibility_timer
var attack_cooldown_timer


func _ready():
	dir = Dir.DOWN

	invincibility_timer = Timer.new()
	add_child(invincibility_timer)
	invincibility_timer.timeout.connect(func(): invincible = false)

	attack_cooldown_timer = Timer.new()
	add_child(attack_cooldown_timer)
	attack_cooldown_timer.timeout.connect(attack_cooldown_finished)

	$Whip/Sprite.visible = false
	$AnimationPlayer.play("front_idle")

func _physics_process(delta):
	if attacking:
		match dir:
			Dir.UP:
				$AnimationPlayer.play("back_attack")
				$Whip/AnimationPlayer.play("back_whip")
			Dir.DOWN:
				$AnimationPlayer.play("front_attack")
				$Whip/AnimationPlayer.play("front_whip")
			Dir.LEFT, Dir.RIGHT:
				$AnimationPlayer.play("side_attack")
				$Whip/AnimationPlayer.play("side_whip")
	else:
		moving = false

		if Input.is_action_pressed("move_up"):
			velocity.y -= SPEED * 4 * delta
			if velocity.y < -SPEED:
				velocity.y = -SPEED
			dir = Dir.UP
			moving = true
			$Sprite.flip_h = false
			$Whip/Sprite.flip_h = false
			$Whip/AnimationPlayer.set_assigned_animation("back_whip")
			$Whip/AnimationPlayer.seek(0.0, true)
		elif Input.is_action_pressed("move_down"):
			velocity.y += SPEED * 4 * delta
			if velocity.y > SPEED:
				velocity.y = SPEED
			dir = Dir.DOWN
			moving = true
			$Sprite.flip_h = false
			$Whip/Sprite.flip_h = false
			$Whip/AnimationPlayer.set_assigned_animation("front_whip")
			$Whip/AnimationPlayer.seek(0.0, true)
		else:
			if velocity.y > 0:
				velocity.y -= SPEED * 4 * delta
				if velocity.y < 0:
					velocity.y = 0
			elif velocity.y < 0:
				velocity.y += SPEED * 4 * delta
				if velocity.y > 0:
					velocity.y = 0

		if Input.is_action_pressed("move_left"):
			velocity.x -= SPEED * 4 * delta
			if velocity.x < -SPEED:
				velocity.x = -SPEED
			dir = Dir.LEFT
			moving = true
			$Sprite.flip_h = true
			$Whip/Sprite.flip_h = true
			$Whip/AnimationPlayer.set_assigned_animation("side_whip")
			$Whip/AnimationPlayer.seek(0.0, true)
		elif Input.is_action_pressed("move_right"):
			velocity.x += SPEED * 4 * delta
			if velocity.x > SPEED:
				velocity.x = SPEED
			dir = Dir.RIGHT
			moving = true
			$Sprite.flip_h = false
			$Whip/Sprite.flip_h = false
			$Whip/AnimationPlayer.set_assigned_animation("side_whip")
			$Whip/AnimationPlayer.seek(0.0, true)
		else:
			if velocity.x > 0:
				velocity.x -= SPEED * 4 * delta
				if velocity.x < 0:
					velocity.x = 0
			elif velocity.x < 0:
				velocity.x += SPEED * 4 * delta
				if velocity.x > 0:
					velocity.x = 0

		if can_attack and Input.is_action_just_pressed("attack"):
			velocity = Vector2(0, 0)

			moving = false
			attacking = true

			match dir:
				Dir.UP:
					$Whip/CollisionShape.position = Vector2(0, -12)
				Dir.DOWN:
					$Whip/CollisionShape.position = Vector2(0, 12)
				Dir.LEFT:
					$Whip/CollisionShape.position = Vector2(-12, 0)
				Dir.RIGHT:
					$Whip/CollisionShape.position = Vector2(12, 0)

			$Whip/CollisionShape.shape.size = Vector2(24, 24)
			$Whip/Sprite.visible = true

		if moving and not is_hit:
			match dir:
				Dir.UP:
					$AnimationPlayer.play("back_walk")
				Dir.DOWN:
					$AnimationPlayer.play("front_walk")
				Dir.LEFT:
					$AnimationPlayer.play("side_walk")
				Dir.RIGHT:
					$AnimationPlayer.play("side_walk")

			move_and_collide(velocity * delta)
		else:
			match dir:
				Dir.UP:
					$AnimationPlayer.play("back_idle")
				Dir.DOWN:
					$AnimationPlayer.play("front_idle")
				Dir.LEFT:
					$AnimationPlayer.play("side_idle")
				Dir.RIGHT:
					$AnimationPlayer.play("side_idle")

func _on_animation_finished(anim_name):
	if anim_name == "front_attack" or anim_name == "back_attack" or anim_name == "side_attack":
		attacking = false
		can_attack = false
		$Whip/CollisionShape.shape.size = Vector2(0, 0)
		$Whip/CollisionShape.position = Vector2(0, 0)
		$Whip/Sprite.visible = false

		attack_cooldown_timer.wait_time = ATTACK_COOLDOWN
		attack_cooldown_timer.one_shot = true
		attack_cooldown_timer.start()

func _on_whip_body_entered(body):
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

		invincible = true
		invincibility_timer.wait_time = INVINCIBILITY
		invincibility_timer.one_shot = true
		invincibility_timer.start()

func knockback_end():
	is_hit = false
	if health <= 0:
		queue_free()

func attack_cooldown_finished():
	can_attack = true
