use crate::dialogue::Room;

use godot::engine::CenterContainer;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=CenterContainer)]
pub struct DeathScreen {
    #[export]
    pub room: Room,
    base: Base<CenterContainer>,
}

#[godot_api]
impl DeathScreen {
    #[func]
    fn _on_restart_button_pressed(&self) {
        let scene = match self.room {
            Room::EntranceHall => "res://scenes/levels/1-entrance-hall.tscn",
            Room::GreatHall => "res://scenes/levels/2-great-hall.tscn",
        };
        self.base()
            .get_tree()
            .unwrap()
            .change_scene_to_file(scene.into());
    }
}
