use core::f32;
use std::vec;

use bevy::{
    math::{
        bounding::{Aabb2d, BoundingCircle, IntersectsVolume},
        ivec3, vec2,
    },
    prelude::*,
};
use bevy_simple_tilemap::{plugin::SimpleTileMapPlugin, Tile, TileMap};
use noise::{NoiseFn, Perlin};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleTileMapPlugin)
        .add_systems(
            Startup,
            (setup, draw_map, spawn_resources, spawn_robots).chain(),
        )
        .add_systems(
            Update,
            (
                pan_view,
                explore,
                check_collisions,
                sense_resource,
                collect_resource,
            ),
        )
        .run();
}

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct Collider {
    bounding_box: Aabb2d,
}

#[derive(Resource)]
struct SensorMaterial {
    on: Handle<ColorMaterial>,
    detected: Handle<ColorMaterial>,
}

#[derive(Component)]
struct Robot {
    direction: f32, // Direction en radians
    radius: f32,
    speed: f32,
    max_turn_rate: f32,
}

#[derive(Component)]

struct Sensor {
    range: u32,
}

#[derive(Component)]
struct GameResource;

#[derive(Component)]

struct Map {
    width: u32,  // Number of tiles in the x-axis
    height: u32, // Number of tiles in the y-axis
    tile_size: u32,
    noise_map: Vec<f64>,
}

impl Map {
    fn new(width: u32, height: u32, tile_size: u32) -> Self {
        Self {
            width,
            height,
            tile_size,
            noise_map: vec![0.0; width as usize * height as usize],
        }
    }

    fn from_perlin_noise(width: u32, height: u32, tile_size: u32, seed: u32, scale: f64) -> Self {
        let mut map = Self::new(width, height, tile_size);
        let perlin = Perlin::new(seed);

        for y in 0..height {
            for x in 0..width {
                let nx = (x as f64 / width as f64) * scale;
                let ny = (y as f64 / height as f64) * scale;

                map.noise_map[(y * width + x) as usize] = perlin.get([nx, ny]);
            }
        }

        map.noise_map[0] = 0.0;

        map
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut window: Single<&mut Window>,
) {
    const MAP_WIDTH: u32 = 250;
    const MAP_HEIGHT: u32 = 250;
    const MAP_SCALE: f64 = 25.0;
    const SEED: u32 = 5;
    const TILE_SIZE: u32 = 16;
    const WINDOW_WIDTH: f32 = 1000.0;
    const WINDOW_HEIGHT: f32 = 800.0;

    let map = Map::from_perlin_noise(MAP_WIDTH, MAP_HEIGHT, TILE_SIZE, SEED, MAP_SCALE);
    let texture_handle = asset_server.load::<Image>("tile.png");
    let atlas_layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), 10, 10, None, None);
    let atlas_layout_handle = texture_atlas_layouts.add(atlas_layout);

    window.resolution.set(WINDOW_WIDTH, WINDOW_HEIGHT);
    window.title = String::from("Robots Exploration");
    window.resizable = false;

    commands.spawn((
        Camera2d,
        Transform::from_xyz(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0, 0.0),
    ));
    commands.spawn((map, TileMap::new(texture_handle, atlas_layout_handle)));
}

fn draw_map(map_query: Single<(&Map, &mut TileMap)>, mut commands: Commands) {
    let (map, mut tile_map) = map_query.into_inner();

    for y in 0..map.height {
        for x in 0..map.width {
            let noise_value = map.noise_map[(y * map.width + x) as usize];

            let sprite_index = if noise_value > 0.75 {
                4
            } else if noise_value > 0.65 {
                3
            } else if noise_value > 0.35 {
                2
            } else if noise_value > 0.2 {
                1
            } else {
                0
            };

            if noise_value > 0.2 && noise_value < 0.35 {
                commands.spawn((
                    Collider {
                        bounding_box: Aabb2d::new(
                            vec2(
                                x as f32 * map.tile_size as f32,
                                y as f32 * map.tile_size as f32,
                            ),
                            vec2(map.tile_size as f32 / 2.0, map.tile_size as f32 / 2.0),
                        ),
                    },
                    Obstacle,
                    Transform::from_xyz(
                        map.tile_size as f32 * x as f32,
                        map.tile_size as f32 * y as f32,
                        0.0,
                    ),
                ));
            }

            tile_map.set_tile(
                ivec3(x as i32, y as i32, 0),
                Some(Tile {
                    sprite_index,
                    color: Color::WHITE,
                    ..Default::default()
                }),
            );
        }
    }
}

