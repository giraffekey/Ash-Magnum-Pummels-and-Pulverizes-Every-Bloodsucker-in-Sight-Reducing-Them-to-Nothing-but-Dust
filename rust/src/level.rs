use crate::math::{attack_positions, compute_fov, line_to, pathfind, Direction, Position};
use crate::ui::AbilityBar;

use godot::engine::{
    AnimationPlayer, AtlasTexture, CanvasLayer, ISprite2D, Sprite2D, Texture2D, TileMap,
};
use godot::global::instance_from_id;
use godot::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

pub const LEVEL_WIDTH: usize = 16;
pub const LEVEL_HEIGHT: usize = 32;
pub const TILE_SIZE: f32 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ability {
    Whip,
    CrossbowIronBolt,
    CrossbowSilverBolt,
    BatBite,
}

#[derive(Debug, Clone, Copy)]
pub enum DamageKind {
    Normal,
    Silver,
    Holy,
    Fire,
    Stake,
}

#[derive(Debug, Clone, Copy)]
pub struct AbilityStats {
    damage_kind: DamageKind,
    damage: u16,
    range: u16,
    targets_enemy: bool,
}

pub fn ability_lists() -> &'static Vec<Vec<Ability>> {
    static ABILITY_LISTS: OnceLock<Vec<Vec<Ability>>> = OnceLock::new();
    ABILITY_LISTS.get_or_init(|| init_ability_lists())
}

fn init_ability_lists() -> Vec<Vec<Ability>> {
    vec![
        vec![
            Ability::Whip,
            Ability::CrossbowIronBolt,
            Ability::CrossbowSilverBolt,
        ],
        vec![Ability::BatBite],
    ]
}

pub fn abilities() -> &'static HashMap<Ability, AbilityStats> {
    static ABILITIES: OnceLock<HashMap<Ability, AbilityStats>> = OnceLock::new();
    ABILITIES.get_or_init(|| init_abilities())
}

fn init_abilities() -> HashMap<Ability, AbilityStats> {
    [
        (
            Ability::Whip,
            AbilityStats {
                damage_kind: DamageKind::Silver,
                damage: 2,
                range: 2,
                targets_enemy: true,
            },
        ),
        (
            Ability::CrossbowIronBolt,
            AbilityStats {
                damage_kind: DamageKind::Normal,
                damage: 2,
                range: 4,
                targets_enemy: true,
            },
        ),
        (
            Ability::CrossbowSilverBolt,
            AbilityStats {
                damage_kind: DamageKind::Silver,
                damage: 2,
                range: 4,
                targets_enemy: true,
            },
        ),
        (
            Ability::BatBite,
            AbilityStats {
                damage_kind: DamageKind::Normal,
                damage: 1,
                range: 1,
                targets_enemy: true,
            },
        ),
    ]
    .into()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, GodotConvert, Var, Export)]
