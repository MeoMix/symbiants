use bevy::prelude::*;

#[derive(Resource)]
pub struct PheromoneVisibility(pub Visibility);

pub fn initialize_pheromone_resources(mut commands: Commands) {
    commands.insert_resource(PheromoneVisibility(Visibility::Visible));
}

/// Remove resources, etc.
pub fn cleanup_pheromones(mut commands: Commands) {
    commands.remove_resource::<PheromoneVisibility>();
}