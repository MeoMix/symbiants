use super::{Position, WorldMap};
use bevy::{prelude::*, sprite::Anchor};

#[derive(Bundle)]
pub struct ElementBundle {
    sprite_bundle: SpriteBundle,
    element: Element,
    //  TODO: This should probably become a "Tile" in the future.
    position: Position,
}

#[derive(Component, PartialEq, Copy, Clone, Debug)]
pub enum Element {
    Air,
    Dirt,
    Sand,
}

impl ElementBundle {
    pub fn create_sand(position: Position) -> Self {
        // The view of the model position is just an inversion along the y-axis.
        let translation = Vec3::new(position.x as f32, -position.y as f32, 1.0);

        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation,
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.761, 0.698, 0.502),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },
            element: Element::Sand,
            position,
        }
    }

    pub fn create_air(position: Position) -> Self {
        // The view of the model position is just an inversion along the y-axis.
        let translation = Vec3::new(position.x as f32, -position.y as f32, 1.0);

        Self {
            sprite_bundle: SpriteBundle {
                // Air is transparent so reveal background
                visibility: Visibility::Hidden,
                transform: Transform {
                    translation,
                    ..default()
                },
                sprite: Sprite {
                    // Fully transparent color - could in theory set to something if air was made visible.
                    color: Color::rgba(0.0, 0.0, 0.0, 0.0),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },
            element: Element::Air,
            position,
        }
    }

    pub fn create_dirt(position: Position) -> Self {
        // The view of the model position is just an inversion along the y-axis.
        let translation = Vec3::new(position.x as f32, -position.y as f32, 1.0);

        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation,
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.514, 0.396, 0.224),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },
            element: Element::Dirt,
            position,
        }
    }
}

pub struct ElementsPlugin;

// Spawn interactive elements - air/dirt/sand. Air isn't visible, background is revealed in its place.
fn setup(mut commands: Commands, mut world_map: ResMut<WorldMap>) {
    let &width = world_map.width();
    let &height = world_map.height();
    let &surface_level = world_map.surface_level();

    // Test Sand
    let sand_bundles = (0..1).flat_map(|row_index| {
        (0..width).map(move |column_index| {
            (ElementBundle::create_sand(Position {
                x: column_index,
                y: row_index,
            }),)
        })
    });

    // Air & Dirt
    // NOTE: starting at 1 to skip sand
    let air_bundles = (1..(surface_level + 1)).flat_map(|row_index| {
        (0..width).map(move |column_index| {
            (
                // NOTE: row_index goes negative because 0,0 is top-left corner
                ElementBundle::create_air(Position {
                    x: column_index,
                    y: row_index,
                }),
            )
        })
    });

    let dirt_bundles = ((surface_level + 1)..height).flat_map(|row_index| {
        (0..width).map(move |column_index| {
            (ElementBundle::create_dirt(Position {
                x: column_index,
                y: row_index,
            }),)
        })
    });

    {
        for sand_bundle in sand_bundles {
            let position = sand_bundle.0.position;
            world_map
                .elements
                .insert(position, commands.spawn(sand_bundle).id());
        }

        for air_bundle in air_bundles {
            let position = air_bundle.0.position;
            world_map
                .elements
                .insert(position, commands.spawn(air_bundle).id());
        }

        for dirt_bundle in dirt_bundles {
            let position = dirt_bundle.0.position;
            world_map
                .elements
                .insert(position, commands.spawn(dirt_bundle).id());
        }
    }
}

impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}
