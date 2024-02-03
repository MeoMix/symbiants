pub mod camera;
pub mod pointer;
pub mod selection;
pub mod visible_grid;

use self::{
    camera::RenderingCameraPlugin,
    pointer::{handle_pointer_tap, initialize_pointer_resources, remove_pointer_resources},
    selection::{
        clear_selection, on_update_selected, on_update_selected_position, SelectedEntity,
        SelectionSprite,
    },
    visible_grid::{VisibleGrid, VisibleGridState},
};
use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_tilemap::TilemapPlugin;
use simulation::{
    app_state::AppState,
    common::{grid::Grid, Zone},
    CleanupSet, FinishSetupSet,
};

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
pub struct CommonRenderingPlugin;

/// Systems which apply to both Nest and Crater rendering.
impl Plugin for CommonRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RenderingCameraPlugin, TilemapPlugin));
        app.add_state::<VisibleGridState>();

        app.add_systems(
            OnEnter(AppState::FinishSetup),
            (
                initialize_common_resources,
                (initialize_pointer_resources, apply_deferred).chain(),
            )
                .in_set(FinishSetupSet::BeforeSimulationFinishSetup),
        );

        app.add_systems(
            Update,
            (on_update_selected, on_update_selected_position).run_if(in_state(AppState::TellStory)),
        );

        // IMPORTANT: don't process user input in FixedUpdate/SimulationUpdate because event reads can be missed
        // https://github.com/bevyengine/bevy/issues/7691
        app.add_systems(
            Update,
            (handle_pointer_tap)
                .run_if(in_state(AppState::TellStory))
                .chain(),
        );

        app.add_systems(
            OnExit(VisibleGridState::Nest),
            (clear_selection,).run_if(in_state(AppState::TellStory)),
        );

        app.add_systems(
            OnExit(VisibleGridState::Crater),
            (clear_selection,).run_if(in_state(AppState::TellStory)),
        );

        app.add_systems(
            OnExit(AppState::Cleanup),
            |model_view_entity_map: Res<ModelViewEntityMap>| {
                if model_view_entity_map.len() > 0 {
                    panic!(
                        "ModelViewEntityMap has {} entries remaining after cleanup",
                        model_view_entity_map.len()
                    );
                }
            },
        );

        app.add_systems(
            OnEnter(AppState::Cleanup),
            (
                despawn_common_entities,
                remove_common_resources,
                remove_pointer_resources,
                reset_visible_grid_state,
            )
                .in_set(CleanupSet::BeforeSimulationCleanup),
        );
    }
}

fn initialize_common_resources(mut commands: Commands) {
    commands.init_resource::<ModelViewEntityMap>();
    commands.init_resource::<SelectedEntity>();
    commands.init_resource::<VisibleGrid>();
}

fn remove_common_resources(mut commands: Commands) {
    commands.remove_resource::<SelectedEntity>();
    commands.remove_resource::<VisibleGrid>();
    // TODO: removing this causes issues because camera Update runs expecting the resource to exist.
    //commands.remove_resource::<ModelViewEntityMap>();
}

fn despawn_common_entities(
    selection_sprite_query: Query<Entity, With<SelectionSprite>>,
    mut commands: Commands,
) {
    if let Ok(selection_sprite_entity) = selection_sprite_query.get_single() {
        commands.entity(selection_sprite_entity).despawn();
    }
}

fn reset_visible_grid_state(mut next_visible_grid_state: ResMut<NextState<VisibleGridState>>) {
    next_visible_grid_state.set(VisibleGridState::Nest);
}
