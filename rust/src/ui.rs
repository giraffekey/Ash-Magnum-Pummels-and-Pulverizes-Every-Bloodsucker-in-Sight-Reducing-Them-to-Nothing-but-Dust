use crate::ability::{abilities, Ability, Action, DamageKind};
use crate::dialogue::Dialogue;
use crate::level::{Ally, AllyId, EnemyId, ItemId, ItemKind, Level};
use crate::traits::Trait;

use godot::engine::{AtlasTexture, HBoxContainer, IHBoxContainer, Label, TextureRect};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=TextureRect)]
pub struct InfoPanel {
    pub selected_ally: Option<AllyId>,
    pub selected_enemy: Option<EnemyId>,
    pub selected_item: Option<ItemId>,
    pub selected_ability: Option<Ability>,
    base: Base<TextureRect>,
}

impl InfoPanel {
    pub fn clear_info(&mut self) {
        let mut title = self.base().get_node_as::<Label>("Info/Title");
        title.set_text("".into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats1");
        stats_text.set_text("".into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats2");
        stats_text.set_text("".into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats3");
        stats_text.set_text("".into());

        self.base_mut().set_visible(false);
    }

    pub fn select_ally(&mut self, ally_id: AllyId, level: &Level) {
        let ally = level.get_ally(ally_id);
        let ally = ally.bind();

        let mut title = self.base().get_node_as::<Label>("Info/Title");
        title.set_text(ally.name().into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats1");
        stats_text.set_text(format!("{}/{} health", ally.health, ally.max_health).into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats2");
        stats_text.set_text(format!("{} speed", ally.speed).into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats3");
        let text = ally
            .traits
            .iter()
            .map(|trait_| trait_description(*trait_))
            .collect::<Vec<String>>()
            .join("\n");
        stats_text.set_text(text.into());

        self.base_mut().set_visible(true);
    }

    pub fn select_enemy(&mut self, enemy_id: EnemyId, level: &Level) {
        let enemy = level.get_enemy(enemy_id);
        let enemy = enemy.bind();

        let mut title = self.base().get_node_as::<Label>("Info/Title");
        title.set_text(enemy.name().into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats1");
        stats_text.set_text(format!("{}/{} health", enemy.health, enemy.max_health).into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats2");
        stats_text.set_text(format!("{} speed", enemy.speed).into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats3");
        let text = enemy
            .traits
            .iter()
            .map(|trait_| trait_description(*trait_))
            .collect::<Vec<String>>()
            .join("\n");
        stats_text.set_text(text.into());

        self.base_mut().set_visible(true);
    }

    pub fn select_item(&mut self, item_id: ItemId, level: &Level) {
        let item = level.get_item(item_id);
        let item = item.bind();
        let stats = abilities().get(&item.ability()).unwrap();

        let mut title = self.base().get_node_as::<Label>("Info/Title");
        title.set_text(item.name().into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats1");
        stats_text.set_text(action_description(stats.action).into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats2");
        let text = match item.kind {
            ItemKind::IronBolt | ItemKind::SilverBolt => "Crossbow ammunition".into(),
            _ => format!("{} range", stats.range),
        };
        stats_text.set_text(text.into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats3");
        stats_text.set_text("".into());

        self.base_mut().set_visible(true);
    }

    pub fn deselect_tile(&mut self) {
        self.selected_ally = None;
        self.selected_enemy = None;
        self.selected_item = None;
        self.clear_info();
    }

    pub fn select_ability(&mut self, ability: Ability) {
        let stats = abilities().get(&&ability).unwrap();
        let mut title = self.base().get_node_as::<Label>("Info/Title");
        title.set_text(stats.name.clone().into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats1");
        stats_text.set_text(action_description(stats.action).into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats2");
        stats_text.set_text(format!("{} range", stats.range).into());

        let mut stats_text = self.base().get_node_as::<Label>("Info/Stats3");
        stats_text.set_text("".into());

        self.base_mut().set_visible(true);
    }

    pub fn deselect_ability(&mut self, level: &Level) {
        self.selected_ability = None;

        if let Some(ally_id) = self.selected_ally {
            self.select_ally(ally_id, level);
        } else if let Some(enemy_id) = self.selected_enemy {
            self.select_enemy(enemy_id, level);
        } else if let Some(item_id) = self.selected_item {
            self.select_item(item_id, level);
        } else {
            self.clear_info();
        }
    }
}

fn trait_description(trait_: Trait) -> String {
    match trait_ {
        Trait::Mist => "Mist".into(),
        Trait::SilverVulnerable => "Vulnerable to silver".into(),
        Trait::HolyVulnerable => "Vulnerable to holy".into(),
        Trait::StakeVulnerable => "Vulnerable to stakes".into(),
        Trait::SunlightVulnerable => "Vulnerable to sunlight".into(),
        Trait::HolyFromSunlight => "Sunlight deals holy damage".into(),
    }
}

fn action_description(action: Action) -> String {
    match action {
        Action::Attack {
            damage_kind,
            damage,
        } => match damage_kind {
            DamageKind::Normal => format!("{} damage", damage),
            DamageKind::Silver => format!("{} silver damage", damage),
            DamageKind::Holy => format!("{} holy damage", damage),
            DamageKind::Fire => format!("{} fire damage", damage),
            DamageKind::LifeSteal => format!("{} damage, life steal", damage),
            DamageKind::Stake => "Insta-kill a vampire".into(),
            DamageKind::Sunlight => format!("{} sunlight damage", damage),
        },
        Action::Push {
            damage_kind,
            damage,
            distance,
        } => match damage_kind {
            DamageKind::Normal => format!("{} damage, push {}", damage, distance),
            DamageKind::Silver => format!("{} silver damage, push {}", damage, distance),
            DamageKind::Holy => format!("{} holy damage, push {}", damage, distance),
            DamageKind::Fire => format!("{} fire damage, push {}", damage, distance),
            DamageKind::LifeSteal => format!("{} damage, life steal, push {}", damage, distance),
            DamageKind::Stake => format!("Insta-kill a vampire, push {}", distance),
            DamageKind::Sunlight => format!("{} sunlight damage, push {}", damage, distance),
        },
        Action::Activate { trait_ } => match trait_ {
            Trait::Mist => "Transform into mist".into(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

const NUM_ICONS: usize = 8;

#[derive(GodotClass)]
#[class(init, base=HBoxContainer)]
pub struct AbilityBar {
    pub selected: Option<AllyId>,
    pub length: usize,
    pub hovered: Option<usize>,
    base: Base<HBoxContainer>,
}

#[godot_api]
impl IHBoxContainer for AbilityBar {
    fn process(&mut self, _delta: f64) {
        let dialogue = self.base().get_node_as::<Dialogue>("../../Dialogue");
        let dialogue = dialogue.bind();

        if dialogue.active {
            return;
        }

        if let Some(selected) = self.selected {
            let input = Input::singleton();

            let level = self.base().get_node_as::<Level>("../..");
            let level = level.bind();
            let mut ally = level.get_ally(selected);
            let mut ally = ally.bind_mut();

            let mut info_panel = self.base().get_node_as::<InfoPanel>("../InfoPanel");
            let mut info_panel = info_panel.bind_mut();

            let toggled = input.is_action_just_pressed("choose".into())
                || input.is_action_just_pressed("select".into()) && self.hovered.is_some();
            if toggled {
                match self.hovered {
                    Some(i) => {
                        self.hovered = None;

                        let mut icon = self
                            .base()
                            .get_node_as::<AbilityIcon>(format!("AbilityIcon{}", i));
                        let mut icon = icon.bind_mut();
                        icon.set_selected(true);
                        icon.set_hovered(false);

                        info_panel.deselect_ability(&level);
                    }
                    None => {
                        self.hovered = Some(ally.selected_ability);

                        let mut icon = self.base().get_node_as::<AbilityIcon>(format!(
                            "AbilityIcon{}",
                            ally.selected_ability
                        ));
                        let mut icon = icon.bind_mut();
                        icon.set_selected(false);
                        icon.set_hovered(true);

                        info_panel.select_ability(*ally.current_ability());
                    }
                }
            }

            if let Some(i) = self.hovered {
                if input.is_action_just_pressed("left".into()) {
                    let mut icon = self
                        .base()
                        .get_node_as::<AbilityIcon>(format!("AbilityIcon{}", i));
                    let mut icon = icon.bind_mut();
                    icon.set_hovered(false);

                    let i = if i > 0 { i - 1 } else { self.length - 1 };

                    let mut icon = self
                        .base()
                        .get_node_as::<AbilityIcon>(format!("AbilityIcon{}", i));
                    let mut icon = icon.bind_mut();
                    icon.set_hovered(true);

                    ally.selected_ability = i;
                    self.hovered = Some(i);

                    info_panel.select_ability(*ally.current_ability());
                }

                if input.is_action_just_pressed("right".into()) {
                    let mut icon = self
                        .base()
                        .get_node_as::<AbilityIcon>(format!("AbilityIcon{}", i));
                    let mut icon = icon.bind_mut();
                    icon.set_hovered(false);

                    let i = if i < self.length - 1 { i + 1 } else { 0 };

                    let mut icon = self
                        .base()
                        .get_node_as::<AbilityIcon>(format!("AbilityIcon{}", i));
                    let mut icon = icon.bind_mut();
                    icon.set_hovered(true);

                    ally.selected_ability = i;
                    self.hovered = Some(i);

                    info_panel.select_ability(*ally.current_ability());
                }
            }
        }
    }
}

impl AbilityBar {
    pub fn select_ally(&mut self, ally: &Ally) {
        for i in 0..NUM_ICONS {
            let mut icon = self
                .base()
                .get_node_as::<AbilityIcon>(format!("AbilityIcon{}", i));
            let mut icon = icon.bind_mut();
            let ability = ally.abilities.get(i);
            icon.set_ability(
                ability,
                *ability
                    .map(|ability| ally.uses.get(ability).unwrap())
                    .unwrap_or(&0),
            );

            if i == ally.selected_ability {
                icon.set_selected(true);
            }
        }
        self.length = ally.abilities.len();
        self.selected = Some(ally.id);
    }

    pub fn select_none(&mut self) {
        for i in 0..NUM_ICONS {
            let mut icon = self
                .base()
                .get_node_as::<AbilityIcon>(format!("AbilityIcon{}", i));
            let mut icon = icon.bind_mut();
            icon.set_ability(None, 0);
            icon.set_selected(false);
            icon.set_hovered(false);
        }
        self.length = 0;
        self.selected = None;
        self.hovered = None;
    }
}

#[derive(GodotClass)]
#[class(init, base=TextureRect)]
pub struct AbilityIcon {
    pub ability: Option<Ability>,
    pub selected: bool,
    pub hovered: bool,
    base: Base<TextureRect>,
}

impl AbilityIcon {
    pub fn set_ability(&mut self, ability: Option<&Ability>, uses: u16) {
        match ability {
            Some(ability) => {
                self.base_mut().set_visible(true);

                let stats = abilities().get(ability).unwrap();
                let mut amount = self.base().get_node_as::<Label>("Amount");
                amount.set_visible(stats.consumable && uses > 0);
                amount.set_text(uses.to_string().into());
            }
            None => self.base_mut().set_visible(false),
        }
        self.ability = ability.cloned();
        self.set_region();
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
        self.set_region();
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.hovered = hovered;
        self.set_region();
    }

    fn set_region(&mut self) {
        match &self.ability {
            Some(ability) => {
                let mut atlas: Gd<AtlasTexture> = self.base().get_texture().unwrap().cast();
                let y = if self.hovered {
                    48.0
                } else if self.selected {
                    24.0
                } else {
                    0.0
                };
                let position = match ability {
                    Ability::Whip => Vector2::new(0.0, y),
                    Ability::CrossbowIronBolt => Vector2::new(24.0, y),
                    Ability::CrossbowSilverBolt => Vector2::new(48.0, y),
                    Ability::Thwack => Vector2::new(72.0, y),
                    Ability::Sword => Vector2::new(96.0, y),
                    Ability::Hellfire => Vector2::new(120.0, y),
                    Ability::VampireBite => Vector2::new(144.0, y),
                    Ability::Mist => Vector2::new(168.0, y),
                    Ability::WoodenStake => Vector2::new(192.0, y),
                    _ => unreachable!(),
                };
                atlas.set_region(Rect2::new(position, Vector2::new(24.0, 24.0)));
            }
            None => (),
        }
    }
}
