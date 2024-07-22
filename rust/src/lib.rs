use godot::prelude::*;

mod level;
mod math;
mod ui;

struct GameExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GameExtension {}
