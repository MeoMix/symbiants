use bevy::{prelude::*, sprite::Anchor};

use super::{Root, WorldState};

fn create_air_sprite(width: f32, height: f32, y_offset: f32) -> SpriteBundle {
    SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0.0, y_offset, 0.0),
            ..default()
        },
        sprite: Sprite {
            color: Color::rgb(0.529, 0.808, 0.922),
            custom_size: Some(Vec2::new(width, height)),
            anchor: Anchor::TopLeft,
            ..default()
        },
        ..default()
    }
}

fn create_tunnel_sprite(width: f32, height: f32, y_offset: f32) -> SpriteBundle {
    SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0.0, y_offset, 0.0),
            ..default()
        },
        sprite: Sprite {
            color: Color::rgb(0.373, 0.290, 0.165),
            custom_size: Some(Vec2::new(width, height)),
            anchor: Anchor::TopLeft,
            ..default()
        },
        ..default()
    }
}

pub struct BackgroundPlugin;

// Spawn non-interactive background (sky blue / tunnel brown)
fn setup(mut commands: Commands, query: Query<Entity, With<Root>>, world_state: Res<WorldState>) {
    let Some(mut entity_commands) = commands.get_entity(query.single()) else { panic!("root missing") };

    entity_commands.with_children(|parent| {
        parent.spawn(create_air_sprite(
            world_state.width as f32,
            world_state.surface_level as f32 + 1.0,
            0.0,
        ));

        parent.spawn(create_tunnel_sprite(
            world_state.width as f32,
            world_state.height as f32 - (world_state.surface_level as f32 + 1.0),
            -(world_state.surface_level as f32 + 1.0),
        ));
    });
}

// TODO: This is probably too aggressive of a plugin architecture, but it's good for practice
impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}
