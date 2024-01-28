use bevy::prelude::*;

#[derive(Component)]
pub struct UICamera;

pub struct UICameraPlugin;

/// Conceptually, a UI camera is necessary to show the main menu.
/// In practice, this isn't true because `bevy_egui` doesn't require a camera to render, but Bevy's native UI does.
/// Spawn one anyway to keep dependencies clear.
impl Plugin for UICameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), UICamera));
}
