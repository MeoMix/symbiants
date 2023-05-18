use bevy::{prelude::*, sprite::Anchor};

use crate::{
    ant::{AntBehavior, AntOrientation, Facing, LabelContainer, TransformOffset},
    elements::Element,
    IsFastForwarding,
};

use super::map::Position;

// NOTE: All of these are able to be ran in parallel, but a lot of them require mutable access to transform which means Bevy
// can't run the systems in parallel. It would be possible to use par_iter_mut() to a query in parallel, but there is performance
// overhead in doing so. So, for now, just run the systems in sequence.

pub fn render_translation(
    mut query: Query<
        (
            &mut Transform,
            Ref<Position>,
            Option<&TransformOffset>,
            Option<&Parent>,
        ),
        Without<LabelContainer>,
    >,
    mut label_container_query: Query<(&mut Transform, &Parent), With<LabelContainer>>,
    is_fast_forwarding: Res<IsFastForwarding>,
) {
    if is_fast_forwarding.0 {
        return;
    }

    for (mut transform, position, transform_offset, parent) in query.iter_mut() {
        if is_fast_forwarding.is_changed() || position.is_changed() {
            let offset_x = transform_offset.map_or(0.0, |offset| offset.0.x);
            let offset_y = transform_offset.map_or(0.0, |offset| offset.0.y);

            transform.translation.x = position.x as f32 + offset_x;
            transform.translation.y = -position.y as f32 + offset_y;

            // If entity has a parent container, check for sibling labels and update their position.
            if let Some(parent) = parent {
                label_container_query.iter_mut().for_each(
                    |(mut label_container_transform, label_container_parent)| {
                        if label_container_parent == parent {
                            label_container_transform.translation.x = transform.translation.x;
                            label_container_transform.translation.y = transform.translation.y;
                        }
                    },
                );
            }
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
            let x_flip = if orientation.get_facing() == Facing::Left {
                -1.0
            } else {
                1.0
            };
            transform.scale = Vec3::new(x_flip, 1.0, 1.0);
            transform.rotation = Quat::from_rotation_z(orientation.get_angle().as_radians());
        }
    }
}

pub fn render_carrying(
    mut commands: Commands,
    mut query: Query<(Entity, &Children, Ref<AntBehavior>)>,
    is_fast_forwarding: Res<IsFastForwarding>,
) {
    if is_fast_forwarding.0 {
        return;
    }

    for (entity, children, behavior) in query.iter_mut() {
        if is_fast_forwarding.is_changed() || behavior.is_changed() {
            // TODO: could be nice to know previous state to only attempt despawn when changing away from carrying
            // TODO: might *need* to know previous state to avoid unintentionally carrying twice
            if *behavior == AntBehavior::Carrying {
                commands
                    .entity(entity)
                    .with_children(|ant: &mut ChildBuilder| {
                        ant.spawn((
                            SpriteBundle {
                                transform: Transform {
                                    translation: Vec3::new(0.5, 0.5, 1.0),
                                    ..default()
                                },
                                sprite: Sprite {
                                    color: Color::rgb(0.761, 0.698, 0.502),
                                    anchor: Anchor::TopLeft,
                                    ..default()
                                },
                                ..default()
                            },
                            Element::Sand,
                        ));
                    });
            } else {
                commands.entity(entity).remove_children(children);
                for child in children {
                    commands.entity(*child).despawn();
                }
            }
        }
    }
}
