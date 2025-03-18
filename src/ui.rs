use crate::common::Score;
use crate::map::Map;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, (pan_view, update));
    }
}

#[derive(Component)]
struct ScoreDisplay;

fn setup(mut commands: Commands) {
    commands.spawn((
        ScoreDisplay,
        Text::new(format!("Score: {}", 0)),
        TextFont {
            font_size: 24.0,
            ..Default::default()
        },
    ));
}

fn update(score: Res<Score>, mut score_display: Single<&mut Text, With<ScoreDisplay>>) {
    score_display.0 = format!("Score: {}", score.0);
}

fn pan_view(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    map: Single<&Map>,
    window: Single<&Window>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mut transform = camera_query.single_mut();
    const PAN_SPEED: f32 = 10.0;

    let left_boundary: f32 = window.width() / 2.0;
    let right_boundary: f32 = map.tile_size as f32 * map.width as f32 - window.width();
    let top_boundary: f32 = map.tile_size as f32 * map.height as f32 - window.height();
    let bottom_boundary: f32 = window.height() / 2.0;

    if keys.pressed(KeyCode::ArrowUp) {
        if transform.translation.y < top_boundary {
            transform.translation.y += PAN_SPEED;
        }
    }
    if keys.pressed(KeyCode::ArrowDown) {
        if transform.translation.y > bottom_boundary {
            transform.translation.y -= PAN_SPEED;
        }
    }
    if keys.pressed(KeyCode::ArrowLeft) {
        if transform.translation.x > left_boundary {
            transform.translation.x -= PAN_SPEED;
        }
    }
    if keys.pressed(KeyCode::ArrowRight) {
        if transform.translation.x < right_boundary {
            transform.translation.x += PAN_SPEED;
        }
    }
}
