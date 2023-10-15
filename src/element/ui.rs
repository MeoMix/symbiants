use super::{Air, Element};
use crate::{
    story_time::StoryPlaybackState,
    world_map::{position::Position, WorldMap}, simulation::SpriteSheets,
};
use bevy::prelude::*;

pub fn get_element_index(element: &Element) -> usize{
    match element {
        // Air is transparent - reveals background color such as tunnel or sky
        Element::Air => panic!("Air element should not be rendered"),
        // TODO: super hardcoded to the order they appear in sheet.png
        Element::Dirt => 0,
        Element::Food => 1,
        Element::Sand => 2,
    }
}

pub fn on_spawn_element(
    mut commands: Commands,
    elements: Query<(Entity, &Position, &Element), (Added<Element>, Without<Air>)>,
    world_map: Res<WorldMap>,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (entity, position, element) in &elements {
        let mut sprite = TextureAtlasSprite::new(get_element_index(element));
        sprite.custom_size = Some(Vec2::splat(1.0));

        commands.entity(entity).insert(SpriteSheetBundle {
            sprite,
            texture_atlas: sprite_sheets.element.clone(),
            transform: Transform::from_translation(position.as_world_position(&world_map)),
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
