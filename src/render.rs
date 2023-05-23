use super::map::Position;
use crate::{
    ant::{AntBehavior, AntOrientation, Label, TranslationOffset},
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

pub fn render_carrying(
    mut commands: Commands,
    mut query: Query<(Entity, Ref<AntBehavior>, Option<&Children>)>,
    is_fast_forwarding: Res<IsFastForwarding>,
) {
    if is_fast_forwarding.0 {
        return;
    }

    for (entity, behavior, children) in query.iter_mut() {
        if is_fast_forwarding.is_changed() || behavior.is_changed() {
            // TODO: could be nice to know previous state to only attempt despawn when changing away from carrying
            // TODO: might *need* to know previous state to avoid unintentionally carrying twice
            if let Some(bundle) = behavior.get_carrying_bundle() {
                commands
                    .entity(entity)
                    .with_children(|ant: &mut ChildBuilder| {
                        ant.spawn(bundle);
                    });
            } else {
                // If children exists, remove them.
                if let Some(children) = children {
                    commands.entity(entity).remove_children(children);
                    for child in children {
                        commands.entity(*child).despawn();
                    }
                }
            }
        }
    }
}
