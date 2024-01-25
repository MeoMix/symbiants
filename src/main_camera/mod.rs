use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera;

pub struct MainCameraPlugin;

/// The main camera for the app.
/// This is spawned on app startup because, conceptually, it is necessary to show the main menu.
/// Currently, this isn't true because `bevy_egui` doesn't require a camera to render, but Bevy's native UI does.
impl Plugin for MainCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}
