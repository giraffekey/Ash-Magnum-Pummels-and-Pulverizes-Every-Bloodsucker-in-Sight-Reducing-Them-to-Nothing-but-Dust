use crate::ability::Ability;
use crate::dialogue::Dialogue;
use crate::level::{Ally, AllyId, Level};

use godot::engine::{AtlasTexture, HBoxContainer, IHBoxContainer, TextureRect};
use godot::prelude::*;

const NUM_ICONS: usize = 9;

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
                    }
                }
            }

            match self.hovered {
                Some(i) => {
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
                    }
                }
                None => (),
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
            icon.set_ability(ally.abilities.get(i));

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
            icon.set_ability(None);
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
    pub fn set_ability(&mut self, ability: Option<&Ability>) {
        match ability {
            Some(_) => self.base_mut().set_visible(true),
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
                    Ability::WoodenStake => Vector2::new(72.0, y),
                    _ => unreachable!(),
                };
                atlas.set_region(Rect2::new(position, Vector2::new(24.0, 24.0)));
            }
            None => (),
        }
    }
}
