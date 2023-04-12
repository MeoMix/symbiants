use bevy::prelude::*;

use crate::ant::{AntAngle, AntFacing, LabelContainer};

use super::map::Position;

pub fn get_xflip(facing: AntFacing) -> f32 {
    if facing == AntFacing::Left {
        -1.0
    } else {
        1.0
    }
}

fn render_translation(
    mut query: Query<
        (&mut Transform, &Position, Option<&Parent>),
        (Changed<Position>, Without<LabelContainer>),
    >,
    mut label_container_query: Query<(&mut Transform, &Parent), With<LabelContainer>>,
) {
    for (mut transform, &position, parent) in query.iter_mut() {
        transform.translation.x = position.x as f32;
        transform.translation.y = -position.y as f32;

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

fn render_scale(mut query: Query<(&mut Transform, &AntFacing), Changed<AntFacing>>) {
    for (mut transform, &facing) in query.iter_mut() {
        transform.scale = Vec3::new(get_xflip(facing), 1.0, 1.0);
    }
}

fn render_rotation(mut query: Query<(&mut Transform, &AntAngle, &AntFacing), Changed<AntFacing>>) {
    for (mut transform, &angle, &facing) in query.iter_mut() {
        let angle_radians = angle as u32 as f32 * std::f32::consts::PI / 180.0 * get_xflip(facing);
        transform.rotation = Quat::from_rotation_z(angle_radians);
    }
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(render_translation.in_schedule(CoreSchedule::FixedUpdate));
        app.add_system(render_scale.in_schedule(CoreSchedule::FixedUpdate));
        app.add_system(render_rotation.in_schedule(CoreSchedule::FixedUpdate));
    }
}
