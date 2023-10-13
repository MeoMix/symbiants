use bevy::prelude::*;

use crate::ui::selection_menu::Selected;

#[derive(Component)]
pub struct SelectionSprite;

pub fn on_add_selected(
    newly_selected_entity_query: Query<Entity, Added<Selected>>,
    selection_sprite_query: Query<(Entity, &SelectionSprite)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if let Ok(newly_selected_entity) = newly_selected_entity_query.get_single() {
        // JIT cleanup existing sprite because waiting until PostUpdate to access RemovedComponents results in UI flicker
        if let Ok((selection_sprite_entity, _)) = selection_sprite_query.get_single() {
            // Surprisingly, Bevy doesn't fix parent/child relationship when despawning children, so do it manually.
            commands.entity(selection_sprite_entity).remove_parent();
            commands.entity(selection_sprite_entity).despawn();
        }

        // Insert a NodeBundle that is transparent, sized to fit the cell, and has a white border.
        commands
            .entity(newly_selected_entity)
            .with_children(|parent| {
                parent.spawn((
                    SpriteBundle {
                        texture: asset_server.load("images/selection.png"),
                        sprite: Sprite {
                            custom_size: Some(Vec2::splat(1.0)),
                            ..default()
                        },
                        ..default()
                    },
                    SelectionSprite,
                ));
            });
    }
}

// TODO: I would prefer this to use this system, but I need to run it from within FixedUpdate and Bevy doesn't support RemovedComponents from within FixedUpdate.
// pub fn on_selected_removed(
//     mut removed: RemovedComponents<Selected>,
//     selection_sprite_query: Query<(Entity, &SelectionSprite)>,
//     mut commands: Commands,
// ) {
//     for entity in &mut removed {
//         let (selection_sprite_entity, _) = selection_sprite_query
//             .iter()
//             .find(|(_, selection_sprite)| selection_sprite.parent_entity == entity)
//             .unwrap();

//         // Surprisingly, Bevy doesn't fix parent/child relationship when despawning children, so do it manually.
//         commands.entity(selection_sprite_entity).remove_parent();
//         commands.entity(selection_sprite_entity).despawn();
//     }
// }
