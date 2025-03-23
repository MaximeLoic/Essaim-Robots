use bevy::{math::bounding::Aabb2d, prelude::*};

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResourceCollectedEvent>();
        app.add_systems(Startup, setup);
        app.add_systems(Update, update_score);
    }
}

#[derive(Component)]
pub struct Collider {
    pub bounding_box: Aabb2d,
}

#[derive(Component)]
pub struct Obstacle;

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub enum ResourceType {
    Energy,
    Mineral,
    Scientific,
}
#[derive(Component)]
pub struct GameResource{
    pub kind: ResourceType,
    pub points: u32,
}

#[derive(Resource)]
pub struct Score(pub u32);

#[derive(Event)]
pub struct ResourceCollectedEvent;

fn setup(mut commands: Commands) {
    commands.insert_resource(Score(0));
}

fn update_score(mut score: ResMut<Score>, mut events: EventReader<ResourceCollectedEvent>) {
    for _ in events.read() {
        score.0 += 1;
        info!("Resource collected!. Score: {}", score.0);
    }
}
