use super::WorldState;
use bevy::{prelude::*, window::PrimaryWindow};

// TODO: this is intentionally public - but is that good architecture?
#[derive(Component)]
pub struct Root;

#[derive(Component)]
struct MainCamera;

pub struct RootPlugin;

fn window_resize_system(
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<Root>>,
    world_state: Res<WorldState>,
) {
    let Ok(primary_window) = primary_window_query.get_single() else { panic!("missing primary window"); };

    let mut transform = query.single_mut();

    let (translation, scale) = get_world_container_transform(primary_window, &world_state);

    transform.translation = translation;
    transform.scale = scale;
}

// World dimensions are integer values (144/81) but <canvas/> has variable, floating point dimensions.
// Determine a scaling factor so world fills available screen space.
fn get_world_container_transform(window: &Window, world_state: &Res<WorldState>) -> (Vec3, Vec3) {
    let world_scale = (window.width() / world_state.width as f32)
        .max(window.height() / world_state.height as f32);

    // info!(
    //     "Window Height/Width: {}/{}, World Scale: {}",
    //     window.width(),
    //     window.height(),
    //     world_scale,
    // );

    (
        // translation:
        Vec3::new(window.width() / -2.0, window.height() / 2.0, 0.0),
        // scale:
        Vec3::new(world_scale, world_scale, 1.0),
    )
}

fn setup(
    mut commands: Commands,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    world_state: Res<WorldState>,
) {
    // Wrap in container and shift to top-left viewport so 0,0 is top-left corner.
    let Ok(primary_window) = primary_window_query.get_single() else { panic!("missing primary window") };

    commands.spawn((Camera2dBundle::default(), MainCamera));

    // Wrap in container and shift to top-left viewport so 0,0 is top-left corner.
    let (translation, scale) = get_world_container_transform(primary_window, &world_state);

    commands.spawn((
        SpatialBundle {
            transform: Transform {
                translation,
                scale,
                ..default()
            },
            ..default()
        },
        Root,
    ));
}

impl Plugin for RootPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Not sure what dragons await me for doing this. Intention is to allow plugins to query for Root in their own startup systems.
        // Inspiration comes from: https://github.com/Leafwing-Studios/Emergence/blob/4e1b12f72f1f73a460a4e2b836163890e31157e7/emergence_lib/src/ui/mod.rs#L32
        app.add_startup_system(setup.in_base_set(StartupSet::PreStartup))
            .add_system(window_resize_system.in_schedule(CoreSchedule::FixedUpdate));
    }
}