fn spawn_resources(
    mut commands: Commands,
    map: Single<&Map>,
    collider_query: Query<&Transform, With<Collider>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    const RESOURCE_COUNT: usize = 300;
    const RESOURCE_SIZE: f32 = 8.0;
    let mut rng = rand::thread_rng();

    let map_width = map.width as f32 * map.tile_size as f32;
    let map_height = map.height as f32 * map.tile_size as f32;

    // Collecter toutes les positions des obstacles
    let obstacle_positions: Vec<Vec2> = collider_query
        .iter()
        .map(|transform| Vec2::new(transform.translation.x, transform.translation.y))
        .collect();

    let mut resources_placed = 0;
    let mut attempts = 0;

    while resources_placed < RESOURCE_COUNT && attempts < 1000 {
        attempts += 1;

        // Générer une position aléatoire
        let x = rng.gen_range(RESOURCE_SIZE..map_width - RESOURCE_SIZE);
        let y = rng.gen_range(RESOURCE_SIZE..map_height - RESOURCE_SIZE);

        // Vérifier s'il y a collision avec un obstacle
        let resource_circle = BoundingCircle::new(Vec2::new(x, y), RESOURCE_SIZE);
        let mut collision = false;

        for obstacle_pos in &obstacle_positions {
            let obstacle_box = Aabb2d::new(
                *obstacle_pos,
                Vec2::new(map.tile_size as f32 / 2.0, map.tile_size as f32 / 2.0),
            );

            if resource_circle.intersects(&obstacle_box) {
                collision = true;
                break;
            }
        }

        // Si pas de collision, placer la ressource
        if !collision {
            commands.spawn((
                GameResource,
                Collider {
                    bounding_box: Aabb2d::new(vec2(x, y), vec2(RESOURCE_SIZE, RESOURCE_SIZE)),
                },
                Transform::from_xyz(x, y, 0.5),
                Mesh2d(meshes.add(Circle::new(RESOURCE_SIZE))),
                MeshMaterial2d(materials.add(Color::hsla(36.0, 1.0, 0.5, 1.0))),
            ));

            resources_placed += 1;
        }
    }

    info!(
        "Ressources placées: {}/{}",
        resources_placed, RESOURCE_COUNT
    );
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

fn spawn_robots(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    const ROBOT_RADIUS: f32 = 10.0;

    let shape = Circle::new(ROBOT_RADIUS);
    let sensor_range = 50;

    let sensor_material_resource = SensorMaterial {
        on: materials.add(Color::hsla(207.0, 1.9, 0.5, 0.2)),
        detected: materials.add(Color::hsla(105.0, 0.55, 0.48, 0.2)),
    };

    for _ in 0..25 {
        commands
            .spawn((
                Mesh2d(meshes.add(shape)),
                Transform::from_xyz(0.0, 0.0, 1.0),
                MeshMaterial2d(materials.add(Color::hsla(0.0, 0.0, 0.0, 1.0))),
                Robot {
                    direction: 0.0,
                    radius: ROBOT_RADIUS,
                    speed: 50.0,
                    max_turn_rate: 5.0,
                },
            ))
            .with_child((
                Sensor {
                    range: sensor_range,
                },
                Mesh2d(meshes.add(Circle::new(sensor_range as f32))),
                MeshMaterial2d(sensor_material_resource.on.clone()),
            ));
    }

    commands.insert_resource(sensor_material_resource);
}

fn explore(mut query: Query<(&mut Transform, &mut Robot)>, map: Single<&Map>, time: Res<Time>) {
    let mut rng = rand::thread_rng();

    for (mut transform, mut robot) in query.iter_mut() {
        transform.translation.x += robot.direction.cos() * robot.speed * time.delta_secs();
        transform.translation.y += robot.direction.sin() * robot.speed * time.delta_secs();

        // Modifier légèrement la direction actuelle
        let turn_amount =
            rng.gen_range(-robot.max_turn_rate..robot.max_turn_rate) * time.delta_secs();

        robot.direction += turn_amount;

        // Limites de la carte pour éviter que le robot ne sorte
        let map_width = map.width as f32 * map.tile_size as f32;
        let map_height = map.height as f32 * map.tile_size as f32;

        // Rebondir sur les bords
        if transform.translation.x < robot.radius {
            transform.translation.x = robot.radius;
            robot.direction = std::f32::consts::PI - robot.direction;
        } else if transform.translation.x > map_width - robot.radius {
            transform.translation.x = map_width - robot.radius;
            robot.direction = std::f32::consts::PI - robot.direction;
        }
        if transform.translation.y < robot.radius {
            transform.translation.y = robot.radius;
            robot.direction = -robot.direction;
        } else if transform.translation.y > map_height - robot.radius {
            transform.translation.y = map_height - robot.radius;
            robot.direction = -robot.direction;
        }
    }
}

fn check_collisions(
    mut robots_query: Query<(&mut Transform, &mut Robot)>,
    obstacles_query: Query<&Collider, With<Obstacle>>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();

    for (transform, mut robot) in robots_query.iter_mut() {
        let mut collision_detected = false;

        // Position actuelle
        let current_pos = vec2(transform.translation.x, transform.translation.y);

        // Calculer le vecteur de déplacement basé sur la direction
        let dx = robot.direction.cos() * robot.speed * time.delta_secs();
        let dy = robot.direction.sin() * robot.speed * time.delta_secs();

        // Nouvelle position prévue
        let new_pos = vec2(current_pos.x + dx, current_pos.y + dy);

        // Vérifier les collisions à la nouvelle position
        let robot_bounding_circle = BoundingCircle::new(new_pos, robot.radius);

        for obstacle_collider in &obstacles_query {
            if robot_bounding_circle.intersects(&obstacle_collider.bounding_box) {
                collision_detected = true;
                break;
            }
        }

        if collision_detected {
            // Modifier la direction de manière aléatoire avec un angle plus important
            robot.direction += rng.gen_range((f32::consts::PI * 0.75)..(f32::consts::PI * 1.25));
        }
    }
}

fn collect_resource(
    mut commands: Commands,
    mut resources_query: Query<(Entity, &Collider), With<GameResource>>,
    robots: Query<(&Transform, &Robot)>,
) {
    for (robot_transform, robot) in &robots {
        for (resource_entity, resource_collider) in resources_query.iter_mut() {
            let robot_bounding_circle = BoundingCircle::new(
                vec2(robot_transform.translation.x, robot_transform.translation.y),
                robot.radius,
            );

            if robot_bounding_circle.intersects(&resource_collider.bounding_box) {
                commands.entity(resource_entity).despawn();
            }
        }
    }
}

fn sense_resource(
    mut sensors_query: Query<(&mut Parent, &Sensor, &mut MeshMaterial2d<ColorMaterial>)>,
    mut parent_query: Query<(&mut Transform, &mut Robot)>,
    resources_query: Query<&Transform, (With<GameResource>, Without<Robot>)>,
    time: Res<Time>,
    sensor_material: Res<SensorMaterial>,
) {
    const ROTATION_SPEED: f32 = 2.0;

    for (parent, sensor, mut material) in sensors_query.iter_mut() {
        let parent_result: Result<
            (Mut<'_, Transform>, Mut<'_, Robot>),
            bevy::ecs::query::QueryEntityError<'_>,
        > = parent_query.get_mut(parent.get());

        if let Ok((robot_transform, mut robot)) = parent_result {
            let mut closest_resource: Option<(Transform, f32)> = None;

            // Trouver la ressource la plus proche dans le rayon de détection
            for resource_transform in &resources_query {
                let distance_to_resource = robot_transform
                    .translation
                    .distance(resource_transform.translation);

                if distance_to_resource < sensor.range as f32 {
                    if closest_resource.is_none()
                        || distance_to_resource < closest_resource.unwrap().1
                    {
                        closest_resource = Some((*resource_transform, distance_to_resource));
                    }
                }
            }

            if let Some((resource_transform, _)) = closest_resource {
                println!(
                    "Found resource at {}, {}",
                    resource_transform.translation.x, resource_transform.translation.y
                );

                *material = MeshMaterial2d(sensor_material.detected.clone());

                let dx = resource_transform.translation.x - robot_transform.translation.x;
                let dy = resource_transform.translation.y - robot_transform.translation.y;

                // Calculer la direction vers la ressource (y, x) pour atan2
                let target_direction = dy.atan2(dx);

                // Rotation progressive vers la cible
                let mut angle_diff = target_direction - robot.direction;

                // Normaliser la différence d'angle entre -PI et PI
                while angle_diff > std::f32::consts::PI {
                    angle_diff -= 2.0 * std::f32::consts::PI;
                }
                while angle_diff < -std::f32::consts::PI {
                    angle_diff += 2.0 * std::f32::consts::PI;
                }

                // Appliquer la rotation avec une vitesse limitée
                let rotation_amount = angle_diff.signum()
                    * (angle_diff.abs().min(ROTATION_SPEED * time.delta_secs()));

                robot.direction += rotation_amount;
            } else {
                *material = MeshMaterial2d(sensor_material.on.clone());
            }
        }
    }
}
