use crate::common::{Collider, GameResource, Obstacle};
use bevy::math::{
    bounding::{Aabb2d, BoundingCircle, IntersectsVolume},
    ivec3, vec2,
};
use bevy::prelude::*;
use bevy_simple_tilemap::{Tile, TileMap};
use noise::{NoiseFn, Perlin};
use rand::Rng;
use std::vec;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup, draw_map, spawn_resources).chain());
    }
}

#[derive(Component)]
pub struct Map {
    pub width: u32,  // Number of tiles in the x-axis
    pub height: u32, // Number of tiles in the y-axis
    pub tile_size: u32,
    pub noise_map: Vec<f64>,
}

impl Map {
    pub fn new(width: u32, height: u32, tile_size: u32) -> Self {
        Self {
            width,
            height,
            tile_size,
            noise_map: vec![0.0; width as usize * height as usize],
        }
    }

    pub fn from_perlin_noise(
        width: u32,
        height: u32,
        tile_size: u32,
        seed: u32,
        scale: f64,
    ) -> Self {
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
