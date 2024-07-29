use crate::level::{AllyId, EnemyKind, Level};

use godot::engine::Sprite2D;
use godot::prelude::*;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Var, Export, GodotConvert)]
#[godot(via = u8)]
pub enum Room {
    #[default]
    EntranceHall,
    GreatHall,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DialogueEvent {
    LevelReady,
    EnemyMoved(EnemyKind),
    EnemyKilled(EnemyKind),
}

pub fn trigger_lists() -> &'static HashMap<Room, Vec<(Vec<DialogueEvent>, String)>> {
    static TRIGGER_LISTS: OnceLock<HashMap<Room, Vec<(Vec<DialogueEvent>, String)>>> =
        OnceLock::new();
    TRIGGER_LISTS.get_or_init(|| init_trigger_lists())
}

fn init_trigger_lists() -> HashMap<Room, Vec<(Vec<DialogueEvent>, String)>> {
    [
        (
            Room::EntranceHall,
            vec![
                (
                    vec![DialogueEvent::LevelReady],
                    "entrance-hall-movement-manual".into(),
                ),
                (
                    vec![DialogueEvent::EnemyMoved(EnemyKind::Bat)],
                    "entrance-hall-attack-manual".into(),
                ),
                (
                    vec![DialogueEvent::EnemyKilled(EnemyKind::Bat)],
                    "entrance-hall-defeat-bat".into(),
                ),
                (
                    vec![DialogueEvent::EnemyMoved(EnemyKind::Vampire)],
                    "entrance-hall-vampire-appears".into(),
                ),
                (
                    vec![DialogueEvent::EnemyMoved(EnemyKind::BigBatty)],
                    "entrance-hall-big-batty".into(),
                ),
                (
                    vec![DialogueEvent::EnemyKilled(EnemyKind::BigBatty)],
                    "entrance-hall-big-batty-death".into(),
                ),
            ],
        ),
        (
            Room::GreatHall,
            // vec![(
            //     vec![DialogueEvent::LevelReady],
            //     "great-hall-alukrod-intro".into(),
            // )],
            vec![],
        ),
    ]
    .into()
}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Dialogue {
    #[export]
    pub room: Room,
    pub active: bool,
    pub events: Vec<DialogueEvent>,
    pub triggers: Vec<(Vec<DialogueEvent>, String)>,
    pub current_timeline: String,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Dialogue {
    fn ready(&mut self) {
        let mut dialogic = self.base().get_node_as::<Node>("../../Dialogic");
        dialogic.connect(
            "timeline_started".into(),
            Callable::from_object_method(&self.base(), "on_started"),
        );
        dialogic.connect(
            "timeline_ended".into(),
            Callable::from_object_method(&self.base(), "on_ended"),
        );

        self.triggers = trigger_lists().get(&self.room).unwrap().clone();
    }

    fn process(&mut self, _delta: f64) {
        if let Some(trigger) = self.next_trigger() {
            for event in self.events.clone() {
                if event == trigger {
                    if self.triggered() {
                        let timeline = self.next_timeline();
                        let mut dialogic = self.base().get_node_as::<Node>("../../Dialogic");
                        dialogic.call_deferred("start".into(), &[Variant::from(timeline)]);
                        self.current_timeline = timeline.into();
                    }

                    self.next();
                }
            }
            self.events.clear();
        }
    }
}

#[godot_api]
impl Dialogue {
    #[func]
    pub fn on_started(&mut self) {
        self.active = true;
    }

    #[func]
    pub fn on_ended(&mut self) {
        self.active = false;

        match self.current_timeline.as_str() {
            "great-hall-alukrod-intro" => {
                let level = self.base().get_node_as::<Level>("..");
                let level = level.bind();
                let mut ally = level.get_ally(AllyId::Alukrod);
                ally.get_node_as::<Sprite2D>("Sprite").set_visible(true);
            }
            _ => (),
        }
    }
}

impl Dialogue {
    pub fn next_trigger(&self) -> Option<DialogueEvent> {
        self.triggers.get(0).map(|triggers| triggers.0[0])
    }

    pub fn next_timeline(&self) -> &str {
        &self.triggers[0].1
    }

    pub fn triggered(&self) -> bool {
        self.triggers[0].0.len() == 1
    }

    pub fn next(&mut self) {
        if self.triggered() {
            self.triggers.remove(0);
        } else {
            self.triggers[0].0.remove(0);
        }
    }

    pub fn push_event(&mut self, event: DialogueEvent) {
        self.events.push(event);
    }
}
