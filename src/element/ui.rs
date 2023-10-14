use super::{Air, Element};
use crate::{
    story_time::StoryPlaybackState,
    world_map::{position::Position, WorldMap},
};
use bevy::prelude::*;

pub fn get_element_texture(element: &Element, asset_server: &Res<AssetServer>) -> Handle<Image> {
    match element {
        // Air is transparent - reveals background color such as tunnel or sky
        Element::Air => panic!("Air element should not be rendered"),
        Element::Dirt => asset_server.load("images/dirt/dirt.png"),
        Element::Sand => asset_server.load("images/sand/sand.png"),
        Element::Food => asset_server.load("images/food/food.png"),
    }
}

pub fn on_spawn_element(
    mut commands: Commands,
    elements: Query<(Entity, &Position, &Element), (Added<Element>, Without<Air>)>,
    world_map: Res<WorldMap>,
    asset_server: Res<AssetServer>,
) {
    for (entity, position, element) in &elements {
        commands.entity(entity).insert(SpriteBundle {
            texture: get_element_texture(element, &asset_server),
            transform: Transform::from_translation(position.as_world_position(&world_map)),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(1.0)),
                ..default()
            },
            ..default()
        });
    }
}

pub fn on_update_element_position(
    mut element_query: Query<(Ref<Position>, &mut Transform), (With<Element>, Without<Air>)>,
    story_playback_state: Res<State<StoryPlaybackState>>,
    world_map: Res<WorldMap>,
) {
    if story_playback_state.get() == &StoryPlaybackState::FastForwarding {
        return;
    }

    for (position, mut transform) in element_query.iter_mut() {
        if story_playback_state.is_changed() || position.is_changed() {
            transform.translation = position.as_world_position(&world_map);
        }
    }
}
