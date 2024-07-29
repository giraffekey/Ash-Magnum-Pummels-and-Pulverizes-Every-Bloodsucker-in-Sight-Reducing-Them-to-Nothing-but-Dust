use crate::level::{Effect, EffectStats, EnemyKind};

use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ability {
    Whip,
    CrossbowIronBolt,
    CrossbowSilverBolt,
    Thwack,
    Sword,
    Hellfire,
    VampireBite,
    Mist,
    WoodenStake,
    BatBite,
    VampireScratch,
    BigBatBite,
    SpawnBat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageKind {
    Normal,
    Silver,
    Holy,
    Fire,
    LifeSteal,
    Stake,
    Sunlight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Attack {
        damage_kind: DamageKind,
        damage: u16,
    },
    Push {
        damage_kind: DamageKind,
        damage: u16,
        distance: u16,
    },
    Effect {
        effect: Effect,
        stats: EffectStats,
    },
    Spawn {
        enemy_kind: EnemyKind,
        cooldown: u16,
    },
}

#[derive(Debug, Clone)]
pub struct AbilityStats {
    pub name: String,
    pub action: Action,
    pub range: u16,
    pub acquirable: bool,
    pub consumable: bool,
    pub persistent: bool,
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
            (Ability::Thwack, 2),
        ],
        vec![
            (Ability::Sword, 1),
            (Ability::Hellfire, 3),
            (Ability::VampireBite, 1),
            (Ability::Mist, 1),
        ],
        vec![(Ability::BatBite, 1)],
        vec![(Ability::VampireScratch, 1), (Ability::VampireBite, 1)],
        vec![(Ability::BigBatBite, 1), (Ability::SpawnBat, 1)],
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
                action: Action::Attack {
                    damage_kind: DamageKind::Silver,
                    damage: 2,
                },
                range: 2,
                acquirable: false,
                consumable: false,
                persistent: false,
            },
        ),
        (
            Ability::CrossbowIronBolt,
            AbilityStats {
                name: "Crossbow (Iron Bolts)".into(),
                action: Action::Attack {
                    damage_kind: DamageKind::Normal,
                    damage: 2,
                },
                range: 6,
                acquirable: false,
                consumable: true,
                persistent: true,
            },
        ),
        (
            Ability::CrossbowSilverBolt,
            AbilityStats {
                name: "Crossbow (Silver Bolts)".into(),
                action: Action::Attack {
                    damage_kind: DamageKind::Silver,
                    damage: 2,
                },
                range: 6,
                acquirable: false,
                consumable: true,
                persistent: true,
            },
        ),
        (
            Ability::Thwack,
            AbilityStats {
                name: "Thwack".into(),
                action: Action::Push {
                    damage_kind: DamageKind::Silver,
                    damage: 2,
                    distance: 4,
                },
                range: 2,
                acquirable: false,
                consumable: true,
                persistent: false,
            },
        ),
        (
            Ability::Sword,
            AbilityStats {
                name: "Sword".into(),
                action: Action::Attack {
                    damage_kind: DamageKind::Normal,
                    damage: 2,
                },
                range: 1,
                acquirable: false,
                consumable: false,
                persistent: false,
            },
        ),
        (
            Ability::Hellfire,
            AbilityStats {
                name: "Hellfire".into(),
                action: Action::Attack {
                    damage_kind: DamageKind::Fire,
                    damage: 2,
                },
                range: 6,
                acquirable: false,
                consumable: true,
                persistent: false,
            },
        ),
        (
            Ability::VampireBite,
            AbilityStats {
                name: "Vampire Bite".into(),
                action: Action::Attack {
                    damage_kind: DamageKind::LifeSteal,
                    damage: 1,
                },
                range: 1,
                acquirable: false,
                consumable: false,
                persistent: false,
            },
        ),
        (
            Ability::Mist,
            AbilityStats {
                name: "Mist".into(),
                action: Action::Effect {
                    effect: Effect::Mist,
                    stats: EffectStats {
                        magnitude: 0,
                        duration: 2,
                    },
                },
                range: 0,
                acquirable: false,
                consumable: true,
                persistent: false,
            },
        ),
        (
            Ability::WoodenStake,
            AbilityStats {
                name: "Wooden Stake".into(),
                action: Action::Attack {
                    damage_kind: DamageKind::Stake,
                    damage: 1,
                },
                range: 1,
                acquirable: true,
                consumable: true,
                persistent: true,
            },
        ),
        (
            Ability::BatBite,
            AbilityStats {
                name: "Bat Bite".into(),
                action: Action::Attack {
                    damage_kind: DamageKind::Normal,
                    damage: 1,
                },
                range: 1,
                acquirable: false,
                consumable: false,
                persistent: false,
            },
        ),
        (
            Ability::VampireScratch,
            AbilityStats {
                name: "Vampire Scratch".into(),
                action: Action::Attack {
                    damage_kind: DamageKind::Normal,
                    damage: 2,
                },
                range: 1,
                acquirable: false,
                consumable: false,
                persistent: false,
            },
        ),
        (
            Ability::BigBatBite,
            AbilityStats {
                name: "Big Bat Bite".into(),
                action: Action::Attack {
                    damage_kind: DamageKind::Normal,
                    damage: 2,
                },
                range: 1,
                acquirable: false,
                consumable: false,
                persistent: false,
            },
        ),
        (
            Ability::SpawnBat,
            AbilityStats {
                name: "Spawn Bat".into(),
                action: Action::Spawn {
                    enemy_kind: EnemyKind::Bat,
                    cooldown: 3,
                },
                range: 1,
                acquirable: false,
                consumable: false,
                persistent: false,
            },
        ),
    ]
    .into()
}
