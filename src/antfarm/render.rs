use bevy::prelude::*;

use super::map::Position;

fn render_system(mut query: Query<(&mut Transform, &Position), Changed<Position>>) {
    for (mut transform, position) in query.iter_mut() {
        transform.translation.x = position.x as f32;
        transform.translation.y = -position.y as f32;
    }
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(render_system.in_schedule(CoreSchedule::FixedUpdate));
    }
}
