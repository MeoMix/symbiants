use bevy::prelude::*;

use crate::ui::selection_menu::Selected;

#[derive(Component)]
pub struct SelectionSprite {
    pub parent_entity: Entity,
}

/// When Selection is added to a component, decorate that component with a white outline sprite.
pub fn on_add_selected(
    newly_selected_entity_query: Query<Entity, Added<Selected>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if let Ok(newly_selected_entity) = newly_selected_entity_query.get_single() {
        commands
            .entity(newly_selected_entity)
            .with_children(|parent| {
                parent.spawn((
                    SpriteBundle {
                        transform: Transform::from_translation(Vec3::Z),
                        texture: asset_server.load("images/selection.png"),
                        sprite: Sprite {
                            custom_size: Some(Vec2::ONE),
                            ..default()
                        },
                        ..default()
                    },
                    SelectionSprite {
                        parent_entity: newly_selected_entity,
                    },
                ));
            });
    }
}

/// When Selection is removed from a component, find the white outline sprite and remove it.
pub fn on_selected_removed(
    mut removed: RemovedComponents<Selected>,
    selection_sprite_query: Query<(Entity, &SelectionSprite)>,
    mut commands: Commands,
) {
    for entity in &mut removed {
        let (selection_sprite_entity, _) = selection_sprite_query
            .iter()
            .find(|(_, selection_sprite)| selection_sprite.parent_entity == entity)
            .unwrap();

        // Surprisingly, Bevy doesn't fix parent/child relationship when despawning children, so do it manually.
        commands
            .entity(selection_sprite_entity)
            .remove_parent()
            .despawn();
    }
}
