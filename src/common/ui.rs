use bevy::prelude::*;
use std::ops::Add;

use crate::{map::Position, time::IsFastForwarding};

use super::{Label, TranslationOffset};

// Manages rerendering entities when their positions are updated. Applies to ants, labels, and elements.
// TODO: it might make more sense to split this into ant-specific and element-specific functions because ants have labels and offsets
// but elements are dead simple. The current way seemed appealing because there could be lots of entities with positions and it seemed like
// they would all want to reflect their position similarly, but maybe that isn't the case.
pub fn on_update_position(
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