use bevy::{prelude::*, sprite::Anchor};

use crate::{
    ant::{AntAngle, AntBehavior, AntFacing, LabelContainer, TransformOffset},
    elements::Element,
};

use super::map::Position;

pub fn render_translation(
    mut query: Query<
        (
            &mut Transform,
            &Position,
            Option<&TransformOffset>,
            Option<&Parent>,
        ),
        (Changed<Position>, Without<LabelContainer>),
    >,
    mut label_container_query: Query<(&mut Transform, &Parent), With<LabelContainer>>,
) {
    info!("render_translation");
    for (mut transform, &position, transform_offset, parent) in query.iter_mut() {
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

pub fn render_scale(mut query: Query<(&mut Transform, &AntFacing), Changed<AntFacing>>) {
    info!("render_scale");
    for (mut transform, &facing) in query.iter_mut() {
        let x_flip = if facing == AntFacing::Left { -1.0 } else { 1.0 };
        transform.scale = Vec3::new(x_flip, 1.0, 1.0);
    }
}

pub fn render_rotation(mut query: Query<(&mut Transform, &AntAngle), Changed<AntAngle>>) {
    info!("render_rotation");
    for (mut transform, &angle) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_z(angle.as_radians());
    }
}

pub fn render_carrying(
    mut commands: Commands,
    mut query: Query<(Entity, &Children, &AntBehavior), Changed<AntBehavior>>,
) {
    info!("render_carrying");
    for (entity, children, behavior) in query.iter_mut() {
        // TODO: could be nice to know previous state to only attempt despawn when changing away from carrying
        // TODO: might *need* to know previous state to avoid unintentionally carrying twice
        if *behavior == AntBehavior::Carrying {
            commands
                .entity(entity)
                .with_children(|ant: &mut ChildBuilder| {
                    ant.spawn((
                        SpriteBundle {
                            transform: Transform {
                                translation: Vec3::new(0.5, 0.5, 0.0),
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
