use crate::{
    common::{Collider, GameResource, Obstacle},
    map::Map,
};
use bevy::{
    math::{
        bounding::{BoundingCircle, IntersectsVolume},
        vec2,
    },
    prelude::*,
};
use core::f32;
use rand::Rng;

pub struct RobotPlugin;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_robots);
        app.add_systems(
            Update,
            (explore, check_collisions, collect_resource, sense_resource),
        );
    }
}

#[derive(Component)]
pub struct Robot {
    pub direction: f32, // Direction en radians
    pub radius: f32,
    pub speed: f32,
    pub max_turn_rate: f32,
}

#[derive(Component)]
pub struct Sensor {
    pub range: u32,
}

#[derive(Resource)]
pub struct SensorMaterial {
    on: Handle<ColorMaterial>,
    detected: Handle<ColorMaterial>,
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
