use crate::ability::{abilities, ability_lists, Ability, Action, DamageKind};
use crate::death_screen::DeathScreen;
use crate::dialogue::{Dialogue, DialogueEvent, Room};
use crate::math::{attack_positions, compute_fov, line_to, pathfind, Direction, Position};
use crate::traits::{trait_lists, Trait};
use crate::ui::{AbilityBar, InfoPanel};

use godot::engine::{
    AnimationPlayer, AtlasTexture, CanvasLayer, ISprite2D, Sprite2D, Texture2D, TileMap,
};
use godot::global::instance_from_id;
use godot::prelude::*;
use std::cmp::{self, Ordering};
use std::collections::{HashMap, HashSet};

pub const LEVEL_WIDTH: usize = 16;
pub const LEVEL_HEIGHT: usize = 32;
pub const TILE_SIZE: f32 = 16.0;
pub const DOOR_TILES: [Position; 2] = [Position { x: 7, y: 0 }, Position { x: 8, y: 0 }];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effect {
    Burn,
    Mist,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectStats {
    pub magnitude: u16,
    pub duration: u16,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, GodotConvert, Var, Export)]
#[godot(via = u8)]
pub enum AllyId {
    #[default]
    AshMagnum,
    Alukrod,
}

impl AllyId {
    pub fn name(&self) -> String {
        match self {
            Self::AshMagnum => "Ash Magnum".into(),
            Self::Alukrod => "Alukrod".into(),
        }
    }
}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Ally {
    #[export]
    pub id: AllyId,
    pub position: Position,
    #[export]
    pub max_health: u16,
    pub health: u16,
    #[export]
    pub speed: u16,
    #[export]
    pub view_distance: u16,
    #[export]
    pub ability_list: u8,
    pub abilities: Vec<Ability>,
    pub uses: HashMap<Ability, u16>,
    #[export]
    pub trait_list: u8,
    pub traits: Vec<Trait>,
    pub selected_ability: usize,
    pub has_moved: bool,
    pub has_acted: bool,
    pub effects: HashMap<Effect, EffectStats>,
    path: Option<Vec<Position>>,
    index: usize,
    #[init(default = "front_idle".into())]
    animation: String,
    whip_animation: Option<String>,
    sword_animation: Option<String>,
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

        self.health = self.max_health;

        let ability_list = ability_lists()[self.ability_list as usize].clone();
        for (ability, uses) in &ability_list {
            self.uses.insert(*ability, *uses);
        }
        self.abilities = ability_list
            .iter()
            .map(|(ability, _)| ability)
            .copied()
            .collect();

        self.traits = trait_lists()[self.trait_list as usize].clone();
    }

    fn process(&mut self, _delta: f64) {
        let mut animation_player = self
            .base()
            .get_node_as::<AnimationPlayer>("AnimationPlayer");
        animation_player
            .play_ex()
            .name(self.animation.clone().into())
            .done();

        if let Some(whip_animation) = &self.whip_animation {
            let mut animation_player = self
                .base()
                .get_node_as::<AnimationPlayer>("Whip/AnimationPlayer");
            animation_player
                .play_ex()
                .name(whip_animation.into())
                .done();
        }

        if let Some(sword_animation) = &self.sword_animation {
            let mut animation_player = self
                .base()
                .get_node_as::<AnimationPlayer>("Sword/AnimationPlayer");
            animation_player
                .play_ex()
                .name(sword_animation.into())
                .done();
        }
    }
}

