use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ability {
    Whip,
    CrossbowIronBolt,
    CrossbowSilverBolt,
    WoodenStake,
    BatBite,
    VampireScratch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageKind {
    Normal,
    Silver,
    Holy,
    Fire,
    Stake,
}

#[derive(Debug, Clone)]
pub struct AbilityStats {
    pub name: String,
    pub damage_kind: DamageKind,
    pub damage: u16,
    pub range: u16,
    pub targets_enemy: bool,
    pub consumable: bool,
}

pub fn ability_lists() -> &'static Vec<Vec<(Ability, u16)>> {
    static ABILITY_LISTS: OnceLock<Vec<Vec<(Ability, u16)>>> = OnceLock::new();
    ABILITY_LISTS.get_or_init(|| init_ability_lists())
}

fn init_ability_lists() -> Vec<Vec<(Ability, u16)>> {
    vec![
        vec![
            (Ability::Whip, 1),
            (Ability::CrossbowIronBolt, 5),
            (Ability::CrossbowSilverBolt, 2),
        ],
        vec![(Ability::BatBite, 1)],
        vec![(Ability::VampireScratch, 1)],
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
                name: "Whip".into(),
                damage_kind: DamageKind::Silver,
                damage: 2,
                range: 2,
                targets_enemy: true,
                consumable: false,
            },
        ),
        (
            Ability::CrossbowIronBolt,
            AbilityStats {
                name: "Crossbow (Iron Bolts)".into(),
                damage_kind: DamageKind::Normal,
                damage: 2,
                range: 6,
                targets_enemy: true,
                consumable: true,
            },
        ),
        (
            Ability::CrossbowSilverBolt,
            AbilityStats {
                name: "Crossbow (Silver Bolts)".into(),
                damage_kind: DamageKind::Silver,
                damage: 2,
                range: 6,
                targets_enemy: true,
                consumable: true,
            },
        ),
        (
            Ability::WoodenStake,
            AbilityStats {
                name: "Wooden Stake".into(),
                damage_kind: DamageKind::Stake,
                damage: 1,
                range: 1,
                targets_enemy: true,
                consumable: true,
            },
        ),
        (
            Ability::BatBite,
            AbilityStats {
                name: "Bat Bite".into(),
                damage_kind: DamageKind::Normal,
                damage: 1,
                range: 1,
                targets_enemy: true,
                consumable: false,
            },
        ),
        (
            Ability::VampireScratch,
            AbilityStats {
                name: "Vampire Scratch".into(),
                damage_kind: DamageKind::Normal,
                damage: 2,
                range: 1,
                targets_enemy: true,
                consumable: false,
            },
        ),
    ]
    .into()
}
