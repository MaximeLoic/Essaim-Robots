use bevy::{math::bounding::Aabb2d, prelude::*};

#[derive(Component)]
pub struct Collider {
    pub bounding_box: Aabb2d,
}

#[derive(Component)]
pub struct Obstacle;

#[derive(Component)]
pub struct GameResource;
