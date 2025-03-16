use bevy::{
    log::tracing_subscriber::fmt::time,
    math::ivec3,
    prelude::*,
    render::{mesh, render_resource::ShaderType},
};
use bevy_simple_tilemap::{plugin::SimpleTileMapPlugin, Tile, TileMap};
use noise::{NoiseFn, Perlin};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleTileMapPlugin)
        .add_systems(Startup, (setup, draw_map, spawn_robot).chain())
        .add_systems(Update, (pan_view, move_robot))
        .run();
}

#[derive(Component)]
struct Robot {
    direction: f32,  // Direction en radians
    turn_speed: f32, // Vitesse de rotation
}

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
    const SEED: u32 = 1;
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

    commands.spawn(Camera2d);
    commands.spawn((
        TileMap::new(texture_handle, atlas_layout_handle),
        Transform::from_xyz(-1.0 * WINDOW_WIDTH / 2.0, -1.0 * WINDOW_HEIGHT / 2.0, 0.0),
    ));
    commands.spawn(map);
}

fn draw_map(mut tile_map_query: Query<&mut TileMap>, map_query: Query<&Map>) {
    let map = map_query.single();
    let mut tile_map = tile_map_query.single_mut();

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

fn pan_view(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    map: Single<&Map>,
    window: Single<&Window>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mut transform = camera_query.single_mut();
    const PAN_SPEED: f32 = 10.0;
    const BORDER: f32 = 10.0;

    let left_boundary: f32 = 0.0;
    let right_boundary: f32 = map.tile_size as f32 * map.width as f32 - window.width() - BORDER;
    let top_boundary: f32 = map.tile_size as f32 * map.height as f32 - window.height() - BORDER;
    let bottom_boundary: f32 = 0.0;

    println!("Camera: {:?}", transform);
    println!("window_height: {:?}", window.height());

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

fn spawn_robot(
    mut commands: Commands,
    map: Single<&Map>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape = Circle::new(10.0);

    for _ in 0..100 {
        commands.spawn((
            Mesh2d(meshes.add(shape)),
            Transform::from_xyz(0.0, 0.0, 1.0),
            MeshMaterial2d(materials.add(Color::hsla(0.0, 0.0, 0.0, 1.0))),
            Robot {
                direction: 0.0,
                turn_speed: 2.0,
            },
        ));
    }
}

fn move_robot(mut query: Query<(&mut Transform, &mut Robot)>, map: Single<&Map>, time: Res<Time>) {
    const SPEED: f32 = 100.0;
    const MAX_TURN_RATE: f32 = 3.0; // Vitesse maximale de rotation en radians par seconde

    let mut rng = rand::thread_rng();

    for (mut transform, mut robot) in query.iter_mut() {
        // Modifier légèrement la direction actuelle
        let turn_amount = rng.gen_range(-MAX_TURN_RATE..MAX_TURN_RATE) * time.delta_secs();
        robot.direction += turn_amount;

        // Calculer le vecteur de déplacement basé sur la direction
        let dx = robot.direction.cos() * SPEED * time.delta_secs();
        let dy = robot.direction.sin() * SPEED * time.delta_secs();

        // Appliquer le déplacement
        transform.translation.x += dx;
        transform.translation.y += dy;

        // Limites de la carte pour éviter que le robot ne sorte
        let map_width = map.width as f32 * map.tile_size as f32;
        let map_height = map.height as f32 * map.tile_size as f32;
        let radius = 10.0; // Rayon du robot

        // Rebondir sur les bords
        if transform.translation.x < radius {
            transform.translation.x = radius;
            robot.direction = std::f32::consts::PI - robot.direction;
        } else if transform.translation.x > map_width - radius {
            transform.translation.x = map_width - radius;
            robot.direction = std::f32::consts::PI - robot.direction;
        }

        if transform.translation.y < radius {
            transform.translation.y = radius;
            robot.direction = -robot.direction;
        } else if transform.translation.y > map_height - radius {
            transform.translation.y = map_height - radius;
            robot.direction = -robot.direction;
        }

        // Orienter le sprite dans la direction du mouvement (optionnel)
        transform.rotation = Quat::from_rotation_z(robot.direction);
    }
}
