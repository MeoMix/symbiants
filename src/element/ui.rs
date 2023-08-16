use super::Element;
use crate::grid::position::Position;
use bevy::{prelude::*, sprite::Anchor};

pub fn get_element_sprite(element: &Element) -> Sprite {
    let color = match element {
        // Air is transparent - reveals background color such as tunnel or sky
        Element::Air => Color::rgba(0.0, 0.0, 0.0, 0.0),
        Element::Dirt => Color::rgb(0.514, 0.396, 0.224),
        Element::Sand => Color::rgb(0.761, 0.698, 0.502),
        Element::Food => Color::rgb(0.388, 0.584, 0.294),
    };

    Sprite {
        color,
        anchor: Anchor::TopLeft,
        ..default()
    }
}

pub fn on_spawn_element(
    mut commands: Commands,
    elements: Query<(Entity, &Position, &Element), Added<Element>>,
) {
    for (entity, position, element) in &elements {
        commands.entity(entity).insert(SpriteBundle {
            transform: Transform::from_translation(position.as_world_position()),
            sprite: get_element_sprite(element),
            ..default()
        });
    }
}
