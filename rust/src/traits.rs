use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trait {
    SilverVulnerable,
    HolyVulnerable,
    StakeVulnerable,
    SunlightVulnerable,
}

pub fn trait_lists() -> &'static Vec<Vec<Trait>> {
    static TRAIT_LISTS: OnceLock<Vec<Vec<Trait>>> = OnceLock::new();
    TRAIT_LISTS.get_or_init(|| init_trait_lists())
}

fn init_trait_lists() -> Vec<Vec<Trait>> {
    vec![
        Vec::new(),
        vec![
            Trait::SilverVulnerable,
            Trait::HolyVulnerable,
            Trait::StakeVulnerable,
            Trait::SunlightVulnerable,
        ],
    ]
}
