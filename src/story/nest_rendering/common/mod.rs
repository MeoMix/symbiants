use bevy::{prelude::*, utils::HashMap};

use crate::story::{grid::Grid, nest_simulation::nest::Nest, common::position::Position};

// TODO: This probably isn't a great home for this. The intent is to mark which of the grid (crater vs nest) is active/shown.
#[derive(Resource, Default)]
pub struct VisibleGrid(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct SelectedEntity(pub Option<Entity>);

#[derive(Component)]
pub struct SelectionSprite;

#[derive(Resource, Default)]
pub struct ModelViewEntityMap(pub HashMap<Entity, Entity>);

/// When Selection is added to a component, decorate that component with a white outline sprite.
pub fn on_update_selected(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    selected_entity: Res<SelectedEntity>,
    entity_position_query: Query<Ref<Position>>,
    selection_sprite_query: Query<Entity, With<SelectionSprite>>,
    nest_query: Query<&Grid, With<Nest>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let grid = match nest_query.get(visible_grid_entity) {
        Ok(grid) => grid,
        Err(_) => return,
    };

    if !selected_entity.is_changed() {
        return;
    }

    if let Ok(selection_sprite_entity) = selection_sprite_query.get_single() {
        commands.entity(selection_sprite_entity).despawn();
    }

    let newly_selected_entity = match selected_entity.0 {
        Some(entity) => entity,
        None => return,
    };

    let position = entity_position_query.get(newly_selected_entity).unwrap();

    let mut world_position = grid.grid_to_world_position(*position);
    // render selection UI above ants
    world_position.z = 3.0;

    // Don't spawn as child of selected view entity for two reasons:
    // 1) Tiles provided by bevy_ecs_tilemap are optimized for performance and don't have SpatialBundle
    // 2) Air isn't rendered (it's invisible) so, even excluding bevy_ecs_tilemap, adjustments would be required to show selection.
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(world_position),
            texture: asset_server.load("images/selection.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::ONE),
                ..default()
            },
            ..default()
        },
        SelectionSprite,
    ));
}

pub fn on_update_selected_position(
    selected_entity: Res<SelectedEntity>,
    entity_position_query: Query<&Position, Changed<Position>>,
    mut selection_sprite_query: Query<&mut Transform, With<SelectionSprite>>,
    nest_query: Query<&Grid, With<Nest>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let grid = match nest_query.get(visible_grid_entity) {
        Ok(grid) => grid,
        Err(_) => return,
    };

    let selected_entity = match selected_entity.0 {
        Some(entity) => entity,
        None => return,
    };

    let selected_entity_position = match entity_position_query.get(selected_entity) {
        Ok(position) => position,
        Err(_) => return,
    };

    // Need to update the transform of the spawned selection sprite.
    let mut transform = match selection_sprite_query.get_single_mut() {
        Ok(transform) => transform,
        Err(_) => return,
    };

    let mut world_position = grid.grid_to_world_position(*selected_entity_position);
    // render selection UI above ants
    world_position.z = 3.0;

    transform.translation = world_position;
}

pub fn setup_common(mut commands: Commands) {
    commands.init_resource::<ModelViewEntityMap>();
    commands.init_resource::<SelectedEntity>();
    commands.init_resource::<VisibleGrid>();
}

pub fn teardown_common(
    selection_sprite_query: Query<Entity, With<SelectionSprite>>,
    mut commands: Commands,
) {
    if let Ok(selection_sprite_entity) = selection_sprite_query.get_single() {
        commands.entity(selection_sprite_entity).despawn();
    }

    commands.remove_resource::<SelectedEntity>();
    commands.remove_resource::<VisibleGrid>();

    // TODO: removing this causes issues because camera Update runs expecting the resource to exist.
    //commands.remove_resource::<ModelViewEntityMap>();
}
