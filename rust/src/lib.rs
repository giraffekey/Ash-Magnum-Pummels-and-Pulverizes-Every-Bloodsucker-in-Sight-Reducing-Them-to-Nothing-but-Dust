use godot::prelude::*;

mod ability;
mod dialogue;
mod level;
mod math;
mod traits;
mod ui;

struct GameExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GameExtension {}
