use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_tilemap::tiles::TilePos;

use crate::story::simulation::common::{grid::Grid, position::Position, Zone};

// TODO: This probably isn't a great home for this. The intent is to mark which of the grid (crater vs nest) is active/shown.
#[derive(Resource, Default)]
pub struct VisibleGrid(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct SelectedEntity(pub Option<Entity>);

#[derive(Component)]
pub struct SelectionSprite;

#[derive(Resource, Default)]
pub struct ModelViewEntityMap(HashMap<Entity, Entity>);

impl ModelViewEntityMap {
    pub fn insert(&mut self, model_entity: Entity, view_entity: Entity) {
        if self.0.contains_key(&model_entity) {
            panic!(
                "ModelViewEntityMap already contains key: {:?}",
                model_entity
            );
        }

        self.0.insert(model_entity, view_entity);
    }

    pub fn get(&self, model_entity: &Entity) -> Option<&Entity> {
        self.0.get(model_entity)
    }

    pub fn remove(&mut self, model_entity: &Entity) -> Option<Entity> {
        self.0.remove(model_entity)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

pub fn clear_selection(mut selected_entity: ResMut<SelectedEntity>) {
    selected_entity.0 = None;
}

/// When Selection is added to a component, decorate that component with a white outline sprite.
pub fn on_update_selected(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    selected_entity: Res<SelectedEntity>,
    entity_position_query: Query<Ref<Position>>,
    selection_sprite_query: Query<Entity, With<SelectionSprite>>,
    grid_query: Query<&Grid>,
    visible_grid: Res<VisibleGrid>,
) {
    if !selected_entity.is_changed() {
        return;
    }

    if let Ok(selection_sprite_entity) = selection_sprite_query.get_single() {
        commands.entity(selection_sprite_entity).despawn();
    }

    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let grid = match grid_query.get(visible_grid_entity) {
        Ok(grid) => grid,
        Err(_) => return,
    };

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
    grid_query: Query<&Grid>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let grid = match grid_query.get(visible_grid_entity) {
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

pub fn initialize_common_resources(mut commands: Commands) {
    commands.init_resource::<ModelViewEntityMap>();
    commands.init_resource::<SelectedEntity>();
    commands.init_resource::<VisibleGrid>();
}

pub fn remove_common_resources(mut commands: Commands) {
    commands.remove_resource::<SelectedEntity>();
    commands.remove_resource::<VisibleGrid>();

    // TODO: removing this causes issues because camera Update runs expecting the resource to exist.
    //commands.remove_resource::<ModelViewEntityMap>();
}

pub fn despawn_common_entities(
    selection_sprite_query: Query<Entity, With<SelectionSprite>>,
    mut commands: Commands,
) {
    if let Ok(selection_sprite_entity) = selection_sprite_query.get_single() {
        commands.entity(selection_sprite_entity).despawn();
    }
}

pub fn despawn_view<View: Component>(
    view_query: Query<Entity, With<View>>,
    mut commands: Commands,
) {
    for view_entity in view_query.iter() {
        commands.entity(view_entity).despawn_recursive();
    }
}

// TODO: It would be nice to make this template expectation tighter and only apply to entities stored in ModelViewEntityMap.
pub fn despawn_view_by_model<Model: Component>(
    model_query: Query<Entity, With<Model>>,
    mut commands: Commands,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    for model_entity in model_query.iter() {
        if let Some(&view_entity) = model_view_entity_map.get(&model_entity) {
            commands.entity(view_entity).despawn_recursive();
            model_view_entity_map.remove(&model_entity);
        }
    }
}

/// When a model is despawned its corresponding view should be despawned, too.
/// If model is despawned when the Zone it's in isn't shown then there is no view to despawn.
/// Noop instead of skipping running `on_despawn` to ensure `RemovedComponents` doesn't become backlogged.
pub fn on_despawn<Model: Component, Z: Zone>(
    mut removed: RemovedComponents<Model>,
    mut commands: Commands,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
    grid_query: Query<&Grid, With<Z>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if grid_query.get(visible_grid_entity).is_err() {
        return;
    }

    for model_entity in removed.read() {
        if let Some(view_entity) = model_view_entity_map.remove(&model_entity) {
            commands.entity(view_entity).despawn_recursive();
        }
    }
}

// TODO: Better home?
pub fn grid_to_tile_pos(grid: &Grid, position: Position) -> TilePos {
    TilePos {
        x: position.x as u32,
        y: (grid.height() - position.y - 1) as u32,
    }
}