#[godot_api]
impl Ally {
    #[func]
    pub fn animation_end(&mut self, name: StringName) {
        let name = name.to_string();

        match name.as_str() {
            "side_whip" => {
                self.animation = "side_idle".into();

                let mut whip = self.base().get_node_as::<Node2D>("Whip");
                whip.set_visible(false);
            }
            "back_whip" => {
                self.animation = "back_idle".into();

                let mut whip = self.base().get_node_as::<Node2D>("Whip");
                whip.set_visible(false);
            }
            "front_whip" => {
                self.animation = "front_idle".into();

                let mut whip = self.base().get_node_as::<Node2D>("Whip");
                whip.set_visible(false);
            }
            "side_sword" => {
                self.animation = "side_idle".into();

                let mut sword = self.base().get_node_as::<Node2D>("Sword");
                sword.set_visible(false);
            }
            "back_sword" => {
                self.animation = "back_idle".into();

                let mut sword = self.base().get_node_as::<Node2D>("Sword");
                sword.set_visible(false);
            }
            "front_sword" => {
                self.animation = "front_idle".into();

                let mut sword = self.base().get_node_as::<Node2D>("Sword");
                sword.set_visible(false);
            }
            "side_crossbow" | "side_hellfire" | "side_bite" | "side_mist" | "side_stake"
            | "side_hit" => self.animation = "side_idle".into(),
            "back_crossbow" | "back_hellfire" | "back_bite" | "back_mist" | "back_stake"
            | "back_hit" => self.animation = "back_idle".into(),
            "front_crossbow" | "front_hellfire" | "front_bite" | "front_mist" | "front_stake"
            | "front_hit" => self.animation = "front_idle".into(),
            "side_death" | "back_death" | "front_death" => {
                let mut level_node = self.base().get_node_as::<Level>("../../..");
                let mut level = level_node.bind_mut();

                match self.id {
                    AllyId::AshMagnum => {
                        let scene = load::<PackedScene>("res://scenes/death.tscn");
                        let mut scene: Gd<DeathScreen> = scene.instantiate().unwrap().cast();

                        {
                            let mut scene = scene.bind_mut();
                            scene.room = level.room;
                        }

                        self.base()
                            .get_tree()
                            .unwrap()
                            .get_root()
                            .unwrap()
                            .add_child(scene.clone().upcast());
                        self.base()
                            .get_tree()
                            .unwrap()
                            .set_current_scene(scene.upcast());

                        drop(level);
                        level_node.queue_free();
                    }
                    _ => {
                        level.grid[self.position.x][self.position.y] = Tile::Empty;
                        level.allies.remove(&self.id);

                        let mut dialogue = self.base().get_node_as::<Dialogue>("../../../Dialogue");
                        let mut dialogue = dialogue.bind_mut();
                        dialogue.push_event(DialogueEvent::AllyKilled(self.id));

                        self.base_mut().queue_free();
                    }
                }
            }
            _ => (),
        }

        match name.as_str() {
            "side_whip" | "side_crossbow" | "side_sword" | "side_hellfire" | "side_bite"
            | "side_mist" | "side_stake" | "back_whip" | "back_crossbow" | "back_sword"
            | "back_hellfire" | "back_bite" | "back_mist" | "back_stake" | "front_whip"
            | "front_crossbow" | "front_sword" | "front_hellfire" | "front_bite" | "front_mist"
            | "front_stake" => {
                self.has_acted = true;

                let mut cursor = self
                    .base()
                    .get_node_as::<Cursor>("../../../CursorLayer/Cursor");
                let mut cursor = cursor.bind_mut();
                cursor.can_interact = true;
                cursor.selected = None;

                let mut ability_bar = self
                    .base()
                    .get_node_as::<AbilityBar>("../../../UILayer/AbilityBar");
                let mut ability_bar = ability_bar.bind_mut();
                ability_bar.select_none();
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
                    0.3,
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

                let mut level_node = self.base().get_node_as::<Level>("../../..");
                let mut level = level_node.bind_mut();

                if DOOR_TILES.contains(&self.position) {
                    let scene = match level.room {
                        Room::EntranceHall => "res://scenes/levels/2-great-hall.tscn",
                        Room::GreatHall => {
                            self.base()
                                .get_tree()
                                .unwrap()
                                .change_scene_to_file("res://scenes/end.tscn".into());
                            return;
                        }
                    };

                    let scene = load::<PackedScene>(scene);
                    let mut next_level: Gd<Level> = scene.instantiate().unwrap().cast();

                    {
                        let mut next_level = next_level.bind_mut();
                        for ally_id in level.allies.keys() {
                            let (abilities, uses) = if self.id == *ally_id {
                                (self.abilities.clone(), self.uses.clone())
                            } else {
                                let ally = level.get_ally(*ally_id);
                                let ally = ally.bind();
                                (ally.abilities.clone(), ally.uses.clone())
                            };
                            let inventory = abilities
                                .iter()
                                .map(|ability| (*ability, uses[ability]))
                                .collect();
                            next_level.inventory.insert(*ally_id, inventory);
                        }
                    }

                    self.base()
                        .get_tree()
                        .unwrap()
                        .get_root()
                        .unwrap()
                        .add_child(next_level.clone().upcast());
                    self.base()
                        .get_tree()
                        .unwrap()
                        .set_current_scene(next_level.upcast());

                    drop(level);
                    level_node.queue_free();
                } else {
                    match self.animation.as_str() {
                        "side_walk" => self.animation = "side_idle".into(),
                        "back_walk" => self.animation = "back_idle".into(),
                        "front_walk" => self.animation = "front_idle".into(),
                        _ => unreachable!(),
                    }

                    match level.at(self.position) {
                        Tile::Item(id) => {
                            let mut item = level.get_item(id);

                            let picked_up = {
                                let item = item.bind();
                                let ability = item.ability();
                                let stats = abilities().get(&ability).unwrap();

                                if stats.acquirable || self.abilities.contains(&ability) {
                                    match self.uses.get_mut(&ability) {
                                        Some(n) => *n += 1,
                                        None => {
                                            self.abilities.push(ability);
                                            self.uses.insert(ability, 1);
                                        }
                                    }
                                    level.items.remove(&id);
                                    true
                                } else {
                                    false
                                }
                            };

                            if picked_up {
                                item.queue_free();
                            }
                        }
                        _ => (),
                    }

                    level.grid[self.position.x][self.position.y] = Tile::Ally(self.id);

                    let mut cursor = self
                        .base()
                        .get_node_as::<Cursor>("../../../CursorLayer/Cursor");
                    let mut cursor = cursor.bind_mut();
                    cursor.can_interact = true;
                    cursor.acting = true;

                    let mut ability_bar = self
                        .base()
                        .get_node_as::<AbilityBar>("../../../UILayer/AbilityBar");
                    let mut ability_bar = ability_bar.bind_mut();
                    ability_bar.select_ally(&self);
                }
            }
            None => (),
        }
    }
}

impl Ally {
    pub fn name(&self) -> String {
        self.id.name()
    }

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

