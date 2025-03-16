use bevy::{math::ivec3, prelude::*};
use bevy_simple_tilemap::{plugin::SimpleTileMapPlugin, Tile, TileMap};
use noise::{NoiseFn, Perlin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SimpleTileMapPlugin)
        .add_systems(Startup, (setup, draw_map).chain())
        .add_systems(Update, pan_view)
        .run();
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
