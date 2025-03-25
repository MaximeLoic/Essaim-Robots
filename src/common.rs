use bevy::{math::bounding::Aabb2d, prelude::*};

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResourceCollectedEvent>();
        app.add_systems(Startup, setup);
        app.add_systems(Update, update_score);
        app.insert_resource(DiscoveredResources(vec![]));
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
pub struct GameResource {
    pub kind: ResourceType,
    pub points: u32,
}
impl GameResource {
    pub fn new(kind: ResourceType) -> Self {
        match kind {
            ResourceType::Energy => Self { kind, points: 10 },
            ResourceType::Mineral => Self { kind, points: 5 },
            ResourceType::Scientific => Self { kind, points: 1 },
        }
    }
}

#[derive(Resource)]
pub struct Score(pub u32);

#[derive(Event)]
pub struct ResourceCollectedEvent {
    pub points: u32,
}

#[derive(Resource)]
pub struct DiscoveredResources(pub Vec<Vec2>);

fn setup(mut commands: Commands) {
    commands.insert_resource(Score(0));
}

fn update_score(mut score: ResMut<Score>, mut events: EventReader<ResourceCollectedEvent>) {
    for e in events.read() {
        score.0 += e.points;
        info!("Resource collected!. Score: {}", score.0);
    }
}