    pub fn use_ability(&mut self, position: Position) -> Option<Gd<Projectile>> {
        let ability = *self.current_ability();
        let stats = abilities().get(&ability).unwrap();
        if stats.consumable {
            let uses = self.uses.get_mut(&ability).unwrap();
            *uses -= 1;

            if stats.acquirable && *uses == 0 {
                self.abilities.remove(self.selected_ability);
                self.uses.remove(&ability);

                if self.selected_ability >= self.abilities.len() {
                    self.selected_ability = self.abilities.len() - 1;
                }
            }
        }

        match ability {
            Ability::Whip | Ability::Thwack => match self.position.direction_to(position) {
                Direction::Left => {
                    self.animation = "side_whip".into();
                    self.flip_h(true);

                    let mut whip = self.base().get_node_as::<Node2D>("Whip");
                    self.whip_animation = Some("side".into());
                    whip.set_visible(true);
                    whip.get_node_as::<Sprite2D>("Sprite").set_flip_h(true);
                }
                Direction::Right => {
                    self.animation = "side_whip".into();
                    self.flip_h(false);

                    let mut whip = self.base().get_node_as::<Node2D>("Whip");
                    self.whip_animation = Some("side".into());
                    whip.set_visible(true);
                    whip.get_node_as::<Sprite2D>("Sprite").set_flip_h(false);
                }
                Direction::Up => {
                    self.animation = "back_whip".into();
                    self.flip_h(false);

                    let mut whip = self.base().get_node_as::<Node2D>("Whip");
                    self.whip_animation = Some("back".into());
                    whip.set_visible(true);
                    whip.get_node_as::<Sprite2D>("Sprite").set_flip_h(false);
                }
                Direction::Down => {
                    self.animation = "front_whip".into();
                    self.flip_h(false);

                    let mut whip = self.base().get_node_as::<Node2D>("Whip");
                    self.whip_animation = Some("front".into());
                    whip.set_visible(true);
                    whip.get_node_as::<Sprite2D>("Sprite").set_flip_h(false);
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
            Ability::Sword => match self.position.direction_to(position) {
                Direction::Left => {
                    self.animation = "side_sword".into();
                    self.flip_h(true);

                    let mut sword = self.base().get_node_as::<Node2D>("Sword");
                    self.sword_animation = Some("side".into());
                    sword.set_visible(true);
                    sword.get_node_as::<Sprite2D>("Sprite").set_flip_h(true);
                }
                Direction::Right => {
                    self.animation = "side_sword".into();
                    self.flip_h(false);

                    let mut sword = self.base().get_node_as::<Node2D>("Sword");
                    self.sword_animation = Some("side".into());
                    sword.set_visible(true);
                    sword.get_node_as::<Sprite2D>("Sprite").set_flip_h(false);
                }
                Direction::Up => {
                    self.animation = "back_sword".into();
                    self.flip_h(false);

                    let mut sword = self.base().get_node_as::<Node2D>("Sword");
                    self.sword_animation = Some("back".into());
                    sword.set_visible(true);
                    sword.get_node_as::<Sprite2D>("Sprite").set_flip_h(false);
                }
                Direction::Down => {
                    self.animation = "front_sword".into();
                    self.flip_h(false);

                    let mut sword = self.base().get_node_as::<Node2D>("Sword");
                    self.sword_animation = Some("front".into());
                    sword.set_visible(true);
                    sword.get_node_as::<Sprite2D>("Sprite").set_flip_h(false);
                }
            },
            Ability::Hellfire => match self.position.direction_to(position) {
                Direction::Left => {
                    self.animation = "side_hellfire".into();
                    self.flip_h(true);
                }
                Direction::Right => {
                    self.animation = "side_hellfire".into();
                    self.flip_h(false);
                }
                Direction::Up => {
                    self.animation = "back_hellfire".into();
                    self.flip_h(false);
                }
                Direction::Down => {
                    self.animation = "front_hellfire".into();
                    self.flip_h(false);
                }
            },
            Ability::VampireBite => match self.position.direction_to(position) {
                Direction::Left => {
                    self.animation = "side_bite".into();
                    self.flip_h(true);
                }
                Direction::Right => {
                    self.animation = "side_bite".into();
                    self.flip_h(false);
                }
                Direction::Up => {
                    self.animation = "back_bite".into();
                    self.flip_h(false);
                }
                Direction::Down => {
                    self.animation = "front_bite".into();
                    self.flip_h(false);
                }
            },
            Ability::Mist => match self.animation.as_str() {
                "side_idle" => self.animation = "side_mist".into(),
                "back_idle" => self.animation = "back_mist".into(),
                "front_idle" => self.animation = "front_mist".into(),
                _ => unreachable!(),
            },
            Ability::WoodenStake | Ability::Garlic | Ability::HolyWater => {
                match self.position.direction_to(position) {
                    Direction::Left => {
                        self.animation = "side_stake".into();
                        self.flip_h(true);
                    }
                    Direction::Right => {
                        self.animation = "side_stake".into();
                        self.flip_h(false);
                    }
                    Direction::Up => {
                        self.animation = "back_stake".into();
                        self.flip_h(false);
                    }
                    Direction::Down => {
                        self.animation = "front_stake".into();
                        self.flip_h(false);
                    }
                }
            }
            _ => unreachable!(),
        }

        match ability {
            Ability::CrossbowIronBolt => {
                let projectile = Projectile::new(ProjectileKind::IronBolt, self.position, position);
                Some(projectile)
            }
            Ability::CrossbowSilverBolt => {
                let projectile =
                    Projectile::new(ProjectileKind::SilverBolt, self.position, position);
                Some(projectile)
            }
            Ability::Hellfire => {
                let projectile = Projectile::new(ProjectileKind::Fireball, self.position, position);
                Some(projectile)
            }
            _ => None,
        }
    }

    pub fn heal(&mut self, amount: u16) {
        self.health = cmp::min(self.health + amount, self.max_health);
    }

    pub fn hit(&mut self, damage: u16, damage_kind: DamageKind) {
        if !self.effects.contains_key(&Effect::Mist) {
            let damage = damage + damage_bonus(damage_kind, &self.traits);
            self.health = self.health.checked_sub(damage).unwrap_or(0);

            if damage_kind == DamageKind::Fire {
                match self.effects.get_mut(&Effect::Burn) {
                    Some(stats) => stats.magnitude += 1,
                    None => {
                        self.effects.insert(
                            Effect::Burn,
                            EffectStats {
                                magnitude: 1,
                                duration: 3,
                            },
                        );
                    }
                }
            }

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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyAction {
    Attack {
        ally_id: AllyId,
        damage_kind: DamageKind,
        damage: u16,
    },
    Spawn {
        enemy_kind: EnemyKind,
        position: Position,
    },
}

pub type EnemyId = u16;

#[derive(Debug, Clone, Copy, Default, PartialEq, Var, Export, GodotConvert)]
#[godot(via = u8)]
pub enum EnemyKind {
    #[default]
    Bat,
    Vampire,
    BigBatty,
}

impl EnemyKind {
    pub fn name(&self) -> String {
        match self {
            Self::Bat => "Bat".into(),
            Self::Vampire => "Vampire".into(),
            Self::BigBatty => "BigBatty".into(),
        }
    }
}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Enemy {
    pub id: EnemyId,
    pub position: Position,
    #[export]
    pub kind: EnemyKind,
    #[export]
    pub max_health: u16,
    pub health: u16,
    #[export]
    pub speed: u16,
    #[export]
    pub view_distance: u16,
    #[export]
    pub width: u16,
    #[export]
    pub height: u16,
    #[export]
    pub ability_list: u8,
    pub abilities: Vec<Ability>,
    pub uses: HashMap<Ability, u16>,
    pub cooldowns: HashMap<Ability, u16>,
    #[export]
    pub trait_list: u8,
    pub traits: Vec<Trait>,
    pub effects: HashMap<Effect, EffectStats>,
    path: Option<Vec<Position>>,
    index: usize,
    current_ability: Option<(Ability, EnemyAction)>,
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

        self.health = self.max_health;

        let ability_list = ability_lists()[self.ability_list as usize].clone();
        for (ability, uses) in &ability_list {
            self.uses.insert(*ability, *uses);
        }
        self.abilities = ability_list
            .iter()
            .map(|(ability, _)| ability)
            .copied()
            .collect();

        self.traits = trait_lists()[self.trait_list as usize].clone();
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

                for i in 0..self.width as usize {
                    for j in 0..self.height as usize {
                        level.grid[self.position.x + i][self.position.y + j] = Tile::Empty;
                    }
                }

                level.enemies.remove(&self.id);
                if let Some(i) = level.turn_order.iter().position(|(id, _)| *id == self.id) {
                    level.turn_order.remove(i);
                }

                let mut dialogue = self.base().get_node_as::<Dialogue>("../../../Dialogue");
                let mut dialogue = dialogue.bind_mut();
                dialogue.push_event(DialogueEvent::EnemyKilled(self.kind));

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
                    0.3,
                );
                tween.tween_callback(Callable::from_object_method(&self.base(), "next_position"));

                if self.position != position {
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
                    "side_idle" | "back_idle" | "front_idle" => (),
                    _ => unreachable!(),
                }

                let mut level = self.base().get_node_as::<Level>("../../..");
                let mut level = level.bind_mut();
                let Turn::Enemy(i, _) = level.turn else {
                    unreachable!()
                };
                level.turn = Turn::Enemy(i + 1, false);

                for (_, cooldown) in &mut self.cooldowns {
                    if *cooldown > 0 {
                        *cooldown -= 1;
                    }
                }

                if let Some((ability, action)) = self.current_ability {
                    match action {
                        EnemyAction::Attack {
                            ally_id,
                            damage_kind,
                            damage,
                        } => {
                            let mut ally = level.get_ally(ally_id);
                            let mut ally = ally.bind_mut();
                            ally.hit(damage, damage_kind);

                            match damage_kind {
                                DamageKind::LifeSteal => self.heal(damage),
                                _ => (),
                            }

                            self.use_ability(ability, ally.position);
                            self.current_ability = None;
                        }
                        EnemyAction::Spawn {
                            enemy_kind,
                            position,
                        } => {
                            let stats = abilities().get(&ability).unwrap();
                            match stats.action {
                                Action::Spawn { cooldown, .. } => {
                                    self.cooldowns.insert(ability, cooldown);
                                }
                                _ => (),
                            }

                            level.spawn_enemy(enemy_kind, position);

                            self.use_ability(ability, position);
                            self.current_ability = None;
                        }
                    }
                }

                let mut dialogue = self.base().get_node_as::<Dialogue>("../../../Dialogue");
                let mut dialogue = dialogue.bind_mut();
                dialogue.push_event(DialogueEvent::EnemyMoved(self.kind));
            }
            None => (),
        }
    }
}

impl Enemy {
    pub fn name(&self) -> String {
        self.kind.name()
    }

