use bevy::prelude::*;
use bevy_simple_tilemap::plugin::SimpleTileMapPlugin;
use common::CommonPlugin;
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
            CommonPlugin,
            MapPlugin,
            RobotPlugin,
            UiPlugin,
        ))
        .run();
}
