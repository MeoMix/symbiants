use super::map::Position;
use crate::{
    ant::{AntInventory, AntOrientation, Label, TranslationOffset},
    elements::Element,
    IsFastForwarding,
};
use bevy::prelude::*;
use std::ops::Add;

// NOTE: All of these are able to be ran in parallel, but a lot of them require mutable access to transform which means Bevy
// can't run the systems in parallel. It would be possible to use par_iter_mut() to a query in parallel, but there is performance
// overhead in doing so. So, for now, just run the systems in sequence.

pub fn render_translation(
    mut query: Query<(Ref<Position>, &mut Transform, Option<&TranslationOffset>), Without<Label>>,
    mut label_query: Query<(&mut Transform, Option<&TranslationOffset>, &Label), With<Label>>,
    is_fast_forwarding: Res<IsFastForwarding>,
) {
    if is_fast_forwarding.0 {
        return;
    }

    for (position, mut transform, translation_offset) in query.iter_mut() {
        if is_fast_forwarding.is_changed() || position.is_changed() {
            transform.translation = position
                .as_world_position()
                .add(translation_offset.map_or(Vec3::ZERO, |offset| offset.0));
        }
    }

    // Labels are positioned relative to their linked entity (stored at Label.0) and don't have a position of their own
    for (mut transform, translation_offset, label) in label_query.iter_mut() {
        let (position, _, _) = query.get(label.0).unwrap();

        if is_fast_forwarding.is_changed() || position.is_changed() {
            transform.translation = position
                .as_world_position()
                .add(translation_offset.map_or(Vec3::ZERO, |offset| offset.0));
        }
    }
}

pub fn render_orientation(
    mut query: Query<(&mut Transform, Ref<AntOrientation>)>,
    is_fast_forwarding: Res<IsFastForwarding>,
) {
    if is_fast_forwarding.0 {
        return;
    }

    for (mut transform, orientation) in query.iter_mut() {
        if is_fast_forwarding.is_changed() || orientation.is_changed() {
            transform.scale = orientation.as_world_scale();
            transform.rotation = orientation.as_world_rotation();
        }
    }
}

// TODO: Instead of render_carrying, should call spawn/despawn and place the entity into inventory
// TODO: Consider using #[derive(WorldQuery)]
pub fn render_carrying(
    mut commands: Commands,
    // TODO: consider `iter_descendants` instead of Option<&Children>
    mut query: Query<(Entity, Ref<AntInventory>, Option<&Children>)>,
    elements_query: Query<&Element>,
    is_fast_forwarding: Res<IsFastForwarding>,
) {
    if is_fast_forwarding.0 {
        return;
    }

    for (entity, inventory, children) in query.iter_mut() {
        if is_fast_forwarding.is_changed() || inventory.is_changed() {
            // TODO: could be nice to know previous state to only attempt despawn when changing away from carrying
            // TODO: might *need* to know previous state to avoid unintentionally carrying twice
            if let Some(bundle) = inventory.get_carrying_bundle() {
                commands
                    .entity(entity)
                    .with_children(|ant: &mut ChildBuilder| {
                        ant.spawn(bundle);
                    });
            } else {
                // If ant was carrying food/sand, but has stopped, then remove associated UI element.
                if let Some(children) = children {
                    let element_children = children
                        .iter()
                        .filter(|child| elements_query.get(**child).is_ok())
                        .cloned()
                        .collect::<Vec<_>>();

                    commands
                        .entity(entity)
                        .remove_children(&element_children[..]);
                    for child in element_children {
                        commands.entity(child).despawn();
                    }
                }
            }
        }
    }
}