    pub fn plan(
        &mut self,
        level: &Level,
    ) -> (Option<Vec<Position>>, Option<(Ability, EnemyAction)>) {
        let visible = compute_fov(self.position, self.view_distance, level);
        let dimensions = (self.width as usize, self.height as usize);

        let mut grid = level.grid;
        if self.traits.contains(&Trait::GarlicAllergy) {
            for item_id in level.items.keys() {
                let item = level.get_item(*item_id);
                let item = item.bind();
                match item.kind {
                    ItemKind::Garlic => {
                        grid[item.position.x][item.position.y] = Tile::Obstacle(0);

                        for position in item.position.adjacent() {
                            grid[position.x][position.y] = Tile::Obstacle(0);
                        }
                    }
                    _ => (),
                }
            }
        }

        let mut actions = Vec::new();
        for ability in &self.abilities {
            let stats = abilities().get(ability).unwrap();
            match stats.action {
                Action::Attack {
                    damage_kind,
                    damage,
                    ..
                } => {
                    for (ally_id, instance_id) in &level.allies {
                        let ally: Gd<Ally> = instance_from_id(*instance_id).unwrap().cast();
                        let ally = ally.bind();

                        if visible.contains(&ally.position) {
                            self.last_known_positions.insert(*ally_id, ally.position);
                            actions.extend(
                                attack_positions(ally.position, stats.range, grid, dimensions)
                                    .iter()
                                    .map(|(position, range)| {
                                        (
                                            Some(*ability),
                                            *ally_id,
                                            *range,
                                            pathfind(
                                                self.position,
                                                *position,
                                                grid,
                                                Tile::Enemy(self.id),
                                                dimensions,
                                            ),
                                        )
                                    })
                                    .filter_map(|(ability, ally_id, range, path)| {
                                        path.map(|path| {
                                            (
                                                ability,
                                                EnemyAction::Attack {
                                                    ally_id,
                                                    damage_kind,
                                                    damage,
                                                },
                                                range,
                                                path,
                                            )
                                        })
                                    }),
                            );
                        } else if let Some(last_known_position) =
                            self.last_known_positions.get(&ally_id)
                        {
                            if let Some(path) = pathfind(
                                self.position,
                                *last_known_position,
                                grid,
                                Tile::Enemy(self.id),
                                dimensions,
                            ) {
                                actions.push((
                                    None,
                                    EnemyAction::Attack {
                                        ally_id: *ally_id,
                                        damage_kind,
                                        damage,
                                    },
                                    1,
                                    path,
                                ));
                            }
                        }
                    }
                }
                Action::Spawn { enemy_kind, .. } => {
                    let cooldown_finished = *self.cooldowns.get(&ability).unwrap_or(&0) == 0;
                    let any_visible = level.allies.values().any(|instance_id| {
                        let ally: Gd<Ally> = instance_from_id(*instance_id).unwrap().cast();
                        let ally = ally.bind();
                        visible.contains(&ally.position)
                    });

                    if cooldown_finished && any_visible {
                        for i in 0..self.width as usize {
                            for j in 0..self.height as usize {
                                let position = Position {
                                    x: self.position.x + i,
                                    y: self.position.y + j,
                                };
                                for adjacent in position.adjacent() {
                                    match level.grid[adjacent.x][adjacent.y] {
                                        Tile::Empty | Tile::Item(_) => actions.push((
                                            Some(*ability),
                                            EnemyAction::Spawn {
                                                enemy_kind,
                                                position: adjacent,
                                            },
                                            stats.range,
                                            vec![self.position],
                                        )),
                                        Tile::Ally(_) | Tile::Enemy(_) | Tile::Obstacle(_) => (),
                                    }
                                }
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        if actions.is_empty() {
            (None, None)
        } else {
            actions.sort_by(
                |(_, a_action, a_range, a_path), (_, b_action, b_range, b_path)| match (
                    a_action, b_action,
                ) {
                    (
                        EnemyAction::Attack {
                            ally_id: a_ally_id,
                            damage_kind: a_damage_kind,
                            damage: a_damage,
                        },
                        EnemyAction::Attack {
                            ally_id: b_ally_id,
                            damage_kind: b_damage_kind,
                            damage: b_damage,
                        },
                    ) => {
                        let a_ally: Gd<Ally> =
                            instance_from_id(level.allies[a_ally_id]).unwrap().cast();
                        let a_ally = a_ally.bind();
                        let b_ally: Gd<Ally> =
                            instance_from_id(level.allies[b_ally_id]).unwrap().cast();
                        let b_ally = b_ally.bind();

                        let a_damage = a_damage + damage_bonus(*a_damage_kind, &a_ally.traits);
                        let b_damage = b_damage + damage_bonus(*b_damage_kind, &b_ally.traits);
                        let a_cost = a_path.len() as u16;
                        let b_cost = b_path.len() as u16;
                        let a_within = a_cost <= self.speed;
                        let b_within = b_cost <= self.speed;

                        a_within
                            .cmp(&b_within)
                            .reverse()
                            .then(a_damage.cmp(&b_damage).reverse())
                            .then(a_range.cmp(b_range).reverse())
                            .then(a_cost.cmp(&b_cost))
                    }
                    (EnemyAction::Attack { .. }, EnemyAction::Spawn { .. }) => Ordering::Greater,
                    (EnemyAction::Spawn { .. }, EnemyAction::Attack { .. }) => Ordering::Less,
                    (EnemyAction::Spawn { .. }, EnemyAction::Spawn { .. }) => Ordering::Equal,
                },
            );

            let (ability, action, _, path) = actions.first().unwrap();

            if path.len() as u16 <= self.speed {
                (
                    Some(path.clone()),
                    ability.map(|ability| (ability, *action)),
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
        let stats = abilities().get(&ability).unwrap();
        if stats.consumable {
            let uses = self.uses.get_mut(&ability).unwrap();
            *uses -= 1;

            if *uses == 0 {
                self.uses.remove(&ability);

                let i = self.abilities.iter().position(|a| *a == ability).unwrap();
                self.abilities.remove(i);
            }
        }

        match ability {
            Ability::BatBite
            | Ability::VampireScratch
            | Ability::VampireBite
            | Ability::BigBatBite => match self.position.direction_to(position) {
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
            Ability::SpawnBat => (),
            _ => unreachable!(),
        }
    }

    pub fn heal(&mut self, amount: u16) {
        self.health = cmp::min(self.health + amount, self.max_health);
    }

    pub fn hit(&mut self, damage: u16, damage_kind: DamageKind) {
        if !self.effects.contains_key(&Effect::Mist) {
            let damage = damage + damage_bonus(damage_kind, &self.traits);
            self.health = self.health.checked_sub(damage).unwrap_or(0);

            if damage_kind == DamageKind::Fire {
                match self.effects.get_mut(&Effect::Burn) {
                    Some(stats) => stats.magnitude += 1,
                    None => {
                        self.effects.insert(
                            Effect::Burn,
                            EffectStats {
                                magnitude: 1,
                                duration: 3,
                            },
                        );
                    }
                }
            }

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

    pub fn push(&mut self, level: &mut Level, direction: Direction, distance: u16) {
        let mut position = self.position;
        for dist in 1..=distance {
            let pos = match self.position.in_direction(direction, dist as usize) {
                Some(pos) => pos,
                None => break,
            };

            match level.grid[pos.x][pos.y] {
                Tile::Empty | Tile::Item(_) => position = pos,
                Tile::Ally(_) | Tile::Enemy(_) | Tile::Obstacle(_) => break,
            }
        }

        for i in 0..self.width as usize {
            for j in 0..self.height as usize {
                level.grid[self.position.x + i][self.position.y + j] = Tile::Empty;
                level.grid[position.x + i][position.y + j] = Tile::Enemy(self.id);
            }
        }
        self.position = position;

        let mut tween = self.base_mut().create_tween().unwrap();
        tween.tween_property(
            self.base().clone().upcast(),
            "position".into(),
            Variant::from(position.to_vector()),
            0.3,
        );
    }
}

fn damage_bonus(damage_kind: DamageKind, traits: &[Trait]) -> u16 {
    traits
        .iter()
        .map(|trait_| match (damage_kind, trait_) {
            (DamageKind::Silver, Trait::SilverVulnerable) => 1,
            (DamageKind::Holy, Trait::HolyVulnerable) => 2,
            (DamageKind::Stake, Trait::StakeVulnerable) => 1_000,
            (DamageKind::Sunlight, Trait::SunlightVulnerable) => 1_000,
            (DamageKind::Sunlight, Trait::HolyFromSunlight) => 2,
            _ => 0,
        })
        .sum()
}

pub type ObstacleId = u16;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, GodotConvert, Var, Export)]
#[godot(via = u8)]
pub enum ObstacleKind {
    #[default]
    Wall,
    LowWall,
    Barrel,
}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Obstacle {
    pub id: ObstacleId,
    pub position: Position,
    #[export]
    pub kind: ObstacleKind,
    #[export]
    pub width: u16,
    #[export]
    pub height: u16,
    base: Base<Node2D>,
}

pub type ItemId = u16;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, GodotConvert, Var, Export)]
#[godot(via = u8)]
pub enum ItemKind {
    #[default]
    IronBolt,
    SilverBolt,
    WoodenStake,
    Garlic,
    HolyWater,
}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Item {
    pub id: ItemId,
    pub position: Position,
    #[export]
    pub kind: ItemKind,
    base: Base<Node2D>,
}

impl Item {
    pub fn name(&self) -> String {
        match self.kind {
            ItemKind::IronBolt => "Iron Bolt".into(),
            ItemKind::SilverBolt => "Silver Bolt".into(),
            ItemKind::WoodenStake => "Wooden Stake".into(),
            ItemKind::Garlic => "Garlic".into(),
            ItemKind::HolyWater => "Holy Water".into(),
        }
    }

    pub fn ability(&self) -> Ability {
        match self.kind {
            ItemKind::IronBolt => Ability::CrossbowIronBolt,
            ItemKind::SilverBolt => Ability::CrossbowSilverBolt,
            ItemKind::WoodenStake => Ability::WoodenStake,
            ItemKind::Garlic => Ability::Garlic,
            ItemKind::HolyWater => Ability::HolyWater,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ProjectileKind {
    #[default]
    IronBolt,
    SilverBolt,
    Fireball,
}

#[derive(GodotClass)]
#[class(init, base=Sprite2D)]
pub struct Projectile {
    pub kind: ProjectileKind,
    pub start: Position,
    pub end: Position,
    base: Base<Sprite2D>,
}

#[godot_api]
impl ISprite2D for Projectile {
    fn ready(&mut self) {
        let mut atlas: Gd<AtlasTexture> = self.base().get_texture().unwrap().cast();
        let x = match self.start.direction_to(self.end) {
            Direction::Left => {
                self.base_mut().set_flip_h(true);
                32.0
            }
            Direction::Right => 32.0,
            Direction::Up => 16.0,
            Direction::Down => 0.0,
        };
        let y = match self.kind {
            ProjectileKind::IronBolt => 0.0,
            ProjectileKind::SilverBolt => 16.0,
            ProjectileKind::Fireball => 32.0,
        };
        atlas.set_region(Rect2::new(Vector2::new(x, y), Vector2::new(16.0, 16.0)));

        let start = self.start.to_vector() + Vector2::new(8.0, 8.0);
        let end = self.end.to_vector() + Vector2::new(8.0, 8.0);
        self.base_mut().set_position(start);

        let mut tween = self.base_mut().create_tween().unwrap();
        tween.tween_property(
            self.base().clone().upcast(),
            "position".into(),
            Variant::from(end),
            0.05 * self.start.distance(self.end) as f64,
        );
        tween.tween_callback(Callable::from_object_method(&self.base(), "queue_free"));
    }
}

impl Projectile {
    pub fn new(kind: ProjectileKind, start: Position, end: Position) -> Gd<Self> {
        let scene = load::<PackedScene>("res://scenes/projectile.tscn");
        let mut projectile: Gd<Self> = scene.instantiate().unwrap().cast();

        {
            let mut projectile = projectile.bind_mut();
            projectile.kind = kind;
            projectile.start = start;
            projectile.end = end;
        }

        projectile
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Tile {
    #[default]
    Empty,
    Ally(AllyId),
    Enemy(EnemyId),
    Obstacle(ObstacleId),
    Item(ItemId),
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
    #[export]
    pub room: Room,
    pub grid: [[Tile; LEVEL_HEIGHT]; LEVEL_WIDTH],
    pub turn: Turn,
    pub turn_order: Vec<(EnemyId, u16)>,
    pub spawn_queue: Vec<EnemyId>,
    pub allies: HashMap<AllyId, i64>,
    pub inventory: HashMap<AllyId, Vec<(Ability, u16)>>,
    pub enemy_id: EnemyId,
    pub enemies: HashMap<EnemyId, i64>,
    pub obstacle_id: ObstacleId,
    pub obstacles: HashMap<ObstacleId, i64>,
    pub item_id: ItemId,
    pub items: HashMap<ItemId, i64>,
    pub shadows_cast: bool,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Level {
    fn ready(&mut self) {
        let allies = self.base().get_node_as::<Node2D>("UnitLayer/Allies");
        for child in allies.get_children().iter_shared() {
            let mut ally_node: Gd<Ally> = child.cast();
            let instance_id = ally_node.instance_id();
            let position = Position::from_vector(ally_node.get_position());

            let mut ally = ally_node.bind_mut();
            self.allies.insert(ally.id, instance_id.to_i64());

            ally.position = position;
            self.grid[position.x][position.y] = Tile::Ally(ally.id);

            for (ability, uses) in self.inventory.get(&ally.id).unwrap_or(&Vec::new()) {
                let stats = abilities().get(&ability).unwrap();
                if stats.persistent {
                    if ally.abilities.contains(ability) {
                        ally.uses.insert(*ability, *uses);
                    } else {
                        ally.abilities.push(*ability);
                        ally.uses.insert(*ability, *uses);
                    }
                }
            }

            match ally.id {
                AllyId::AshMagnum => {
                    let mut cursor = self.base().get_node_as::<Cursor>("CursorLayer/Cursor");
                    cursor.set_position(position.to_vector() + Vector2::new(8.0, 8.0));
                    let mut cursor = cursor.bind_mut();
                    cursor.position = position;

                    let id = ally.id;
                    drop(ally);
                    let mut info_panel = self.base().get_node_as::<InfoPanel>("UILayer/InfoPanel");
                    let mut info_panel = info_panel.bind_mut();
                    info_panel.select_ally(id, self);
                }
                AllyId::Alukrod => {
                    drop(ally);
                    if self.room == Room::GreatHall {
                        ally_node
                            .get_node_as::<Sprite2D>("Sprite")
                            .set_visible(false);
                    }
                }
            }
        }
        self.inventory.clear();

        let enemies = self.base().get_node_as::<Node2D>("UnitLayer/Enemies");
        let mut turn_order = Vec::new();
        for child in enemies.get_children().iter_shared() {
            let mut enemy: Gd<Enemy> = child.cast();
            let position = enemy.get_position();
            let position = Position::from_vector(position);
            self.enemies
                .insert(self.enemy_id, enemy.instance_id().to_i64());

            let mut enemy = enemy.bind_mut();
            enemy.position = position;

            for i in 0..enemy.width as usize {
                for j in 0..enemy.height as usize {
                    self.grid[position.x + i][position.y + j] = Tile::Enemy(self.enemy_id);
                }
            }

            turn_order.push((self.enemy_id, enemy.speed));

            enemy.id = self.enemy_id;
            self.enemy_id += 1;
        }

        turn_order.sort_by(|(_, a_speed), (_, b_speed)| a_speed.cmp(b_speed).reverse());
        self.turn_order = turn_order;

        let obstacles = self.base().get_node_as::<CanvasLayer>("ObstacleLayer");
        for child in obstacles.get_children().iter_shared() {
            let mut obstacle: Gd<Obstacle> = child.cast();
            let position = Position::from_vector(obstacle.get_position());
            self.obstacles
                .insert(self.obstacle_id, obstacle.instance_id().to_i64());

            let mut obstacle = obstacle.bind_mut();
            obstacle.position = position;

            for i in 0..obstacle.width as usize {
                for j in 0..obstacle.height as usize {
                    if position.x + i < LEVEL_WIDTH && position.y + j < LEVEL_HEIGHT {
                        self.grid[position.x + i][position.y + j] =
                            Tile::Obstacle(self.obstacle_id);
                    }
                }
            }

            obstacle.id = self.obstacle_id;
            self.obstacle_id += 1;
        }

        let items = self.base().get_node_as::<CanvasLayer>("ItemLayer");
        for child in items.get_children().iter_shared() {
            let mut item: Gd<Item> = child.cast();
            let position = Position::from_vector(item.get_position());
            self.items.insert(self.item_id, item.instance_id().to_i64());

            let mut item = item.bind_mut();
            item.position = position;
            self.grid[position.x][position.y] = Tile::Item(self.item_id);

            item.id = self.item_id;
            self.item_id += 1;
        }

        let mut dialogue = self.base().get_node_as::<Dialogue>("Dialogue");
        let mut dialogue = dialogue.bind_mut();
        dialogue.push_event(DialogueEvent::LevelReady);
    }

    fn process(&mut self, _delta: f64) {
        let dialogue = self.base().get_node_as::<Dialogue>("Dialogue");
        let dialogue = dialogue.bind();

        if !dialogue.active {
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
                    if waiting {
                        if i < self.turn_order.len() {
                            let cursor = self.base().get_node_as::<Cursor>("CursorLayer/Cursor");
                            let mut camera = cursor.get_node_as::<Camera2D>("Camera");

                            let (enemy_id, _) = self.turn_order[i];
                            let enemy = self.get_enemy(enemy_id);

                            camera.set_position_smoothing_enabled(true);
                            camera.set_position_smoothing_speed(8.0);
                            camera.set_position(enemy.get_position() - cursor.get_position());
                        }
                    } else {
                        if i == 0 {
                            let mut cursor =
                                self.base().get_node_as::<Cursor>("CursorLayer/Cursor");
                            let mut cursor = cursor.bind_mut();
                            cursor.acting = false;
                            cursor.selected = None;

                            let path = self.base().get_node_as::<Path>("PathLayer/Path");
                            let path = path.bind();
                            path.clear_path();

                            let mut info_panel =
                                self.base().get_node_as::<InfoPanel>("UILayer/InfoPanel");
                            let mut info_panel = info_panel.bind_mut();
                            info_panel.deselect_tile();
                            info_panel.deselect_ability(self);

                            let mut ability_bar =
                                self.base().get_node_as::<AbilityBar>("UILayer/AbilityBar");
                            let mut ability_bar = ability_bar.bind_mut();
                            ability_bar.select_none();

                            for enemy_id in self.enemies.keys() {
                                let mut enemy = self.get_enemy(*enemy_id);
                                let mut enemy = enemy.bind_mut();
                                for (effect, mut stats) in enemy.effects.clone() {
                                    match effect {
                                        Effect::Burn => {
                                            enemy.hit(stats.magnitude, DamageKind::Normal)
                                        }
                                        _ => (),
                                    }
                                    stats.duration -= 1;
                                    if stats.duration == 0 {
                                        enemy.effects.remove(&effect);
                                    } else {
                                        enemy.effects.insert(effect, stats);
                                    }
                                }
                            }
                        }

                        if i < self.turn_order.len() {
                            let (enemy_id, _) = self.turn_order[i];
                            let mut enemy = self.get_enemy(enemy_id);
                            let mut enemy = enemy.bind_mut();
                            match enemy.animation.as_str() {
                                "side_death" | "front_death" | "back_death" => (),
                                _ => {
                                    let (path, ability) = enemy.plan(self);

                                    if let Some(path) = path {
                                        let position = *path.last().unwrap();

                                        for i in 0..enemy.width as usize {
                                            for j in 0..enemy.height as usize {
                                                self.grid[enemy.position.x + i]
                                                    [enemy.position.y + j] = Tile::Empty;
                                                self.grid[position.x + i][position.y + j] =
                                                    Tile::Enemy(enemy_id);
                                            }
                                        }

                                        enemy.current_ability = ability;
                                        enemy.follow_path(path);

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

                                for (effect, mut stats) in ally.effects.clone() {
                                    match effect {
                                        Effect::Burn => {
                                            ally.hit(stats.magnitude, DamageKind::Normal)
                                        }
                                        _ => (),
                                    }
                                    stats.duration -= 1;
                                    if stats.duration == 0 {
                                        ally.effects.remove(&effect);
                                    } else {
                                        ally.effects.insert(effect, stats);
                                    }
                                }

                                match ally.id {
                                    AllyId::AshMagnum => {
                                        let mut cursor =
                                            self.base().get_node_as::<Cursor>("CursorLayer/Cursor");
                                        cursor.set_position(
                                            ally.position.to_vector() + Vector2::new(8.0, 8.0),
                                        );
                                        let mut cursor = cursor.bind_mut();
                                        cursor.position = ally.position;
                                    }
                                    _ => (),
                                }
                            }

                            let path = self.base().get_node_as::<Path>("PathLayer/Path");
                            let path = path.bind();
                            path.clear_path();

                            let mut camera = self
                                .base()
                                .get_node_as::<Camera2D>("CursorLayer/Cursor/Camera");
                            camera.set_position_smoothing_enabled(false);
                            camera.set_position(Vector2::default());

                            for enemy_id in &self.spawn_queue {
                                let enemy = self.get_enemy(*enemy_id);
                                let enemy = enemy.bind();
                                self.turn_order.push((*enemy_id, enemy.speed));
                            }
                            self.turn_order.sort_by(|(_, a_speed), (_, b_speed)| {
                                a_speed.cmp(b_speed).reverse()
                            });
                            self.spawn_queue.clear();
                        }
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

    pub fn get_item(&self, item_id: ItemId) -> Gd<Item> {
        let instance_id = *self.items.get(&item_id).unwrap();
        instance_from_id(instance_id).unwrap().cast()
    }

    pub fn cast_shadows(&self) {
        let mut visible = HashSet::new();
        for ally_id in self.allies.keys() {
            let ally = self.get_ally(*ally_id);
            let ally = ally.bind();
            visible.extend(compute_fov(ally.position, ally.view_distance, self));
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

        for item_id in self.items.keys() {
            let mut item = self.get_item(*item_id);
            let position = item.bind().position;
            item.set_visible(visible.contains(&position));
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
            match pathfind(
                ally.position,
                position,
                self.grid,
                Tile::Ally(ally.id),
                (1, 1),
            ) {
                Some(path) if !path.is_empty() && path.len() as u16 <= ally.speed => {
                    self.grid[ally.position.x][ally.position.y] = Tile::Empty;
                    ally.follow_path(path);
                    return true;
                }
                _ => (),
            }
        }
        false
    }

    pub fn use_ability(
        &mut self,
        ally_id: AllyId,
        position: Position,
        enemy_id: Option<EnemyId>,
    ) -> bool {
        let mut ally = self.get_ally(ally_id);
        let mut ally = ally.bind_mut();

        if !ally.has_acted && !ally.effects.contains_key(&Effect::Mist) {
            let stats = abilities().get(ally.current_ability()).unwrap();
            match stats.action {
                Action::Attack {
                    damage_kind,
                    damage,
                    ..
                }
                | Action::Push {
                    damage_kind,
                    damage,
                    ..
                } => {
                    if let Some(enemy_id) = enemy_id {
                        let mut enemy_ids = HashSet::new();
                        enemy_ids.insert(enemy_id);

                        match stats.action {
                            Action::Attack { aoe, .. } if aoe => {
                                for position in position.adjacent() {
                                    match self.grid[position.x][position.y] {
                                        Tile::Enemy(id) => {
                                            enemy_ids.insert(id);
                                        }
                                        _ => (),
                                    }
                                }
                            }
                            _ => (),
                        };

                        for enemy_id in enemy_ids {
                            let mut enemy = self.get_enemy(enemy_id);
                            let mut enemy = enemy.bind_mut();
                            for i in 0..enemy.width as usize {
                                for j in 0..enemy.height as usize {
                                    let position = Position {
                                        x: enemy.position.x + i,
                                        y: enemy.position.y + j,
                                    };
                                    match line_to(ally.position, position, self.grid) {
                                        Some(path) if path.len() as u16 <= stats.range => {
                                            if let Some(projectile) = ally.use_ability(position) {
                                                self.base_mut().add_child(projectile.upcast());
                                            }

                                            enemy.hit(damage, damage_kind);
                                            enemy
                                                .last_known_positions
                                                .insert(ally.id, ally.position);

                                            match damage_kind {
                                                DamageKind::LifeSteal => ally.heal(damage),
                                                _ => (),
                                            }

                                            match stats.action {
                                                Action::Push { distance, .. } => {
                                                    let direction =
                                                        ally.position.direction_to(enemy.position);
                                                    enemy.push(self, direction, distance);
                                                }
                                                _ => (),
                                            }

                                            return true;
                                        }
                                        _ => (),
                                    }
                                }
                            }
                        }
                    }
                }
                Action::Effect { effect, stats } => {
                    let position = ally.position;
                    ally.use_ability(position);
                    ally.effects.insert(effect, stats);
                    return true;
                }
                Action::PlaceItem { kind } => match line_to(ally.position, position, self.grid) {
                    Some(path) if path.len() as u16 <= stats.range => {
                        self.spawn_item(kind, position);
                        return true;
                    }
                    _ => (),
                },
                _ => unreachable!(),
            }
        }

        false
    }

    pub fn spawn_enemy(&mut self, enemy_kind: EnemyKind, position: Position) {
        let scene = match enemy_kind {
            EnemyKind::Bat => load::<PackedScene>("res://scenes/enemies/bat.tscn"),
            EnemyKind::Vampire => load::<PackedScene>("res://scenes/enemies/vampire.tscn"),
            EnemyKind::BigBatty => load::<PackedScene>("res://scenes/enemies/big_batty.tscn"),
        };

        let mut enemy: Gd<Enemy> = scene.instantiate().unwrap().cast();
        let instance_id = enemy.instance_id().to_i64();
        enemy.set_position(position.to_vector());

        {
            let mut enemy = enemy.bind_mut();
            enemy.id = self.enemy_id;
            enemy.position = position;

            for i in 0..enemy.width as usize {
                for j in 0..enemy.height as usize {
                    self.grid[position.x + i][position.y + j] = Tile::Enemy(self.enemy_id);
                }
            }
        }

        self.spawn_queue.push(self.enemy_id);
        self.enemies.insert(self.enemy_id, instance_id);
        self.enemy_id += 1;

        let mut enemies = self.base().get_node_as::<Node2D>("UnitLayer/Enemies");
        enemies.add_child(enemy.upcast());
    }

    pub fn spawn_item(&mut self, item_kind: ItemKind, position: Position) {
        let scene = match item_kind {
            ItemKind::Garlic => load::<PackedScene>("res://scenes/items/garlic.tscn"),
            _ => unreachable!(),
        };

        let mut item: Gd<Item> = scene.instantiate().unwrap().cast();
        let instance_id = item.instance_id().to_i64();
        item.set_position(position.to_vector());

        {
            let mut item = item.bind_mut();
            item.id = self.item_id;
            item.position = position;
        }

        self.grid[position.x][position.y] = Tile::Item(self.item_id);
        self.items.insert(self.enemy_id, instance_id);
        self.item_id += 1;

        let mut layer = self.base().get_node_as::<CanvasLayer>("ItemLayer");
        layer.add_child(item.upcast());
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

        let dialogue = self.base().get_node_as::<Dialogue>("../../Dialogue");
        let dialogue = dialogue.bind();

        let mut ability_bar = self
            .base()
            .get_node_as::<AbilityBar>("../../UILayer/AbilityBar");
        let mut ability_bar = ability_bar.bind_mut();

        if self.can_interact
            && level.turn == Turn::Ally
            && !dialogue.active
            && ability_bar.hovered.is_none()
        {
            let input = Input::singleton();

            let shadow_map = self
                .base()
                .get_node_as::<ShadowMap>("../../ShadowLayer/ShadowMap");
            let shadow_map = shadow_map.bind();

            let mut position = self.base().get_position();
            let last_position = self.position;
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
                    Tile::Empty | Tile::Item(_) => {
                        if let Some(selected) = self.selected {
                            if self.acting {
                                if level.use_ability(selected, self.position, None) {
                                    path_node.clear_path();
                                    self.can_interact = false;
                                    self.acting = false;

                                    let mut info_panel = self
                                        .base()
                                        .get_node_as::<InfoPanel>("../../UILayer/InfoPanel");
                                    let mut info_panel = info_panel.bind_mut();
                                    info_panel.deselect_tile();
                                }
                            } else {
                                if level.move_ally(selected, self.position) {
                                    path_node.clear_path();
                                    self.can_interact = false;
                                }
                            }
                        }
                    }
                    Tile::Ally(id) => match self.selected {
                        Some(selected) if selected == id => {
                            if level.use_ability(selected, self.position, None) {
                                path_node.clear_path();
                                self.can_interact = false;
                                self.acting = false;

                                let mut info_panel = self
                                    .base()
                                    .get_node_as::<InfoPanel>("../../UILayer/InfoPanel");
                                let mut info_panel = info_panel.bind_mut();
                                info_panel.deselect_tile();
                            }
                        }
                        _ => {
                            let ally = level.get_ally(id);
                            let ally = ally.bind();

                            if !ally.has_acted {
                                self.acting = ally.has_moved;

                                self.selected = Some(id);
                                ability_bar.select_ally(&ally);
                            }
                        }
                    },
                    Tile::Enemy(id) if self.acting => {
                        if let Some(selected) = self.selected {
                            if level.use_ability(selected, self.position, Some(id)) {
                                path_node.clear_path();
                                self.can_interact = false;
                                self.acting = false;

                                let mut info_panel = self
                                    .base()
                                    .get_node_as::<InfoPanel>("../../UILayer/InfoPanel");
                                let mut info_panel = info_panel.bind_mut();
                                info_panel.deselect_tile();
                            }
                        }
                    }
                    _ => (),
                }
            } else {
                match level.at(self.position) {
                    Tile::Empty | Tile::Item(_) => {
                        if let Some(selected) = self.selected {
                            let ally = level.get_ally(selected);
                            let ally = ally.bind();

                            if self.position != ally.position {
                                if self.acting {
                                    let stats = abilities().get(ally.current_ability()).unwrap();
                                    match stats.action {
                                        Action::PlaceItem { .. } => {
                                            match line_to(ally.position, self.position, level.grid)
                                            {
                                                Some(path) if path.len() as u16 <= stats.range => {
                                                    path_node.set_path(path, PathKind::Attack);
                                                }
                                                _ => path_node.set_path(
                                                    vec![self.position],
                                                    PathKind::Attack,
                                                ),
                                            }
                                        }
                                        _ => path_node
                                            .set_path(vec![self.position], PathKind::Attack),
                                    }
                                } else {
                                    match pathfind(
                                        ally.position,
                                        self.position,
                                        level.grid,
                                        Tile::Ally(ally.id),
                                        (1, 1),
                                    ) {
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
                            match stats.action {
                                Action::Attack { .. } | Action::Push { .. } => {
                                    match line_to(ally.position, self.position, level.grid) {
                                        Some(path) if path.len() as u16 <= stats.range => {
                                            path_node.set_path(path, PathKind::Attack);
                                        }
                                        _ => path_node
                                            .set_path(vec![self.position], PathKind::Attack),
                                    }
                                }
                                _ => path_node.set_path(vec![self.position], PathKind::Attack),
                            }
                        }
                    }
                    _ => path_node.clear_path(),
                }

                if last_position != self.position {
                    let mut info_panel = self
                        .base()
                        .get_node_as::<InfoPanel>("../../UILayer/InfoPanel");
                    let mut info_panel = info_panel.bind_mut();

                    match level.at(self.position) {
                        Tile::Empty | Tile::Obstacle(_) => info_panel.deselect_tile(),
                        Tile::Ally(ally_id) => info_panel.select_ally(ally_id, &level),
                        Tile::Enemy(enemy_id) => info_panel.select_enemy(enemy_id, &level),
                        Tile::Item(item_id) => info_panel.select_item(item_id, &level),
                    }
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

#[derive(Debug, Clone, Copy, PartialEq)]
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
        for position in &path {
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
