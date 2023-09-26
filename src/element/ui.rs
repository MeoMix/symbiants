use super::Element;
use crate::{
    grid::{position::Position, WorldMap},
    time::IsFastForwarding,
};
use bevy::prelude::*;

pub fn get_element_sprite(element: &Element) -> Sprite {
    let color = match element {
        // Air is transparent - reveals background color such as tunnel or sky
        Element::Air => Color::rgba(0.0, 0.0, 0.0, 0.0),
        Element::Dirt => Color::rgb(0.514, 0.396, 0.224),
        Element::Sand => Color::rgb(0.761, 0.698, 0.502),
        Element::Food => Color::rgb(0.388, 0.584, 0.294),
    };

    Sprite { color, ..default() }
}

pub fn on_spawn_element(
    mut commands: Commands,
    elements: Query<(Entity, &Position, &Element), Added<Element>>,
    world_map: Res<WorldMap>,
) {
    for (entity, position, element) in &elements {
        commands.entity(entity).insert(SpriteBundle {
            transform: Transform::from_translation(position.as_world_position(&world_map)),
            sprite: get_element_sprite(element),
            ..default()
        });
    }
}

pub fn on_update_element_position(
    mut element_query: Query<(Ref<Position>, &mut Transform), With<Element>>,
    is_fast_forwarding: Res<IsFastForwarding>,
    world_map: Res<WorldMap>,
) {
    if is_fast_forwarding.0 {
        return;
    }

    for (position, mut transform) in element_query.iter_mut() {
        if is_fast_forwarding.is_changed() || position.is_changed() {
            transform.translation = position.as_world_position(&world_map);
        }
    }
}