#[godot(via = u8)]
pub enum AllyId {
    #[default]
    AshMagnum,
}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Ally {
    #[export]
    pub id: AllyId,
    pub position: Position,
    #[export]
    pub health: u16,
    #[export]
    pub speed: u16,
    #[export]
    pub view_distance: u16,
    #[export]
    pub ability_list: u8,
    pub abilities: Vec<Ability>,
    pub selected_ability: usize,
    pub has_moved: bool,
    pub has_acted: bool,
    path: Option<Vec<Position>>,
    index: usize,
    #[init(default = "front_idle".into())]
    animation: String,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Ally {
    fn ready(&mut self) {
        let mut animation_player = self
            .base()
            .get_node_as::<AnimationPlayer>("AnimationPlayer");
        animation_player.connect(
            "animation_finished".into(),
            Callable::from_object_method(&self.base(), "animation_end"),
        );

        self.abilities = ability_lists()[self.ability_list as usize].clone();
    }

    fn process(&mut self, _delta: f64) {
        let mut animation_player = self
            .base()
            .get_node_as::<AnimationPlayer>("AnimationPlayer");
        animation_player
            .play_ex()
            .name(self.animation.clone().into())
            .done();
    }
}

#[godot_api]
impl Ally {
    #[func]
    pub fn animation_end(&mut self, name: StringName) {
        let name = name.to_string();

        match name.as_str() {
            "side_whip" | "side_crossbow" | "side_hit" => self.animation = "side_idle".into(),
            "back_whip" | "back_crossbow" | "back_hit" => self.animation = "back_idle".into(),
            "front_whip" | "front_crossbow" | "front_hit" => self.animation = "front_idle".into(),
            "side_death" | "back_death" | "front_death" => self.base_mut().queue_free(),
            _ => (),
        }

        match name.as_str() {
            "side_whip" | "side_crossbow" | "back_whip" | "back_crossbow" | "front_whip"
            | "front_crossbow" => {
                self.has_acted = true;

                let mut cursor = self
                    .base()
                    .get_node_as::<Cursor>("../../../CursorLayer/Cursor");
                let mut cursor = cursor.bind_mut();
                cursor.can_interact = true;
                cursor.selected = None;
            }
            _ => (),
        }
    }

    #[func]
    pub fn next_position(&mut self) {
        if self.index > 0 {
            let mut level = self.base().get_node_as::<Level>("../../..");
            let mut level = level.bind_mut();
            level.shadows_cast = false;
        }

        match &self.path {
            Some(path) if self.index < path.len() => {
                let position = path[self.index];
                let mut tween = self.base_mut().create_tween().unwrap();
                tween.tween_property(
                    self.base().clone().upcast(),
                    "position".into(),
                    Variant::from(position.to_vector()),
                    0.2,
                );
                tween.tween_callback(Callable::from_object_method(&self.base(), "next_position"));

                match self.position.direction_to(position) {
                    Direction::Left => {
                        self.animation = "side_walk".into();
                        self.flip_h(true);
                    }
                    Direction::Right => {
                        self.animation = "side_walk".into();
                        self.flip_h(false);
                    }
                    Direction::Up => {
                        self.animation = "back_walk".into();
                        self.flip_h(false);
                    }
                    Direction::Down => {
                        self.animation = "front_walk".into();
                        self.flip_h(false);
                    }
                }

                self.position = position;
                self.index += 1;
            }
            Some(path) => {
                self.position = *path.last().unwrap();
                self.path = None;
                self.index = 0;
                self.has_moved = true;

                match self.animation.as_str() {
                    "side_walk" => self.animation = "side_idle".into(),
                    "back_walk" => self.animation = "back_idle".into(),
                    "front_walk" => self.animation = "front_idle".into(),
                    _ => unreachable!(),
                }

                let mut cursor = self
                    .base()
                    .get_node_as::<Cursor>("../../../CursorLayer/Cursor");
                let mut cursor = cursor.bind_mut();
                cursor.can_interact = true;
                cursor.acting = true;
            }
            None => (),
        }
    }
}

impl Ally {
    pub fn current_ability(&self) -> &Ability {
        &self.abilities[self.selected_ability]
    }

    pub fn flip_h(&mut self, flip_h: bool) {
        let mut sprite = self.base().get_node_as::<Sprite2D>("Sprite");
        sprite.set_flip_h(flip_h);
    }

    pub fn follow_path(&mut self, path: Vec<Position>) {
        self.path = Some(path);
        self.index = 0;
        self.next_position();
    }

    pub fn use_ability(&mut self, position: Position) {
        match self.current_ability() {
            Ability::Whip => match self.position.direction_to(position) {
                Direction::Left => {
                    self.animation = "side_whip".into();
                    self.flip_h(true);
                }
                Direction::Right => {
                    self.animation = "side_whip".into();
                    self.flip_h(false);
                }
                Direction::Up => {
                    self.animation = "back_whip".into();
                    self.flip_h(false);
                }
                Direction::Down => {
                    self.animation = "front_whip".into();
                    self.flip_h(false);
                }
            },
            Ability::CrossbowIronBolt | Ability::CrossbowSilverBolt => {
                match self.position.direction_to(position) {
                    Direction::Left => {
                        self.animation = "side_crossbow".into();
                        self.flip_h(true);
                    }
                    Direction::Right => {
                        self.animation = "side_crossbow".into();
                        self.flip_h(false);
                    }
                    Direction::Up => {
                        self.animation = "back_crossbow".into();
                        self.flip_h(false);
                    }
                    Direction::Down => {
                        self.animation = "front_crossbow".into();
                        self.flip_h(false);
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn hit(&mut self, damage: u16) {
        self.health = self.health.checked_sub(damage).unwrap_or(0);

        if self.health == 0 {
            match self.animation.as_str() {
                "side_idle" => self.animation = "side_death".into(),
                "back_idle" => self.animation = "back_death".into(),
                "front_idle" => self.animation = "front_death".into(),
                _ => unreachable!(),
            }
        } else {
            match self.animation.as_str() {
                "side_idle" => self.animation = "side_hit".into(),
                "back_idle" => self.animation = "back_hit".into(),
                "front_idle" => self.animation = "front_hit".into(),
                _ => unreachable!(),
            }
        }
    }
}

pub type EnemyId = u16;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Enemy {
    pub id: EnemyId,
    pub position: Position,
    #[export]
    pub y_offset: f32,
    #[export]
    pub health: u16,
    #[export]
    pub speed: u16,
    #[export]
    pub view_distance: u16,
    #[export]
    pub ability_list: u8,
    pub abilities: Vec<Ability>,
    path: Option<Vec<Position>>,
    index: usize,
    current_ability: Option<(AllyId, Ability)>,
    last_known_positions: HashMap<AllyId, Position>,
    #[init(default = "front_idle".into())]
    animation: String,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Enemy {
    fn ready(&mut self) {
        let mut animation_player = self
            .base()
            .get_node_as::<AnimationPlayer>("AnimationPlayer");
        animation_player.connect(
            "animation_finished".into(),
            Callable::from_object_method(&self.base(), "animation_end"),
        );

        self.abilities = ability_lists()[self.ability_list as usize].clone();
    }

    fn process(&mut self, _delta: f64) {
        let mut animation_player = self
            .base()
            .get_node_as::<AnimationPlayer>("AnimationPlayer");
        animation_player
            .play_ex()
            .name(self.animation.clone().into())
            .done();
    }
}

#[godot_api]
impl Enemy {
    #[func]
    pub fn animation_end(&mut self, name: StringName) {
        let name = name.to_string();

        match name.as_str() {
            "side_attack" | "side_hit" => self.animation = "side_idle".into(),
            "back_attack" | "back_hit" => self.animation = "back_idle".into(),
            "front_attack" | "front_hit" => self.animation = "front_idle".into(),
            "side_death" | "back_death" | "front_death" => {
                let mut level = self.base().get_node_as::<Level>("../../..");
                let mut level = level.bind_mut();
                level.grid[self.position.x][self.position.y] = Tile::Empty;
                level.enemies.remove(&self.id);
                let i = level
                    .turn_order
                    .iter()
                    .position(|id| *id == self.id)
                    .unwrap();
                level.turn_order.remove(i);

                self.base_mut().queue_free();
            }
            _ => (),
        }
    }

    #[func]
    pub fn next_position(&mut self) {
        if self.index > 0 {
            let shadow_map = self
                .base()
                .get_node_as::<ShadowMap>("../../../ShadowLayer/ShadowMap");
            let shadow_map = shadow_map.bind();

            let visible = shadow_map.visible.contains(&self.position);
            self.base_mut().set_visible(visible);
        }

        match &self.path {
            Some(path) if self.index < path.len() => {
                let position = path[self.index];
                let mut tween = self.base_mut().create_tween().unwrap();
                tween.tween_property(
                    self.base().clone().upcast(),
                    "position".into(),
                    Variant::from(position.to_vector()),
                    0.2,
                );
                tween.tween_callback(Callable::from_object_method(&self.base(), "next_position"));

                match self.position.direction_to(position) {
                    Direction::Left => {
                        self.animation = "side_walk".into();
                        self.flip_h(true);
                    }
                    Direction::Right => {
                        self.animation = "side_walk".into();
                        self.flip_h(false);
                    }
                    Direction::Up => {
                        self.animation = "back_walk".into();
                        self.flip_h(false);
                    }
                    Direction::Down => {
                        self.animation = "front_walk".into();
                        self.flip_h(false);
                    }
                }

                self.position = position;
                self.index += 1;
            }
            Some(path) => {
                self.position = *path.last().unwrap();
                self.path = None;
                self.index = 0;

                match self.animation.as_str() {
                    "side_walk" => self.animation = "side_idle".into(),
                    "back_walk" => self.animation = "back_idle".into(),
                    "front_walk" => self.animation = "front_idle".into(),
                    _ => unreachable!(),
                }

                let mut level = self.base().get_node_as::<Level>("../../..");
                let mut level = level.bind_mut();
                let Turn::Enemy(i, _) = level.turn else {
                    unreachable!()
                };
                level.turn = Turn::Enemy(i + 1, false);

                if let Some((ally_id, ability)) = self.current_ability {
                    let mut ally = level.get_ally(ally_id);
                    let mut ally = ally.bind_mut();

                    let stats = abilities().get(&ability).unwrap();
                    ally.hit(stats.damage);

                    self.use_ability(ability, ally.position);
                    self.current_ability = None;
                }
            }
            None => (),
        }
    }
}

impl Enemy {
    pub fn plan(
        &mut self,
        grid: [[Tile; LEVEL_HEIGHT]; LEVEL_WIDTH],
        allies: &HashMap<AllyId, i64>,
    ) -> (Option<Vec<Position>>, Option<(Ability, AllyId)>) {
        let visible = compute_fov(self.position, self.view_distance, grid);

        let mut positions = Vec::new();
        for (ally_id, instance_id) in allies {
            let ally: Gd<Ally> = instance_from_id(*instance_id).unwrap().cast();
            let ally = ally.bind();

            if visible.contains(&ally.position) {
                self.last_known_positions.insert(*ally_id, ally.position);

                for ability in &self.abilities {
                    let stats = abilities().get(ability).unwrap();
                    positions.extend(
                        attack_positions(ally.position, stats.range, grid)
                            .iter()
                            .map(|(position, range)| {
                                (
                                    Some(*ability),
                                    *ally_id,
                                    *range,
                                    stats.damage,
                                    pathfind(self.position, *position, grid),
                                )
                            })
                            .filter_map(|(ability, ally_id, range, damage, path)| {
                                path.map(|path| (ability, ally_id, range, damage, path))
                            }),
                    );
                }
            } else if let Some(last_known_position) = self.last_known_positions.get(&ally_id) {
                if let Some(path) = pathfind(self.position, *last_known_position, grid) {
                    positions.push((None, *ally_id, 1, 0, path));
                }
            }
        }

        if positions.is_empty() {
            (None, None)
        } else {
            positions.sort_by(
                |(_, _, a_range, a_damage, a_path), (_, _, b_range, b_damage, b_path)| {
                    let a_cost = a_path.len() as u16;
                    let b_cost = b_path.len() as u16;
                    let a_within = a_cost <= self.speed;
                    let b_within = b_cost <= self.speed;

                    a_within
                        .cmp(&b_within)
                        .reverse()
                        .then(a_damage.cmp(b_damage).reverse())
                        .then(a_range.cmp(b_range).reverse())
                        .then(a_cost.cmp(&b_cost))
                },
            );

            let (ability, ally_id, _, _, path) = positions.first().unwrap();

            if path.len() as u16 <= self.speed {
                (
                    Some(path.clone()),
                    ability.map(|ability| (ability, *ally_id)),
                )
            } else {
                (Some(path[0..self.speed as usize].to_vec()), None)
            }
        }
    }

    pub fn flip_h(&mut self, flip_h: bool) {
        let mut sprite = self.base().get_node_as::<Sprite2D>("Sprite");
        sprite.set_flip_h(flip_h);
    }

    pub fn follow_path(&mut self, path: Vec<Position>) {
        self.path = Some(path);
        self.index = 0;
        self.next_position();
    }

    pub fn use_ability(&mut self, ability: Ability, position: Position) {
        match ability {
            Ability::BatBite => match self.position.direction_to(position) {
                Direction::Left => {
                    self.animation = "side_attack".into();
                    self.flip_h(true);
                }
                Direction::Right => {
                    self.animation = "side_attack".into();
                    self.flip_h(false);
                }
                Direction::Up => {
                    self.animation = "back_attack".into();
                    self.flip_h(false);
                }
                Direction::Down => {
                    self.animation = "front_attack".into();
                    self.flip_h(false);
                }
            },
            _ => unreachable!(),
        }
    }

    pub fn hit(&mut self, damage: u16) {
        self.health = self.health.checked_sub(damage).unwrap_or(0);

        if self.health == 0 {
            match self.animation.as_str() {
                "side_idle" => self.animation = "side_death".into(),
                "back_idle" => self.animation = "back_death".into(),
                "front_idle" => self.animation = "front_death".into(),
                _ => unreachable!(),
            }
        } else {
            match self.animation.as_str() {
                "side_idle" => self.animation = "side_hit".into(),
                "back_idle" => self.animation = "back_hit".into(),
                "front_idle" => self.animation = "front_hit".into(),
                _ => unreachable!(),
            }
        }
    }
}

pub type ObstacleId = u16;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, GodotConvert, Var, Export)]
#[godot(via = u8)]
pub enum ObstacleKind {
    #[default]
    Wall,
    Barrel,
}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Obstacle {
    pub id: ObstacleId,
    pub position: Position,
    #[export]
    pub kind: ObstacleKind,
    base: Base<Node2D>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Tile {
    #[default]
    Empty,
    Ally(AllyId),
    Enemy(EnemyId),
    Obstacle(ObstacleKind),
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Turn {
    #[default]
    Ally,
    Enemy(usize, bool),
}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Level {
    pub grid: [[Tile; LEVEL_HEIGHT]; LEVEL_WIDTH],
    pub turn: Turn,
    pub turn_order: Vec<EnemyId>,
    pub allies: HashMap<AllyId, i64>,
    pub enemies: HashMap<EnemyId, i64>,
    pub obstacles: HashMap<ObstacleId, i64>,
    pub shadows_cast: bool,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Level {
    fn ready(&mut self) {
        let allies = self.base().get_node_as::<Node2D>("UnitLayer/Allies");
        for child in allies.get_children().iter_shared() {
            let mut ally: Gd<Ally> = child.cast();
            let instance_id = ally.instance_id();
            let position = Position::from_vector(ally.get_position());
            let mut ally = ally.bind_mut();
            self.allies.insert(ally.id, instance_id.to_i64());

            ally.position = position;
            self.grid[position.x][position.y] = Tile::Ally(ally.id);

            match ally.id {
                AllyId::AshMagnum => {
                    let mut cursor = self.base().get_node_as::<Cursor>("CursorLayer/Cursor");
                    cursor.set_position(position.to_vector() + Vector2::new(8.0, 8.0));
                    let mut cursor = cursor.bind_mut();
                    cursor.position = position;
                }
            }
        }

        let enemies = self.base().get_node_as::<Node2D>("UnitLayer/Enemies");
        let mut enemy_id = 0;
        let mut turn_order = Vec::new();
        for child in enemies.get_children().iter_shared() {
            let mut enemy: Gd<Enemy> = child.cast();
            let position = enemy.get_position();
            let position = Position::from_vector(position);
            self.enemies.insert(enemy_id, enemy.instance_id().to_i64());

            let mut enemy = enemy.bind_mut();
            enemy.position = position;
            self.grid[position.x][position.y] = Tile::Enemy(enemy_id);
            turn_order.push((enemy_id, enemy.speed));

            enemy.id = enemy_id;
            enemy_id += 1;
        }

        turn_order.sort_by(|(_, a_speed), (_, b_speed)| a_speed.cmp(b_speed).reverse());
        self.turn_order = turn_order.iter().map(|(enemy_id, _)| *enemy_id).collect();

        let obstacles = self.base().get_node_as::<CanvasLayer>("ObstacleLayer");
        let mut obstacle_id = 0;
        for child in obstacles.get_children().iter_shared() {
            let mut obstacle: Gd<Obstacle> = child.cast();
            let position = Position::from_vector(obstacle.get_position());
            self.obstacles
                .insert(obstacle_id, obstacle.instance_id().to_i64());

            let mut obstacle = obstacle.bind_mut();
            obstacle.position = position;
            self.grid[position.x][position.y] = Tile::Obstacle(obstacle.kind);

            obstacle.id = obstacle_id;
            obstacle_id += 1;
        }
    }

    fn process(&mut self, _delta: f64) {
        match self.turn {
            Turn::Ally => {
                if !self.shadows_cast {
                    self.cast_shadows();
                    self.shadows_cast = true;
                }

                let input = Input::singleton();
                if input.is_action_just_pressed("skip".into()) {
                    self.turn = Turn::Enemy(0, false);
                } else {
                    let all_acted = self.allies.keys().all(|ally_id| {
                        let ally = self.get_ally(*ally_id);
                        let ally = ally.bind();
                        ally.has_acted
                    });
                    if all_acted {
                        self.turn = Turn::Enemy(0, false);
                    }
                }
            }
            Turn::Enemy(i, waiting) => {
                if !waiting {
                    if i < self.turn_order.len() {
                        let enemy_id = self.turn_order[i];
                        let mut enemy = self.get_enemy(enemy_id);
                        let mut enemy = enemy.bind_mut();

                        match enemy.animation.as_str() {
                            "side_death" | "front_death" | "back_death" => (),
                            _ => {
                                let (path, ability) = enemy.plan(self.grid, &self.allies);

                                if let Some(path) = path {
                                    let position = path.last().unwrap();
                                    self.grid[enemy.position.x][enemy.position.y] = Tile::Empty;
                                    self.grid[position.x][position.y] = Tile::Enemy(enemy_id);
                                    enemy.follow_path(path);

                                    if let Some((ability, ally_id)) = ability {
                                        enemy.current_ability = Some((ally_id, ability));
                                    }

                                    self.turn = Turn::Enemy(i, true);
                                } else {
                                    self.turn = Turn::Enemy(i + 1, false);
                                }
                            }
                        }
                    } else {
                        self.turn = Turn::Ally;
                        self.shadows_cast = false;

                        for ally_id in self.allies.keys() {
                            let mut ally = self.get_ally(*ally_id);
                            let mut ally = ally.bind_mut();
                            ally.has_moved = false;
                            ally.has_acted = false;
                        }

                        let mut cursor = self.base().get_node_as::<Cursor>("CursorLayer/Cursor");
                        let mut cursor = cursor.bind_mut();
                        cursor.acting = false;
                        cursor.selected = None;

                        let path = self.base().get_node_as::<Path>("PathLayer/Path");
                        let path = path.bind();
                        path.clear_path();
                    }
                }
            }
        }
    }
}

impl Level {
    pub fn at(&self, position: Position) -> Tile {
        self.grid[position.x][position.y]
    }

    pub fn get_ally(&self, ally_id: AllyId) -> Gd<Ally> {
        let instance_id = *self.allies.get(&ally_id).unwrap();
        instance_from_id(instance_id).unwrap().cast()
    }

    pub fn get_enemy(&self, enemy_id: EnemyId) -> Gd<Enemy> {
        let instance_id = *self.enemies.get(&enemy_id).unwrap();
        instance_from_id(instance_id).unwrap().cast()
    }

    pub fn get_obstacle(&self, obstacle_id: ObstacleId) -> Gd<Obstacle> {
        let instance_id = *self.obstacles.get(&obstacle_id).unwrap();
        instance_from_id(instance_id).unwrap().cast()
    }

    pub fn cast_shadows(&self) {
        let mut visible = HashSet::new();
        for ally_id in self.allies.keys() {
            let ally = self.get_ally(*ally_id);
            let ally = ally.bind();
            visible.extend(compute_fov(ally.position, ally.view_distance, self.grid));
        }

        for ally_id in self.allies.keys() {
            let mut ally = self.get_ally(*ally_id);
            let position = ally.bind().position;
            ally.set_visible(visible.contains(&position));
        }

        for enemy_id in self.enemies.keys() {
            let mut enemy = self.get_enemy(*enemy_id);
            let position = enemy.bind().position;
            enemy.set_visible(visible.contains(&position));
        }

        for obstacle_id in self.obstacles.keys() {
            let mut obstacle = self.get_obstacle(*obstacle_id);
            let position = obstacle.bind().position;
            obstacle.set_visible(visible.contains(&position));
        }

        let mut shadow_map = self
            .base()
            .get_node_as::<ShadowMap>("ShadowLayer/ShadowMap");
        let mut shadow_map = shadow_map.bind_mut();
        shadow_map.cast_shadows(visible);
    }

    pub fn move_ally(&mut self, ally_id: AllyId, position: Position) -> bool {
        let mut ally = self.get_ally(ally_id);
        let mut ally = ally.bind_mut();
        if !ally.has_moved {
            match pathfind(ally.position, position, self.grid) {
                Some(path) if !path.is_empty() && path.len() as u16 <= ally.speed => {
                    self.grid[ally.position.x][ally.position.y] = Tile::Empty;
                    self.grid[position.x][position.y] = Tile::Ally(ally_id);
                    ally.follow_path(path);
                    return true;
                }
                _ => (),
            }
        }
        false
    }

    pub fn attack_enemy(&mut self, ally_id: AllyId, enemy_id: EnemyId) -> bool {
        let mut ally = self.get_ally(ally_id);
        let mut ally = ally.bind_mut();
        let mut enemy = self.get_enemy(enemy_id);
        let mut enemy = enemy.bind_mut();

        if !ally.has_acted {
            let stats = abilities().get(ally.current_ability()).unwrap();
            match line_to(ally.position, enemy.position, self.grid) {
                Some(path) if path.len() as u16 <= stats.range => {
                    ally.use_ability(enemy.position);
                    enemy.hit(stats.damage);
                    return true;
                }
                _ => (),
            }
        }

        false
    }
}

#[derive(GodotClass)]
#[class(init, base=TileMap)]
pub struct ShadowMap {
    pub visible: HashSet<Position>,
    base: Base<TileMap>,
}

impl ShadowMap {
    pub fn cast_shadows(&mut self, visible: HashSet<Position>) {
        for x in 0..LEVEL_WIDTH {
            for y in 0..LEVEL_HEIGHT {
                if visible.contains(&Position { x, y }) {
                    self.base_mut()
                        .erase_cell(0, Vector2i::new(x as i32, y as i32));
                } else {
                    self.base_mut()
                        .set_cell_ex(0, Vector2i::new(x as i32, y as i32))
                        .source_id(0)
                        .atlas_coords(Vector2i::new(0, 0))
                        .done();
                }
            }
        }
        self.visible = visible;
    }
}

#[derive(GodotClass)]
#[class(init, base=Sprite2D)]
pub struct Cursor {
    pub position: Position,
    pub selected: Option<AllyId>,
    pub acting: bool,
    #[init(default = true)]
    pub can_interact: bool,
    base: Base<Sprite2D>,
}

#[godot_api]
impl ISprite2D for Cursor {
    fn process(&mut self, _delta: f64) {
        let mut level = self.base().get_node_as::<Level>("../..");
        let mut level = level.bind_mut();
        let mut ability_bar = self
            .base()
            .get_node_as::<AbilityBar>("../../UILayer/AbilityBar");
        let mut ability_bar = ability_bar.bind_mut();

        if self.can_interact && level.turn == Turn::Ally && ability_bar.hovered.is_none() {
            let input = Input::singleton();

            let shadow_map = self
                .base()
                .get_node_as::<ShadowMap>("../../ShadowLayer/ShadowMap");
            let shadow_map = shadow_map.bind();

            let mut position = self.base().get_position();
            if input.is_action_just_pressed("left".into()) {
                let last = self.position;
                if self.move_in_direction(Direction::Left) {
                    if shadow_map.visible.contains(&self.position) {
                        position.x -= 16.0;
                    } else {
                        self.position = last;
                    }
                }
            }
            if input.is_action_just_pressed("right".into()) {
                let last = self.position;
                if self.move_in_direction(Direction::Right) {
                    if shadow_map.visible.contains(&self.position) {
                        position.x += 16.0;
                    } else {
                        self.position = last;
                    }
                }
            }
            if input.is_action_just_pressed("up".into()) {
                let last = self.position;
                if self.move_in_direction(Direction::Up) {
                    if shadow_map.visible.contains(&self.position) {
                        position.y -= 16.0;
                    } else {
                        self.position = last;
                    }
                }
            }
            if input.is_action_just_pressed("down".into()) {
                let last = self.position;
                if self.move_in_direction(Direction::Down) {
                    if shadow_map.visible.contains(&self.position) {
                        position.y += 16.0;
                    } else {
                        self.position = last;
                    }
                }
            }
            self.base_mut().set_position(position);

            let mut path_node = self.base().get_node_as::<Path>("../../PathLayer/Path");
            let mut path_node = path_node.bind_mut();

            if input.is_action_just_pressed("use_ability".into()) && self.selected.is_some() {
                if let Some(selected) = self.selected {
                    let ally = level.get_ally(selected);
                    let ally = ally.bind();

                    if ally.has_moved {
                        self.acting = true;
                    } else {
                        self.acting = !self.acting;
                    }
                }
            }

            if input.is_action_just_pressed("select".into()) {
                match level.at(self.position) {
                    Tile::Empty if !self.acting => {
                        if let Some(selected) = self.selected {
                            if level.move_ally(selected, self.position) {
                                path_node.clear_path();
                                self.can_interact = false;
                            }
                        }
                    }
                    Tile::Ally(id) => {
                        self.selected = Some(id);

                        {
                            let ally = level.get_ally(id);
                            let ally = ally.bind();
                            if ally.has_moved {
                                self.acting = true;
                            }
                        }

                        ability_bar.select_ally(&level, id);
                    }
                    Tile::Enemy(id) if self.acting => {
                        if let Some(selected) = self.selected {
                            if level.attack_enemy(selected, id) {
                                path_node.clear_path();
                                self.can_interact = false;
                                self.acting = false;
                            }
                        }
                    }
                    _ => (),
                }
            } else {
                match level.at(self.position) {
                    Tile::Empty => {
                        if let Some(selected) = self.selected {
                            let ally = level.get_ally(selected);
                            let ally = ally.bind();

                            if self.position != ally.position {
                                if self.acting {
                                    path_node.set_path(vec![self.position], PathKind::Attack);
                                } else {
                                    match pathfind(ally.position, self.position, level.grid) {
                                        Some(path) if path.len() as u16 <= ally.speed => {
                                            path_node.set_path(path, PathKind::Move);
                                        }
                                        _ => path_node.clear_path(),
                                    }
                                }
                            } else {
                                path_node.clear_path();
                            }
                        }
                    }
                    Tile::Enemy(_) if self.acting => {
                        if let Some(selected) = self.selected {
                            let ally = level.get_ally(selected);
                            let ally = ally.bind();

                            let stats = abilities().get(ally.current_ability()).unwrap();
                            if stats.targets_enemy {
                                match line_to(ally.position, self.position, level.grid) {
                                    Some(path) if path.len() as u16 <= stats.range => {
                                        path_node.set_path(path, PathKind::Attack);
                                    }
                                    _ => path_node.set_path(vec![self.position], PathKind::Attack),
                                }
                            } else {
                                path_node.set_path(vec![self.position], PathKind::Attack);
                            }
                        }
                    }
                    _ => path_node.clear_path(),
                }

                let mut atlas: Gd<AtlasTexture> = self.base().get_texture().unwrap().cast();
                match level.at(self.position) {
                    Tile::Ally(_) => {
                        atlas.set_region(Rect2::new(
                            Vector2::new(16.0, 0.0),
                            Vector2::new(16.0, 16.0),
                        ));
                    }
                    _ => {
                        atlas.set_region(Rect2::new(
                            Vector2::new(0.0, 0.0),
                            Vector2::new(16.0, 16.0),
                        ));
                    }
                }
            }
        }
    }
}

impl Cursor {
    pub fn move_in_direction(&mut self, direction: Direction) -> bool {
        match direction {
            Direction::Left => {
                if self.position.x > 0 {
                    self.position.x -= 1;
                    return true;
                }
            }
            Direction::Right => {
                if self.position.x < LEVEL_WIDTH - 1 {
                    self.position.x += 1;
                    return true;
                }
            }
            Direction::Up => {
                if self.position.y > 0 {
                    self.position.y -= 1;
                    return true;
                }
            }
            Direction::Down => {
                if self.position.y < LEVEL_HEIGHT - 1 {
                    self.position.y += 1;
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PathKind {
    Move,
    Attack,
}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Path {
    base: Base<Node2D>,
}

impl Path {
    pub fn clear_path(&self) {
        for mut child in self.base().get_children().iter_shared() {
            child.queue_free();
        }
    }

    pub fn set_path(&mut self, path: Vec<Position>, kind: PathKind) {
        self.clear_path();

        let texture = load::<Texture2D>("res://assets/sprites/cursor.png");
        for position in path {
            let mut sprite = Sprite2D::new_alloc();

            let mut atlas = AtlasTexture::new_gd();
            atlas.set_atlas(texture.clone());

            match kind {
                PathKind::Move => atlas.set_region(Rect2::new(
                    Vector2::new(32.0, 0.0),
                    Vector2::new(16.0, 16.0),
                )),
                PathKind::Attack => atlas.set_region(Rect2::new(
                    Vector2::new(48.0, 0.0),
                    Vector2::new(16.0, 16.0),
                )),
            }

            sprite.set_texture(atlas.upcast());
            sprite.set_position(position.to_vector() + Vector2::new(8.0, 8.0));

            self.base_mut().add_child(sprite.upcast());
        }
    }
}
