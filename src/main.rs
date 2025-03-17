use bevy::prelude::*;
use bevy_simple_tilemap::plugin::SimpleTileMapPlugin;
use map::MapPlugin;
use robot::RobotPlugin;
use ui::UiPlugin;

mod common;
mod map;
mod robot;
mod ui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            SimpleTileMapPlugin,
            MapPlugin,
            RobotPlugin,
            UiPlugin,
        ))
        .run();
}
