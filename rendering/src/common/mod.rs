pub mod camera;
pub mod element;
pub mod pheromone;
pub mod pointer;
pub mod selection;
pub mod visible_grid;

use self::{
    camera::RenderingCameraPlugin,
    element::{sprite_sheet::start_load_element_sprite_sheet, ElementTilemap},
    pheromone::{
        cleanup_pheromones, initialize_pheromone_resources, on_update_pheromone_visibility,
    },
    pointer::{handle_pointer_tap, initialize_pointer_resources, remove_pointer_resources},
    selection::{
        clear_selection, on_update_selected, on_update_selected_position, SelectedEntity,
        SelectionSprite,
    },
    visible_grid::{set_visible_grid_state_none, VisibleGrid, VisibleGridState},
};
use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_tilemap::TilemapPlugin;
use simulation::{
    app_state::AppState,
    common::{grid::Grid, LoadProgress, SimulationLoadProgress, Zone},
    crater_simulation::crater::AtCrater,
    nest_simulation::nest::AtNest,
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
pub fn despawn_view_by_model<Model: Component, Z: Zone>(
    model_query: Query<Entity, (With<Model>, With<Z>)>,
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

pub fn on_model_removed_zone<Z: Zone>(
    mut removed: RemovedComponents<Z>,
    mut commands: Commands,
    grid_query: Query<&Grid, With<Z>>,
    visible_grid: Res<VisibleGrid>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if grid_query.get(visible_grid_entity).is_err() {
        return;
    }

    for at_zone_entity in removed.read() {
        if let Some(&view_entity) = model_view_entity_map.get(&at_zone_entity) {
            commands.entity(view_entity).despawn_recursive();
            model_view_entity_map.remove(&at_zone_entity);
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

#[derive(Resource, Default, Debug)]
pub struct RenderingLoadProgress {
    // TODO: kind of weird LoadProgress type comes from Simulation rather than being distinct
    pub element_sprite_sheet: LoadProgress,
}

pub fn initialize_loading_resources(mut commands: Commands) {
    commands.init_resource::<RenderingLoadProgress>();
}

pub fn remove_loading_resources(mut commands: Commands) {
    commands.remove_resource::<RenderingLoadProgress>();
}

pub struct CommonRenderingPlugin;

/// Systems which apply to both Nest and Crater rendering.
impl Plugin for CommonRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RenderingCameraPlugin, TilemapPlugin));
        app.init_state::<VisibleGridState>();

        app.add_systems(
            OnEnter(AppState::Loading),
            // TODO: I don't like using `before(fn)` like this
            (initialize_loading_resources, apply_deferred)
                .chain()
                .before(start_load_element_sprite_sheet),
        );

        app.add_systems(
            Update,
            check_load_progress.run_if(in_state(AppState::Loading)),
        );

        app.add_systems(
            OnEnter(AppState::FinishSetup),
            (initialize_common_resources, initialize_pointer_resources)
                .in_set(FinishSetupSet::BeforeSimulationFinishSetup),
        );

        app.add_systems(
            OnEnter(AppState::FinishSetup),
            (initialize_pheromone_resources).in_set(FinishSetupSet::AfterSimulationFinishSetup),
        );

        app.add_systems(
            Update,
            (
                on_update_selected,
                on_update_selected_position,
                on_update_pheromone_visibility,
            )
                .run_if(
                    in_state(AppState::TellStory { ended: false })
                        .or_else(in_state(AppState::PostSetupClearChangeDetection)),
                ),
        );

        // IMPORTANT: don't process user input in FixedUpdate because event reads can be missed
        // https://github.com/bevyengine/bevy/issues/7691
        app.add_systems(
            Update,
            (handle_pointer_tap::<AtNest>, handle_pointer_tap::<AtCrater>)
                .run_if(in_state(AppState::TellStory { ended: false }))
                .chain(),
        );

        app.add_systems(
            OnExit(VisibleGridState::Nest),
            (clear_selection,).run_if(in_state(AppState::TellStory { ended: false })),
        );

        app.add_systems(
            OnExit(VisibleGridState::Crater),
            (clear_selection,).run_if(in_state(AppState::TellStory { ended: false })),
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
                cleanup_pheromones,
                set_visible_grid_state_none,
                remove_loading_resources,
                despawn_view::<ElementTilemap>,
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

// TODO: This should live adjacent to the other AppState systems, but it requires knowledge of rendering - implying I need two separate states one for simulation and one for simulation + rendering
pub fn check_load_progress(
    mut next_app_state: ResMut<NextState<AppState>>,
    rendering_load_progress: Res<RenderingLoadProgress>,
    simulation_load_progress: ResMut<SimulationLoadProgress>,
) {
    if rendering_load_progress.element_sprite_sheet != LoadProgress::Success {
        return;
    }

    if simulation_load_progress.save_file == LoadProgress::Failure {
        next_app_state.set(AppState::MainMenu);
    } else if simulation_load_progress.save_file == LoadProgress::Success {
        next_app_state.set(AppState::FinishSetup);
    }
}
